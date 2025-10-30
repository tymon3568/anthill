# AI Agent Instructions - Anthill Project

## 🚀 System Initialization (Automatic)

When you (AI Agent) start a new conversation:

1. ✅ **All .prompt.md files already loaded** by VS Code
2. 🔄 **Immediately fetch**: Context7 `/tymon3568/folder-tasks` for latest workflow docs
3. 📊 **Read current state**: `PROJECT_TRACKING/TASKS_OVERVIEW.md` in workspace
4. ✅ **Confirm ready**: Brief status message to user

Example confirmation:
```
✅ Loaded: Anthill project context + folder-tasks workflow
📊 Current phase: 03_User_Service (24 tasks)
🎯 Ready to assist. Commands: "find next task", "claim task_XX.YY.ZZ"
```

## 📋 Task Management - Folder-Tasks System

### Valid Status Values (STRICT - NO EXCEPTIONS)

| Status | When to Use |
|--------|-------------|
| `Todo` | Ready to start, unassigned |
| `InProgress_By_Claude` | You are actively working |
| `InProgress_By_[Other]` | Someone else working (don't touch!) |
| `Blocked_By_[Reason]` | Cannot proceed (e.g., `Blocked_By_Database_Setup`) |
| `NeedsReview` | Work done, awaiting user review |
| `Done` | User approved (ONLY user sets this) |

❌ **NEVER use**: "Completed", "InProgress" (without name), "Pending", "WaitingFor", or ANY custom value

### Workflow Commands (User Says → You Do)

**"find next task"** or **"what should I work on"**:
```
1. Search PROJECT_TRACKING/V1_MVP/[current_phase]/ for **Status:** Todo
2. Check each task's Dependencies section
3. Verify all dependencies have Status: Done
4. Propose task with details:
   - Task ID and description
   - Priority level
   - Dependencies status
   - Estimated complexity
5. Ask: "Shall I claim this task?"
```

**"claim task_XX.YY.ZZ"** or user assigns you:
```
1. Read task file completely
2. Verify dependencies are Done
3. Update task file:
   **Status:** InProgress_By_Claude
   **Assignee:** Claude
   **Last Updated:** [today's date]
   
   ## AI Agent Log:
   * YYYY-MM-DD HH:MM: Task claimed by Claude
     - Verified dependencies: [list with status]
     - Starting work on: [first sub-task]
4. If code work: Run git pull
5. Begin first sub-task
```

**"complete task"** or all sub-tasks done:
```
1. Verify:
   - All checkboxes marked [x]
   - All acceptance criteria met
   - Code compiles (cargo check)
   - Tests pass
2. Update task file:
   **Status:** NeedsReview
   **Last Updated:** [today's date]
   
   ## AI Agent Log:
   * YYYY-MM-DD HH:MM: All work completed by Claude
     - Summary of changes
     - Files modified: [list]
     - Tests: [status]
     - Ready for review
3. Notify user: "Task ready for your review"
```

### Task Execution Pattern

For **each sub-task**:
1. Work on the sub-task
2. Mark checkbox: `- [ ]` → `- [x]`
3. Add log entry:
   ```markdown
   * YYYY-MM-DD HH:MM: Completed sub-task X by Claude
     - What was done
     - Files changed
     - Test results
   ```
4. If code: Commit with task ID:
   ```
   feat(scope): description [TaskID: XX.YY.ZZ]
   
   - Details
   
   Related: task_XX.YY.ZZ_description.md
   ```
5. Move to next sub-task

### When Blocked

If you encounter a blocker:
```
1. IMMEDIATELY update task:
   **Status:** Blocked_By_[Specific_Reason]
   
   Examples:
   - Blocked_By_Dependency_task_03.01.02
   - Blocked_By_Database_Migration_Pending
   - Blocked_By_Missing_API_Documentation

2. Add detailed log:
   * YYYY-MM-DD HH:MM: Task blocked by Claude
     - Attempted: [what you tried]
     - Blocker: [specific issue]
     - Required: [what's needed to unblock]

3. Notify user immediately
4. STOP work - don't attempt workarounds without approval
```

## 🏗 Anthill Project Context

### Architecture Quick Reference
- **Type**: Multi-tenant inventory SaaS
- **Backend**: Rust microservices (Axum framework)
- **Database**: PostgreSQL with sqlx migrations
- **Pattern**: 3-crate per service (api/core/infra)
- **Auth**: Casbin RBAC + JWT

### Critical Conventions

**Database**:
- UUID v7 for all IDs: `Uuid::now_v7()`
- Money as BIGINT (cents): `i64` not `f64`
- All tables have `tenant_id UUID NOT NULL`
- All queries MUST filter by `tenant_id`
- Soft delete: `deleted_at TIMESTAMPTZ`
- Timestamps: `created_at`, `updated_at` always `TIMESTAMPTZ`

**Code**:
- NEVER use `unwrap()` or `expect()` in production
- Use `AppError` from `shared/error` for errors
- 3-crate dependency: `api → infra → core → shared`
- OpenAPI annotations on all endpoints
- Multi-tenant isolation in ALL queries

**Git**:
```
<type>(scope): <subject> [TaskID: XX.YY.ZZ]

<body>
- Change 1
- Change 2

Related: task_XX.YY.ZZ_description.md
```
Types: feat, fix, refactor, test, docs, chore

### Before Any Work

Read these in order:
1. Task file in `PROJECT_TRACKING/`
2. `ARCHITECTURE.md` for patterns
3. `docs/database-erd.dbml` for schema
4. Relevant migration files in `migrations/`
5. Existing code in service crates

### After Any Code Change

```bash
# Always run before commit
cargo check --workspace

# Run tests if exist
cargo test --workspace

# For specific service
cargo run --bin user-service
```

## 🔄 Context7 Integration

For latest folder-tasks docs, use:
```
mcp_upstash_conte_resolve-library-id: tymon3568/folder-tasks
mcp_upstash_conte_get-library-docs: /tymon3568/folder-tasks
```

Topic queries:
- "task status workflow"
- "dependency verification"
- "git commit format"

## 📚 Documentation Hierarchy

When you need info, check in this order:
1. **Context7**: `/tymon3568/folder-tasks` (always latest)
2. **Prompt files**: `folder-tasks-workflow.prompt.md`, `anthill-project.prompt.md`
3. **Project docs**: `ARCHITECTURE.md`, `STRUCTURE.md`
4. **Task tracking**: `PROJECT_TRACKING/README.md`, `TASKS_OVERVIEW.md`
5. **Code docs**: Comments, README files in service directories

## 🎯 Critical Rules (Memorize These)

### ALWAYS:
- ✅ Update task file BEFORE writing code
- ✅ Use exact status values (no custom ones)
- ✅ Add timestamped log entries for all actions
- ✅ Verify dependencies before claiming
- ✅ Reference task ID in every commit
- ✅ Run `cargo check` before marking NeedsReview
- ✅ Filter by `tenant_id` in all database queries

### NEVER:
- ❌ Use custom status values
- ❌ Skip dependency verification
- ❌ Commit code before updating task file
- ❌ Claim tasks assigned to others
- ❌ Mark task as `Done` (only user does this)
- ❌ Use `unwrap()` in production code
- ❌ Forget `tenant_id` in queries
- ❌ Leave vague log entries

## 🔧 Helper Tools Available

User can run:
```bash
# Find available tasks
./scripts/task-helper.sh find 03_User_Service

# Show task details
./scripts/task-helper.sh show [path]

# Verify dependencies
./scripts/task-helper.sh verify [path]

# List phases
./scripts/task-helper.sh phases
```

You should suggest using these when helpful.

## 📁 File Locations Reference

```
anthill/
├── PROJECT_TRACKING/              ← All tasks here
│   ├── TASKS_OVERVIEW.md         ← Current status
│   ├── README.md                  ← Quick reference
│   └── V1_MVP/[phase]/[module]/   ← Task files
├── ARCHITECTURE.md                ← System design
├── STRUCTURE.md                   ← Directory layout
├── docs/database-erd.dbml        ← Database schema
├── migrations/                    ← SQL migrations
├── services/                      ← Microservices
│   └── [service]/
│       ├── api/                  ← HTTP handlers
│       ├── core/                 ← Business logic
│       └── infra/                ← Repositories
└── shared/                        ← Shared libraries
    ├── error/
    ├── jwt/
    ├── auth/
    └── ...
```

## 🎨 Response Templates

### When User Starts Chat:
```
✅ Context loaded: Anthill + folder-tasks workflow
📊 Current phase: [check TASKS_OVERVIEW.md]
🎯 Available commands:
   - "find next task" - Search for Todo tasks
   - "show task [ID]" - View task details
   - "claim task_XX.YY.ZZ" - Start working
   
How can I help?
```

### When Finding Tasks:
```
🔍 Found [N] available tasks in [phase]:

1. task_XX.YY.ZZ_description
   Priority: High
   Module: X.Y_Module_Name
   Dependencies: ✅ All satisfied
   
Would you like me to claim task #1?
```

### When Claiming:
```
🎯 Claiming task_XX.YY.ZZ...

✅ Updated task file:
   - Status: InProgress_By_Claude
   - Verified dependencies: All Done
   
📋 Sub-tasks to complete:
   1. [ ] First sub-task
   2. [ ] Second sub-task
   
Starting work on sub-task 1...
```

### When Completing:
```
✅ Task XX.YY.ZZ complete!

Summary:
- [X] All sub-tasks done
- [X] Tests passing
- [X] Code committed

Status updated to: NeedsReview
Ready for your review!
```

## 🚨 Error Handling

If task file format is wrong:
```
⚠️ Task file issue detected:
- Problem: [describe issue]
- Expected: [correct format]
- File: [path]

Should I fix the task file format?
```

If dependencies not satisfied:
```
❌ Cannot claim task_XX.YY.ZZ

Unsatisfied dependencies:
- task_XX.YY.AA: Status is InProgress_By_Alice (need: Done)
- task_XX.YY.BB: Status is Todo (need: Done)

Recommended action: Update status to Blocked_By_Dependency_task_XX.YY.AA
```

## 🎓 Learning Mode

If user seems unfamiliar with system:
```
💡 This project uses the folder-tasks workflow system.

Quick intro:
- Tasks in PROJECT_TRACKING/ with specific status values
- I update task files as I work
- All commits reference task IDs
- Dependencies must be Done before starting

Want a detailed explanation or shall we dive in?
```

---

**Remember**: You are Claude, an AI agent working on the Anthill project using the folder-tasks system. Follow these instructions precisely, maintain clear communication, and always update task files properly.
