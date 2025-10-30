# Task Management Quick Reference

## 📋 Valid Status Values (ONLY THESE - NO EXCEPTIONS)

| Status | Meaning | When to Use |
|--------|---------|-------------|
| `Todo` | Ready to start, unassigned | Initial state or returned from blocked |
| `InProgress_By_[Agent]` | Actively being worked on | Agent has claimed and started work |
| `Blocked_By_[Reason]` | Cannot proceed | Missing dependency, blocker found |
| `NeedsReview` | Work complete, needs review | All sub-tasks done, ready for approval |
| `Done` | Reviewed and approved | User has reviewed and approved |

### ❌ INVALID Status Examples:
- `Completed` (use `Done`)
- `InProgress` (missing agent name)
- `Pending` (use `Todo` or `Blocked_By_[Reason]`)
- `WaitingFor_Database` (use `Blocked_By_Database_Setup`)
- `Completed_Pending_Review` (use `NeedsReview`)
- Any custom value not listed above

## 🔄 Task Workflow

```
1. Todo
   ↓ (Agent claims task)
2. InProgress_By_Claude
   ↓ (Work on sub-tasks, update logs)
   ├─→ Blocked_By_[Reason] (if blocker found)
   │   ↓ (blocker resolved)
   │   └─→ Back to InProgress_By_Claude
   ↓ (All sub-tasks complete)
3. NeedsReview
   ↓ (User reviews)
4. Done
```

## ✅ Critical Rules

### DO:
- ✅ ALWAYS update task file BEFORE writing code
- ✅ ALWAYS verify dependencies are Done before claiming
- ✅ ALWAYS add log entry for each significant action
- ✅ ALWAYS reference task ID in git commits
- ✅ ALWAYS run tests before marking NeedsReview
- ✅ ALWAYS use exact status values from the table above

### DON'T:
- ❌ NEVER use custom status values
- ❌ NEVER skip dependency verification
- ❌ NEVER commit code before updating task file
- ❌ NEVER claim tasks assigned to others
- ❌ NEVER mark task Done yourself (user does this)
- ❌ NEVER leave incomplete work in InProgress state

## 📝 Task Claiming Process

1. **Find Task**: Search for `**Status:** Todo` in PROJECT_TRACKING/
2. **Check Dependencies**: Verify all dependencies show `Status: Done`
3. **Check Assignee**: Ensure field is empty or unassigned
4. **Update Task File**:
   ```markdown
   **Status:** InProgress_By_Claude
   **Assignee:** Claude
   **Last Updated:** 2025-10-29
   
   ## AI Agent Log:
   ---
   *   2025-10-29 14:30: Task claimed by Claude
       - Verified dependencies: All Done ✓
       - Starting work on sub-task 1
   ```
5. **Git Pull**: `git pull origin main` (if code work)
6. **Begin Work**: Start on first sub-task

## 🎯 Sub-task Execution

For each sub-task:
1. Work on the sub-task
2. Mark checkbox: `- [ ]` → `- [x]`
3. Add log entry with details
4. Commit code (if applicable)
5. Move to next sub-task

Example log entry:
```markdown
*   2025-10-29 15:45: Completed sub-task 3 by Claude
    - Implemented user profile repository
    - Added 5 database queries with tenant isolation
    - Files: services/user_service/infra/src/auth/profile_repository.rs
    - Tests: All passing ✓
```

## 🚦 Completing a Task

When all sub-tasks are done:

1. **Verify Completion**:
   - [ ] All sub-task checkboxes marked `[x]`
   - [ ] All acceptance criteria met
   - [ ] Code compiles: `cargo check --workspace`
   - [ ] Tests pass (if applicable)
   - [ ] No uncommitted changes
   - [ ] Documentation updated

2. **Update Task File**:
   ```markdown
   **Status:** NeedsReview
   **Last Updated:** 2025-10-29
   
   ## AI Agent Log:
   ---
   [... previous entries ...]
   *   2025-10-29 17:00: All work completed by Claude
       - Summary: [brief summary of all changes]
       - Files modified: [list key files]
       - Tests: All passing ✓
       - Ready for user review
   ```

3. **Notify User**: Inform that task is ready for review

## 🚫 When Blocked

If you encounter a blocker:

1. **Update Status Immediately**:
   ```markdown
   **Status:** Blocked_By_Dependency_task_03.01.02
   ```
   or
   ```markdown
   **Status:** Blocked_By_Database_Migration_Pending
   ```

2. **Add Detailed Log**:
   ```markdown
   *   2025-10-29 16:15: Task blocked by Claude
       - Attempted: [what you tried to do]
       - Blocker: [specific issue encountered]
       - Required: [what's needed to unblock]
       - Dependencies: task_03.01.02 still InProgress_By_Alice
       - Status: Blocked_By_Dependency_task_03.01.02
   ```

3. **Notify User**: Explain blocker and what's needed

4. **Stop Work**: Do NOT attempt workarounds without approval

## 🔍 Finding Next Task

Search pattern in PROJECT_TRACKING/:
```
grep -r "**Status:** Todo" PROJECT_TRACKING/V1_MVP/03_User_Service/
```

Check:
1. Dependencies section - all must be Done
2. Assignee field - must be empty
3. Priority - consider High priority first
4. Module - work within logical grouping

## 📊 Task File Anatomy

```markdown
# Task: [Clear descriptive title]

**Task ID:** V1_MVP/03_User_Service/3.2_Module/task_03.02.XX_description.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.2_Module_Name
**Priority:** High|Medium|Low
**Status:** Todo|InProgress_By_X|Blocked_By_X|NeedsReview|Done
**Assignee:** [Empty or Agent Name]
**Created Date:** YYYY-MM-DD
**Last Updated:** YYYY-MM-DD

## Detailed Description:
[What needs to be done]

## Specific Sub-tasks:
- [ ] 1. First sub-task
- [ ] 2. Second sub-task
- [ ] 3. Third sub-task

## Acceptance Criteria:
- [ ] Criterion 1
- [ ] Criterion 2

## Dependencies:
*   Task: `task_03.02.05_dependency.md` (Status: Done)

## Related Documents:
*   `path/to/related/file.md`

## Notes / Discussion:
---

## AI Agent Log:
---
*   YYYY-MM-DD HH:MM: [Action] by [Agent]
    - Details
```

## 🎨 Git Commit Format

```
<type>(scope): <subject> [TaskID: XX.YY.ZZ]

<body>
- Change 1
- Change 2

Related: task_XX.YY.ZZ_description.md
```

**Types**: feat, fix, refactor, test, docs, chore

**Example**:
```
feat(user_service): implement profile endpoints [TaskID: 03.03.05]

- Added GET /api/v1/profile endpoint
- Added PUT /api/v1/profile endpoint  
- Implemented profile completeness calculation
- All integration tests passing

Related: task_03.03.05_profile_management.md
```

## 🔧 Common Task Types

### Database Tasks:
1. Update migration files
2. Update docs/database-erd.dbml
3. Update domain models in core/
4. Update repository queries in infra/
5. Verify with `cargo check`

### API Tasks:
1. Define DTOs in core/dto/
2. Add handlers in api/src/
3. Add OpenAPI annotations
4. Update tests
5. Verify with `cargo run`

### Testing Tasks:
1. Create test files
2. Write test cases
3. Add test helpers
4. Verify with `cargo test`
5. Update README if needed

## 📞 Need Help?

- **Workflow questions**: Read `folder-tasks-workflow.prompt.md`
- **Project structure**: Read `anthill-project.prompt.md`
- **Coding standards**: Read `.github/copilot-instructions.md`
- **Architecture**: Read `ARCHITECTURE.md`
- **Database schema**: Read `docs/database-erd.dbml`

## 🎯 Success Metrics

A well-managed task has:
- ✅ Clear, actionable sub-tasks
- ✅ All checkboxes properly maintained
- ✅ Detailed AI Agent Log with timestamps
- ✅ Proper status transitions
- ✅ Git commits referencing task ID
- ✅ No gaps in the workflow
- ✅ Ready for review when marked NeedsReview
