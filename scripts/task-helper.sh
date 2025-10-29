#!/bin/bash
# Task Helper Script for Folder-Tasks Workflow
# Usage: ./scripts/task-helper.sh [command] [args]

set -e

TRACKING_DIR="PROJECT_TRACKING/V1_MVP"
TEMPLATE_FILE="PROJECT_TRACKING/TASK_TEMPLATE.md"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function: Find all Todo tasks in a phase
find_todo_tasks() {
    local phase=${1:-"03_User_Service"}
    local search_path="$TRACKING_DIR/$phase"
    
    if [ ! -d "$search_path" ]; then
        echo -e "${RED}❌ Phase directory not found: $search_path${NC}"
        return 1
    fi
    
    echo -e "${BLUE}🔍 Searching for Todo tasks in $phase...${NC}\n"
    
    local found=0
    while IFS= read -r file; do
        if grep -q "^\*\*Status:\*\* Todo" "$file" 2>/dev/null; then
            found=$((found + 1))
            local task_id=$(basename "$file" .md)
            local priority=$(grep "^\*\*Priority:\*\*" "$file" | sed 's/.*: //')
            local module=$(grep "^\*\*Module:\*\*" "$file" | sed 's/.*: //')
            
            echo -e "${GREEN}✓ Found:${NC} $task_id"
            echo -e "  ${YELLOW}Priority:${NC} $priority"
            echo -e "  ${YELLOW}Module:${NC} $module"
            echo -e "  ${YELLOW}Path:${NC} $file"
            echo ""
        fi
    done < <(find "$search_path" -type f -name "task_*.md")
    
    if [ $found -eq 0 ]; then
        echo -e "${YELLOW}⚠ No Todo tasks found in $phase${NC}"
    else
        echo -e "${GREEN}📊 Total: $found task(s) available${NC}"
    fi
}

# Function: Find all tasks by status
find_by_status() {
    local status=$1
    local phase=${2:-"03_User_Service"}
    local search_path="$TRACKING_DIR/$phase"
    
    if [ -z "$status" ]; then
        echo -e "${RED}❌ Please specify status${NC}"
        echo "  Valid: Todo, InProgress_By_*, Blocked_By_*, NeedsReview, Done"
        return 1
    fi
    
    echo -e "${BLUE}🔍 Searching for '$status' tasks in $phase...${NC}\n"
    
    local found=0
    while IFS= read -r file; do
        if grep -q "^\*\*Status:\*\* $status" "$file" 2>/dev/null; then
            found=$((found + 1))
            local task_id=$(basename "$file" .md)
            echo -e "${GREEN}✓${NC} $task_id → $file"
        fi
    done < <(find "$search_path" -type f -name "task_*.md")
    
    echo -e "\n${GREEN}📊 Total: $found task(s) with status '$status'${NC}"
}

# Function: Show task details
show_task() {
    local task_file=$1
    
    if [ -z "$task_file" ]; then
        echo -e "${RED}❌ Please specify task file path${NC}"
        return 1
    fi
    
    if [ ! -f "$task_file" ]; then
        echo -e "${RED}❌ Task file not found: $task_file${NC}"
        return 1
    fi
    
    echo -e "${BLUE}📋 Task Details:${NC}\n"
    echo -e "${YELLOW}════════════════════════════════════════${NC}"
    head -30 "$task_file"
    echo -e "${YELLOW}════════════════════════════════════════${NC}\n"
    
    # Show dependencies
    echo -e "${BLUE}📦 Dependencies:${NC}"
    grep -A 5 "## Dependencies:" "$task_file" || echo "  None specified"
    echo ""
    
    # Show sub-tasks status
    echo -e "${BLUE}✅ Sub-tasks Progress:${NC}"
    local total=$(grep -c "^- \[ \]" "$task_file" 2>/dev/null || echo 0)
    local done=$(grep -c "^- \[x\]" "$task_file" 2>/dev/null || echo 0)
    echo -e "  Completed: $done / $total"
}

# Function: Verify dependencies
verify_dependencies() {
    local task_file=$1
    
    if [ ! -f "$task_file" ]; then
        echo -e "${RED}❌ Task file not found: $task_file${NC}"
        return 1
    fi
    
    echo -e "${BLUE}🔍 Verifying dependencies for: $(basename "$task_file")${NC}\n"
    
    # Extract dependencies section
    local in_deps=0
    local all_done=1
    
    while IFS= read -r line; do
        if [[ "$line" =~ ^##\ Dependencies: ]]; then
            in_deps=1
            continue
        fi
        
        if [[ $in_deps -eq 1 ]]; then
            # Stop at next section
            if [[ "$line" =~ ^## ]]; then
                break
            fi
            
            # Parse dependency line
            if [[ "$line" =~ Task:.*'`'([^'`']+'`') ]]; then
                local dep_file="${BASH_REMATCH[1]}"
                local dep_path="$TRACKING_DIR/$dep_file"
                
                if [ -f "$dep_path" ]; then
                    local dep_status=$(grep "^\*\*Status:\*\*" "$dep_path" | sed 's/.*: //')
                    if [ "$dep_status" == "Done" ]; then
                        echo -e "${GREEN}✓${NC} $dep_file → ${GREEN}Done${NC}"
                    else
                        echo -e "${RED}✗${NC} $dep_file → ${YELLOW}$dep_status${NC}"
                        all_done=0
                    fi
                else
                    echo -e "${RED}✗${NC} $dep_file → ${RED}File not found${NC}"
                    all_done=0
                fi
            fi
        fi
    done < "$task_file"
    
    echo ""
    if [ $all_done -eq 1 ]; then
        echo -e "${GREEN}✅ All dependencies satisfied! Task ready to claim.${NC}"
        return 0
    else
        echo -e "${RED}❌ Some dependencies not satisfied. Cannot start task yet.${NC}"
        return 1
    fi
}

# Function: Create new task from template
create_task() {
    local phase=$1
    local module=$2
    local task_num=$3
    local description=$4
    
    if [ -z "$phase" ] || [ -z "$module" ] || [ -z "$task_num" ] || [ -z "$description" ]; then
        echo -e "${RED}❌ Usage: create <phase> <module> <task_num> <description>${NC}"
        echo "  Example: create 03_User_Service 3.2_Casbin_Authorization 03.02.15 implement_role_hierarchy"
        return 1
    fi
    
    local target_dir="$TRACKING_DIR/$phase/$module"
    local filename="task_${task_num}_${description}.md"
    local target_path="$target_dir/$filename"
    
    if [ ! -d "$target_dir" ]; then
        echo -e "${YELLOW}⚠ Creating directory: $target_dir${NC}"
        mkdir -p "$target_dir"
    fi
    
    if [ -f "$target_path" ]; then
        echo -e "${RED}❌ Task file already exists: $target_path${NC}"
        return 1
    fi
    
    if [ ! -f "$TEMPLATE_FILE" ]; then
        echo -e "${RED}❌ Template file not found: $TEMPLATE_FILE${NC}"
        return 1
    fi
    
    # Copy template and replace placeholders
    cp "$TEMPLATE_FILE" "$target_path"
    
    local today=$(date +%Y-%m-%d)
    sed -i.bak \
        -e "s|\[Phase\]|$phase|g" \
        -e "s|\[Module\]|$module|g" \
        -e "s|\[XX.YY.ZZ\]|$task_num|g" \
        -e "s|\[description\]|$description|g" \
        -e "s|YYYY-MM-DD|$today|g" \
        "$target_path"
    
    rm "${target_path}.bak"
    
    echo -e "${GREEN}✅ Created new task: $target_path${NC}"
    echo -e "${YELLOW}📝 Please edit the file to fill in details${NC}"
}

# Function: List all phases
list_phases() {
    echo -e "${BLUE}📂 Available phases in V1_MVP:${NC}\n"
    
    for dir in "$TRACKING_DIR"/*; do
        if [ -d "$dir" ]; then
            local phase=$(basename "$dir")
            local task_count=$(find "$dir" -type f -name "task_*.md" | wc -l)
            echo -e "${GREEN}▶${NC} $phase ${YELLOW}($task_count tasks)${NC}"
        fi
    done
}

# Function: Show help
show_help() {
    cat << EOF
${BLUE}═══════════════════════════════════════════════════════════${NC}
${GREEN}Task Helper Script for Folder-Tasks Workflow${NC}
${BLUE}═══════════════════════════════════════════════════════════${NC}

${YELLOW}Usage:${NC}
  ./scripts/task-helper.sh [command] [arguments]

${YELLOW}Commands:${NC}

  ${GREEN}find [phase]${NC}
      Find all Todo tasks in specified phase
      Example: ./scripts/task-helper.sh find 03_User_Service

  ${GREEN}status <status> [phase]${NC}
      Find all tasks with specified status
      Example: ./scripts/task-helper.sh status NeedsReview 03_User_Service

  ${GREEN}show <task_file>${NC}
      Show details of a specific task
      Example: ./scripts/task-helper.sh show PROJECT_TRACKING/.../task_03.02.10_*.md

  ${GREEN}verify <task_file>${NC}
      Verify all dependencies for a task
      Example: ./scripts/task-helper.sh verify PROJECT_TRACKING/.../task_03.02.10_*.md

  ${GREEN}create <phase> <module> <task_num> <description>${NC}
      Create new task from template
      Example: ./scripts/task-helper.sh create 03_User_Service 3.2_Casbin 03.02.15 implement_roles

  ${GREEN}phases${NC}
      List all available phases with task counts

  ${GREEN}help${NC}
      Show this help message

${YELLOW}Valid Task Statuses:${NC}
  - Todo
  - InProgress_By_[Agent]
  - Blocked_By_[Reason]
  - NeedsReview
  - Done

${BLUE}═══════════════════════════════════════════════════════════${NC}
EOF
}

# Main command dispatcher
case "${1:-help}" in
    find)
        find_todo_tasks "$2"
        ;;
    status)
        find_by_status "$2" "$3"
        ;;
    show)
        show_task "$2"
        ;;
    verify)
        verify_dependencies "$2"
        ;;
    create)
        create_task "$2" "$3" "$4" "$5"
        ;;
    phases)
        list_phases
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo -e "${RED}❌ Unknown command: $1${NC}\n"
        show_help
        exit 1
        ;;
esac
