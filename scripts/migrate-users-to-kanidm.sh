#!/bin/bash
# Migrate users from PostgreSQL to Kanidm
# Creates users in Kanidm, assigns to groups, updates PostgreSQL with kanidm_user_id

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
KANIDM_URL=${KANIDM_URL:-"https://idm.example.com"}
DATABASE_URL=${DATABASE_URL:-"postgresql://anthill:anthill@localhost:5432/anthill"}
INPUT_FILE=$1
DRY_RUN=${DRY_RUN:-"false"}
SKIP_CONFIRMATION=${SKIP_CONFIRMATION:-"false"}
DEFAULT_PASSWORD=${DEFAULT_PASSWORD:-""}

# Statistics
TOTAL_USERS=0
SUCCESS_COUNT=0
SKIP_COUNT=0
ERROR_COUNT=0

# ============================================================================
# Helper Functions
# ============================================================================

print_header() {
  echo ""
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${BLUE}$1${NC}"
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo ""
}

print_error() {
  echo -e "${RED}❌ Error: $1${NC}"
}

print_success() {
  echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
  echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
  echo -e "${CYAN}ℹ️  $1${NC}"
}

# ============================================================================
# Validation
# ============================================================================

if [ -z "$INPUT_FILE" ]; then
  echo -e "${RED}Usage: $0 <users_export.json>${NC}"
  echo ""
  echo "Example:"
  echo "  $0 users_export_20251103.json"
  echo ""
  echo "Environment variables:"
  echo "  DRY_RUN=true              - Don't make changes, just show what would happen"
  echo "  SKIP_CONFIRMATION=true    - Don't ask for confirmation"
  echo "  DEFAULT_PASSWORD=secret   - Set default password for new Kanidm users"
  exit 1
fi

if [ ! -f "$INPUT_FILE" ]; then
  print_error "File not found: $INPUT_FILE"
  exit 1
fi

# Validate JSON
if ! jq empty "$INPUT_FILE" 2>/dev/null; then
  print_error "Invalid JSON file: $INPUT_FILE"
  exit 1
fi

# Check database connection
if ! psql "$DATABASE_URL" -c "SELECT 1" > /dev/null 2>&1; then
  print_error "Cannot connect to database: $DATABASE_URL"
  exit 1
fi

# Check Kanidm login (if not dry run)
if [ "$DRY_RUN" != "true" ]; then
  if ! kanidm self whoami > /dev/null 2>&1; then
    print_error "Not logged in to Kanidm"
    echo "Run: kanidm login admin"
    exit 1
  fi
  
  KANIDM_USER=$(kanidm self whoami)
  print_success "Logged in to Kanidm as: $KANIDM_USER"
fi

# ============================================================================
# Configuration Summary
# ============================================================================

print_header "Kanidm User Migration"

echo "Configuration:"
echo "  Kanidm URL:       $KANIDM_URL"
echo "  Database:         $(echo $DATABASE_URL | sed 's/:[^:]*@/:***@/')"
echo "  Input File:       $INPUT_FILE"
echo "  Dry Run:          $DRY_RUN"
echo "  Default Password: $([ -z "$DEFAULT_PASSWORD" ] && echo "NOT SET (users must reset)" || echo "SET")"
echo ""

# Read users
USERS=$(cat "$INPUT_FILE")
TOTAL_USERS=$(echo "$USERS" | jq '. | length')

echo "Users to migrate: $TOTAL_USERS"
echo ""

if [ "$TOTAL_USERS" == "null" ] || [ "$TOTAL_USERS" == "0" ]; then
  print_warning "No users to migrate"
  exit 0
fi

# Show sample
echo "Sample (first 3 users):"
echo "$USERS" | jq -r '.[:3][] | "  - \(.email) (\(.tenant_slug)) - role: \(.role)"'
if [ "$TOTAL_USERS" -gt 3 ]; then
  echo "  ... and $((TOTAL_USERS - 3)) more"
fi
echo ""

# Confirmation
if [ "$SKIP_CONFIRMATION" != "true" ] && [ "$DRY_RUN" != "true" ]; then
  echo -e "${YELLOW}⚠️  This will create $TOTAL_USERS users in Kanidm and update PostgreSQL${NC}"
  read -p "Continue? (yes/no): " -r
  echo ""
  if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
    echo "Aborted."
    exit 0
  fi
fi

# ============================================================================
# Migration Process
# ============================================================================

print_header "Starting Migration"

# Create log file
LOG_FILE="migration_log_$(date +%Y%m%d_%H%M%S).txt"
echo "Log file: $LOG_FILE"
echo ""

for i in $(seq 0 $((TOTAL_USERS - 1))); do
  USER=$(echo "$USERS" | jq -r ".[$i]")
  
  # Extract user data
  USER_ID=$(echo "$USER" | jq -r '.user_id')
  EMAIL=$(echo "$USER" | jq -r '.email')
  FULL_NAME=$(echo "$USER" | jq -r '.full_name // .email')
  TENANT_SLUG=$(echo "$USER" | jq -r '.tenant_slug')
  TENANT_ID=$(echo "$USER" | jq -r '.tenant_id')
  ROLE=$(echo "$USER" | jq -r '.role')
  HAS_KANIDM=$(echo "$USER" | jq -r '.has_kanidm')
  
  echo -e "${CYAN}[$((i+1))/$TOTAL_USERS] $EMAIL${NC} (${TENANT_SLUG})"
  echo "  User ID:  $USER_ID"
  echo "  Role:     $ROLE"
  
  # Skip if already in Kanidm
  if [ "$HAS_KANIDM" == "true" ]; then
    print_warning "Already in Kanidm - skipping"
    ((SKIP_COUNT++))
    echo "$EMAIL,SKIP,Already in Kanidm" >> "$LOG_FILE"
    echo ""
    continue
  fi
  
  if [ "$DRY_RUN" == "true" ]; then
    print_info "[DRY RUN] Would create user in Kanidm"
    print_info "[DRY RUN] Would add to group: tenant_${TENANT_SLUG}_users"
    if [ "$ROLE" == "admin" ] || [ "$ROLE" == "super_admin" ]; then
      print_info "[DRY RUN] Would add to group: tenant_${TENANT_SLUG}_admins"
    fi
    print_info "[DRY RUN] Would update PostgreSQL with kanidm_user_id"
    ((SUCCESS_COUNT++))
    echo ""
    continue
  fi
  
  # ============================================================================
  # Create user in Kanidm
  # ============================================================================
  
  echo "  Creating user in Kanidm..."
  
  # Create person account
  if KANIDM_RESULT=$(kanidm person create "$EMAIL" "$FULL_NAME" --output json 2>&1); then
    KANIDM_USER_ID=$(echo "$KANIDM_RESULT" | jq -r '.uuid')
    
    if [ -z "$KANIDM_USER_ID" ] || [ "$KANIDM_USER_ID" == "null" ]; then
      print_error "Failed to extract Kanidm UUID from response"
      echo "$EMAIL,ERROR,No UUID in response: $KANIDM_RESULT" >> "$LOG_FILE"
      ((ERROR_COUNT++))
      echo ""
      continue
    fi
    
    print_success "Created in Kanidm: $KANIDM_USER_ID"
  else
    # Check if user already exists
    if echo "$KANIDM_RESULT" | grep -q "already exists"; then
      print_warning "User already exists in Kanidm"
      
      # Try to get existing user UUID
      if KANIDM_USER_INFO=$(kanidm person get "$EMAIL" --output json 2>&1); then
        KANIDM_USER_ID=$(echo "$KANIDM_USER_INFO" | jq -r '.uuid')
        print_info "Found existing UUID: $KANIDM_USER_ID"
      else
        print_error "Cannot get existing user info"
        echo "$EMAIL,ERROR,User exists but cannot get UUID" >> "$LOG_FILE"
        ((ERROR_COUNT++))
        echo ""
        continue
      fi
    else
      print_error "Failed to create user: $KANIDM_RESULT"
      echo "$EMAIL,ERROR,Create failed: $KANIDM_RESULT" >> "$LOG_FILE"
      ((ERROR_COUNT++))
      echo ""
      continue
    fi
  fi
  
  # Set password (if provided)
  if [ -n "$DEFAULT_PASSWORD" ]; then
    echo "  Setting default password..."
    if kanidm person set-password "$EMAIL" "$DEFAULT_PASSWORD" > /dev/null 2>&1; then
      print_success "Password set"
    else
      print_warning "Failed to set password (user must reset)"
    fi
  fi
  
  # ============================================================================
  # Add to tenant groups
  # ============================================================================
  
  echo "  Adding to tenant groups..."
  
  # Add to users group
  USERS_GROUP="tenant_${TENANT_SLUG}_users"
  if kanidm group add-members "$USERS_GROUP" "$EMAIL" 2>&1; then
    print_success "Added to $USERS_GROUP"
  else
    print_warning "Failed to add to $USERS_GROUP (group may not exist)"
  fi
  
  # Add to admins group if admin
  if [ "$ROLE" == "admin" ] || [ "$ROLE" == "super_admin" ]; then
    ADMINS_GROUP="tenant_${TENANT_SLUG}_admins"
    if kanidm group add-members "$ADMINS_GROUP" "$EMAIL" 2>&1; then
      print_success "Added to $ADMINS_GROUP"
    else
      print_warning "Failed to add to $ADMINS_GROUP (group may not exist)"
    fi
  fi
  
  # ============================================================================
  # Update PostgreSQL
  # ============================================================================
  
  echo "  Updating PostgreSQL..."
  
  if psql "$DATABASE_URL" -v ON_ERROR_STOP=1 <<SQL
    UPDATE users 
    SET 
      kanidm_user_id = '$KANIDM_USER_ID',
      kanidm_synced_at = NOW(),
      auth_method = 'dual',
      migration_completed_at = NOW()
    WHERE user_id = '$USER_ID';
SQL
  then
    print_success "Database updated"
    ((SUCCESS_COUNT++))
    echo "$EMAIL,SUCCESS,$KANIDM_USER_ID" >> "$LOG_FILE"
  else
    print_error "Failed to update database"
    echo "$EMAIL,ERROR,Database update failed" >> "$LOG_FILE"
    ((ERROR_COUNT++))
  fi
  
  echo ""
done

# ============================================================================
# Summary
# ============================================================================

print_header "Migration Complete"

echo "📊 Statistics:"
echo "  Total users:    $TOTAL_USERS"
echo "  ✅ Migrated:    $SUCCESS_COUNT"
echo "  ⏭️  Skipped:     $SKIP_COUNT"
echo "  ❌ Errors:      $ERROR_COUNT"
echo ""

if [ "$ERROR_COUNT" -gt 0 ]; then
  echo -e "${RED}⚠️  Some users failed to migrate. Check log: $LOG_FILE${NC}"
  echo ""
fi

# Show verification query
echo "🔍 Verification:"
echo ""
echo "Check migrated users in database:"
echo "  psql \$DATABASE_URL -c \"SELECT email, kanidm_user_id, auth_method FROM users WHERE migration_completed_at IS NOT NULL ORDER BY migration_completed_at DESC LIMIT 10;\""
echo ""

if [ "$SUCCESS_COUNT" -gt 0 ]; then
  print_success "Migration completed successfully!"
  echo ""
  echo "Next steps:"
  echo "  1. Verify users can login with OAuth2"
  echo "  2. Test dual authentication (password + OAuth2)"
  echo "  3. Monitor error logs"
  echo "  4. Send migration invitation emails to users"
fi

exit $([ "$ERROR_COUNT" -eq 0 ] && echo 0 || echo 1)
