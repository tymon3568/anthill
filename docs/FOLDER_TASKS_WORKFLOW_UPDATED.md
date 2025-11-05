# Folder-Tasks Workflow Rules (Updated from Context7)

## Core Principles
- ALWAYS use Context7 to fetch latest folder-tasks documentation: `/tymon3568/folder-tasks`
- NEVER use custom status values - only: `Todo`, `InProgress_By_[Agent]`, `Blocked_By_[Reason]`, `NeedsReview`, `Done`
- Task files are the single source of truth - update them FIRST before any code work
- All content in task files MUST be in English

## Pre-Task Checklist (MANDATORY)
Before claiming ANY task:
1. ✅ Read task file completely from PROJECT_TRACKING/
2. ✅ Verify ALL dependencies have Status: Done
3. ✅ Check "Assignee" field - respect existing assignments
4. ✅ Update Status to `InProgress_By_[Agent]` (e.g., `InProgress_By_Grok`)
5. ✅ Add initial log entry with timestamp to AI Agent Log
6. ✅ Run `git pull` if task involves code changes

## During Task Execution
For EACH sub-task completed:
- Mark checkbox: `- [ ]` → `- [x]`
- Add detailed log entry with timestamp
- Commit code with task ID in message: `git commit -m "task_XX.YY.ZZ: description"`
- Reference task file in commit body

## Task Completion Flow
```text
Todo → InProgress_By_Agent → All sub-tasks done → NeedsReview → User reviews → Done
           ↓
      Blocked_By_[Reason] (if issues arise)
```

## Status Transitions (STRICT - NO EXCEPTIONS)
```text
Valid statuses ONLY:
- Todo                      (Initial state, ready to be claimed)
- InProgress_By_[Agent]     (Agent actively working)
- Blocked_By_[Reason]       (Cannot proceed - MUST specify reason)
- NeedsReview              (Work complete, awaiting review)
- Done                      (Reviewed and approved)

❌ INVALID: Completed, InProgress, Pending, WaitingFor, Custom_Status
```
```text
Valid statuses ONLY:
- Todo                      (Initial state, ready to be claimed)
- InProgress_By_[Agent]     (Agent actively working)
- Blocked_By_[Reason]       (Cannot proceed - MUST specify reason)
- NeedsReview              (Work complete, awaiting review)
- Done                      (Reviewed and approved)

❌ INVALID: Completed, InProgress, Pending, WaitingFor, Custom_Status
```

## Git Workflow for Code-Related Tasks
```bash
# Before starting code work:
git pull origin main
# Or create feature branch:
git checkout -b feature/task_XX.YY.ZZ_description
```

## Dependency Verification Logic
```text
# Agent verification process:
# 1. Read dependency task file
#    Extract Status field → "Done" ✓
# 2. If any dependency not Done, block task
#    Update Status: Blocked_By_Dependency_task_XX.YY

# Agent response:
# Cannot start task: Dependency not satisfied
# Blocking dependency: task_XX.YY.ZZ (Status: InProgress_By_OtherAgent)
```

## Commit Message Format
```text
task_XX.YY.ZZ: <subject>

<body with details>
- Change 1
- Change 2
- Testing notes

Related: task_XX.YY.ZZ_description.md
Refs: #issue_number (if applicable)
```

## When Blocked
1. Update Status immediately: `Blocked_By_[Specific_Reason]`
   - Example: `Blocked_By_Database_Migration_Pending`
   - Example: `Blocked_By_Dependency_task_03.01.02`
2. Add detailed log entry explaining:
   - What was attempted
   - Why it failed/blocked
   - What is needed to unblock
3. Notify user immediately - do NOT continue work
4. Do NOT attempt workarounds without explicit user approval

## Task Discovery Process
When user says "find next task" or "what should I work on":

1. **Read Project Overview**:
   ```text
   Read: PROJECT_TRACKING/TASKS_OVERVIEW.md
   Identify current active phase and progress
   ```

2. **Search for Available Tasks**:
   ```text
   Search pattern: **Status:** Todo
   Within: PROJECT_TRACKING/V1_MVP/[current_phase]/
   Filter: Assignee field is empty OR not assigned
   ```

3. **Verify Dependencies**:
   ```text
   For each candidate task:
   - Read Dependencies section
   - Check each dependency's Status
   - Only propose if ALL dependencies are Done
   ```

4. **Propose Task**:
   ```text
   Format:
   "Found available task: task_XX.YY.ZZ_description.md
   - Priority: [High/Medium/Low]
   - Module: [Module name]
   - Dependencies: All satisfied ✓
   - Estimated complexity: [assessment]

   Shall I claim this task?"
   ```

## AI Agent Log Entry Format
```markdown
## AI Agent Log:
---
*   YYYY-MM-DD HH:MM: [Action] by [Agent Name]
    - Detail 1
    - Detail 2
    - Status: [result/outcome]
    - Files modified: [list if applicable]
```

**Examples**:
```markdown
*   2025-11-05 10:30: Task claimed by Grok
    - Verified dependencies: all Done
    - Starting work on sub-task 1

*   2025-11-05 11:00: Completed sub-task 1 by Grok
    - Created user registration endpoint
    - Added validation logic
    - Files: services/user_service/api/handlers.rs
    - Status: Tests passing ✓

*   2025-11-05 14:20: Encountered blocker by Grok
    - Database migration conflict
    - Status: Blocked_By_Migration_Conflict
    - Notified user for resolution
```

## Task File Metadata and Structure
```markdown
# Task: [Task Title]

**Task ID:** V1_MVP/XX.YY.ZZ_task_name.md
**Version:** V1_MVP
**Phase:** XX_Phase_Name
**Module:** XX.YY_Module_Name
**Priority:** High/Medium/Low
**Status:** [Valid Status]
**Assignee:** [Agent Name or empty]
**Created Date:** YYYY-MM-DD
**Last Updated:** YYYY-MM-DD

## Detailed Description
[Task description in English]

## Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2

## Specific Sub-tasks
- [ ] 1. Sub-task description
  - [ ] 1.1. Sub-sub-task
- [ ] 2. Another sub-task

## Dependencies
- Task: `task_XX.YY.ZZ_dependency.md` (Status: Done)

## Files to Create/Modify
[List files]

## Code Examples
[If applicable]

## Testing Steps
[If applicable]

## References
[Links/docs]

## AI Agent Log
[Log entries]
```

## Sub-task Progress Tracking
```markdown
# Initial state:
- [ ] 1. Main task
  - [ ] 1.1. Sub-task

# After completion:
- [x] 1. Main task
  - [x] 1.1. Sub-task
```

## Quality Checklist Before NeedsReview
- [ ] All sub-tasks have `[x]` checkboxes
- [ ] All acceptance criteria are met
- [ ] Code compiles: `cargo check --workspace` passing
- [ ] Tests run (if applicable): `cargo test` passing
- [ ] Git commits reference task ID
- [ ] AI Agent Log has detailed entries for all work
- [ ] No uncommitted changes or incomplete work
- [ ] Status updated to `NeedsReview`

## Common Pitfalls to AVOID
❌ Using custom status like "Completed_Pending_Database"
❌ Skipping dependency verification
❌ Committing code before updating task file
❌ Vague log entries like "fixed stuff"
❌ Forgetting to mark sub-task checkboxes
❌ Not updating Last Updated date
❌ Claiming tasks assigned to others
❌ Marking task Done without user review

## Task Assignment Commands
```bash
# Direct assignment by user
# "Grok, please work on task V1_MVP/XX.YY.ZZ_task_name.md"

# Autonomous task discovery
# "Grok, find a Todo task in phase XX of version V1_MVP"
```

## Commit and Push Workflow
```bash
git add .
git commit -m "task_XX.YY.ZZ: Complete implementation

Detailed description of changes:
- Change 1
- Change 2
- Testing completed

All acceptance criteria met. Ready for review."
git push origin feature/task_XX.YY.ZZ
```

## Create Pull Request
```shell
gh pr create \
  --title "task_XX.YY.ZZ: [Task Title]" \
  --body "Implements [task description]

## Changes
- [List changes]

## Testing
- [Testing status]

Closes task V1_MVP/XX.YY.ZZ_task_name.md"
```

## Success Criteria
A task is ready for NeedsReview when:
✅ All checkboxes marked
✅ All acceptance criteria met
✅ Code compiles and tests pass
✅ Git history is clean with proper commits
✅ AI Agent Log tells complete story
✅ No known issues or TODOs left
✅ Documentation updated
✅ Ready for user to review and test

## Updated from Context7 (/tymon3568/folder-tasks)
- Latest workflow rules as of 2025-11-05
- Agent naming: Use actual agent name (e.g., Grok) instead of placeholders
- Enhanced dependency verification logic
- Improved commit message standards
- Added PR creation workflow
- Task file structure standardization</content>
<parameter name="filePath">/home/arch/Project/test/anthill/docs/FOLDER_TASKS_WORKFLOW_UPDATED.md
