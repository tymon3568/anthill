# AI Agent Prompt System Guide

## 🚀 Auto-Loading for GitHub Copilot

### Current Setup

GitHub Copilot **automatically loads** all `.prompt.md` files from:
```
~/.config/Code/User/prompts/*.prompt.md
```

### How It Works

When you start a conversation with GitHub Copilot in VS Code, it:
1. Automatically reads ALL `.prompt.md` files in the prompts directory
2. Uses them as context for the conversation
3. No manual loading required!

### Our Files

```
~/.config/Code/User/prompts/
├── general.prompt.md                    ← Master instructions (auto-loaded)
├── folder-tasks-workflow.prompt.md      ← Workflow rules (auto-loaded)
├── anthill-project.prompt.md           ← Project context (auto-loaded)
└── README.md                            ← Documentation (not auto-loaded)
```

## 📝 Recommended: Update `general.prompt.md`

Replace your current `general.prompt.md` with this enhanced version:

```markdown
# AI Agent Instructions - Anthill Project

## System Initialization (Automatic)

When starting a new conversation, I will automatically:

1. ✅ **Context Loaded**: This file and all other .prompt.md files are already loaded by VS Code
2. 🔄 **Fetch Latest Docs**: Use Context7 to get `/tymon3568/folder-tasks` documentation
3. 📊 **Read Project State**: Check `PROJECT_TRACKING/TASKS_OVERVIEW.md` for current status
4. ✅ **Ready**: I'm now ready to work with full context

## Core Workflow Integration

### For Task Management
All task work follows the **folder-tasks** system. Key rules:

**Valid Status Values ONLY**:
- `Todo` - Ready to start
- `InProgress_By_[Agent]` - Currently working (e.g., InProgress_By_Claude)
- `Blocked_By_[Reason]` - Cannot proceed (e.g., Blocked_By_Database_Setup)
- `NeedsReview` - Work complete, awaiting review
- `Done` - Reviewed and approved

**Never use**: "Completed", "InProgress" (without agent name), "Pending", or any custom values.

### Workflow Commands

User can say:
- **"find next task"** → I search PROJECT_TRACKING for available Todo tasks
- **"claim task_XX.YY.ZZ"** → I update task file and begin work
- **"show task status"** → I check current task progress
- **"complete task"** → I update status to NeedsReview

### Task Work Pattern

When working on tasks:
1. **Always update task file BEFORE coding**
2. **Verify all dependencies are Done**
3. **Add detailed log entries with timestamps**
4. **Mark sub-tasks as completed**
5. **Reference task ID in all git commits**
6. **Update status to NeedsReview when done** (not Done - user does that)

### Git Commit Format
```
<type>(scope): <subject> [TaskID: XX.YY.ZZ]

<body>
- Detail 1
- Detail 2

Related: task_XX.YY.ZZ_description.md
```

Types: feat, fix, refactor, test, docs, chore

## Project-Specific Context

### Anthill Architecture
- Multi-tenant inventory SaaS
- Rust microservices (Axum framework)
- 3-crate pattern: api/ core/ infra/
- PostgreSQL with sqlx
- Multi-tenancy via tenant_id column

### Key Conventions
- Use UUID v7 for all IDs (timestamp-based)
- Money stored as BIGINT (cents/xu)
- All tables have soft delete (deleted_at)
- Multi-tenant queries MUST filter by tenant_id
- Never use unwrap() or expect() in production code

### Before Any Work
1. Read relevant task file in PROJECT_TRACKING/
2. Check ARCHITECTURE.md for patterns
3. Verify database schema in migrations/ and docs/database-erd.dbml
4. Run `cargo check --workspace` after changes

## Context7 Integration

For latest folder-tasks documentation, I will fetch:
```
/tymon3568/folder-tasks
```

This ensures I always have the most current workflow rules.

## Helper Tools

Users can use CLI tool:
```bash
./scripts/task-helper.sh [command]

Commands:
- find [phase]              Find Todo tasks
- show <task_file>          Show task details  
- verify <task_file>        Check dependencies
- status <status> [phase]   Find by status
- phases                    List all phases
```

## Critical Rules (STRICT)

### DO:
✅ Update task files BEFORE writing code
✅ Use exact status values from approved list
✅ Add timestamped log entries for all actions
✅ Verify dependencies before claiming tasks
✅ Reference task ID in every commit
✅ Run tests before marking NeedsReview

### DON'T:
❌ Use custom status values
❌ Skip dependency verification
❌ Commit code before updating task file
❌ Claim tasks assigned to others
❌ Mark task as Done (only user does this)
❌ Leave vague or missing log entries

## Documentation Hierarchy

When I need information:
1. Context7: `/tymon3568/folder-tasks` (always latest)
2. `folder-tasks-workflow.prompt.md` (workflow rules)
3. `anthill-project.prompt.md` (project patterns)
4. `PROJECT_TRACKING/README.md` (quick reference)
5. Project docs: ARCHITECTURE.md, STRUCTURE.md

## Session Start Confirmation

At the start of each session, I will confirm:
> "✅ Context loaded: Anthill project with folder-tasks workflow
> 📊 Current phase: [from TASKS_OVERVIEW.md]
> 🎯 Ready to assist. Say 'find next task' or specify what you need."

This lets you know I'm ready with full context.
```

## 🎨 Alternative Approach: Use VS Code Settings

Add to `.vscode/settings.json`:

```json
{
  "github.copilot.chat.systemPrompts": [
    "Load and follow instructions from ~/.config/Code/User/prompts/folder-tasks-workflow.prompt.md and anthill-project.prompt.md",
    "Always use Context7 to fetch latest /tymon3568/folder-tasks documentation",
    "Check PROJECT_TRACKING/TASKS_OVERVIEW.md for current project state"
  ]
}
```

## 🔧 Best Practice: Session Start Script

Create a **chat template** file:

```markdown
File: ~/.config/Code/User/prompts/SESSION_START.md

# Session Start Template

Copy/paste this to start a new AI conversation with full context:

---

I'm starting work on the Anthill project. Please:

1. ✅ Confirm you have loaded folder-tasks-workflow.prompt.md and anthill-project.prompt.md
2. 🔄 Fetch latest docs from Context7: /tymon3568/folder-tasks
3. 📊 Check PROJECT_TRACKING/TASKS_OVERVIEW.md for current status
4. 🎯 Ready for work

After loading, please show me:
- Current active phase
- Number of Todo tasks available
- Your understanding of valid status values

---
```

Then you just copy/paste this at session start!

## 💡 Recommended Solution

**Option 1: Update general.prompt.md** (Best)
- Update your `general.prompt.md` with the enhanced version above
- AI automatically gets all context on every conversation
- No manual steps needed

**Option 2: Quick start template** (Backup)
- Keep `SESSION_START.md` template
- Copy/paste at beginning of new chats
- Ensures AI loads everything explicitly

**Option 3: VS Code settings** (Advanced)
- Add to workspace settings
- Requires VS Code restart when changed
- Good for team collaboration

## 🎯 My Recommendation

**Use enhanced `general.prompt.md`** because:
1. ✅ Automatically loaded every session
2. ✅ No manual copy/paste needed
3. ✅ Contains all critical rules
4. ✅ References other prompt files
5. ✅ Clear workflow commands

Would you like me to create the enhanced `general.prompt.md` file for you to copy to `~/.config/Code/User/prompts/`?
