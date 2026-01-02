# Task: Reconcile Invalid Task Status Values (Folder-Tasks Compliance)

**Task ID:** `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.00_reconcile_invalid_task_statuses.md`  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.5_Authz_Versioning  
**Priority:** High  
**Status:** Todo  
**Assignee:**  
**Created Date:** 2026-01-02  
**Last Updated:** 2026-01-02  

## Detailed Description
Some task files under `PROJECT_TRACKING/` currently use non-compliant status values (e.g., `InProgress` without agent suffix). The repository’s folder-tasks system only allows exactly these statuses:

- `Todo`
- `InProgress_By_[AgentName]`
- `Blocked_By_[Reason]`
- `NeedsReview`
- `Done`

This task standardizes all invalid statuses to compliant values, ensuring multi-agent coordination safety and preventing automation/parsing failures.

## Scope
- ✅ Included:
  - Identify task files with invalid `Status:` values under `V1_MVP/03_User_Service/**`.
  - Propose/perform safe status normalization:
    - If `Assignee` clearly indicates a single owner → convert to `InProgress_By_[Assignee]`.
    - If `Assignee` is empty/unclear → convert to `Todo` (or `Blocked_By_[Reason]` if clearly blocked).
  - Fix any other format issues that could break task parsers (missing Task ID, missing Status line, malformed header).
  - Add an audit log entry in each modified task file’s `AI Agent Log`.

- ❌ Excluded:
  - Any product/feature implementation work.
  - Changing priorities, dependencies, or scopes beyond what is needed for status compliance.
  - Editing tasks that are currently owned by another agent unless explicitly reassigned.

## Specific Sub-tasks
- [ ] 1. Scan `V1_MVP/03_User_Service/**/task_*.md` for invalid `Status:` values.
- [ ] 2. Create a list of offending files and their current `Status:` + `Assignee:`.
- [ ] 3. For each offending task:
  - [ ] 3.1. Determine safe target status per rules:
    - `InProgress_By_[AgentName]` if clearly owned and actively worked
    - otherwise `Todo` (default)
    - or `Blocked_By_[Reason]` if explicitly blocked
  - [ ] 3.2. Update `Status:` line to compliant value.
  - [ ] 3.3. Update `Last Updated` to current date.
  - [ ] 3.4. Add/update `AI Agent Log` entry describing what changed and why.
- [ ] 4. Run a second pass to ensure no invalid statuses remain in `03_User_Service`.
- [ ] 5. Document the final list of changed files in this task’s `Notes / Discussion`.

## Acceptance Criteria
- [ ] No tasks under `V1_MVP/03_User_Service/**` contain invalid status values.
- [ ] Each modified task file contains an `AI Agent Log` entry explaining the status correction.
- [ ] No ownership conflicts introduced (do not claim work owned by another agent).

## Dependencies
- None

## Notes / Discussion
- Observed example of invalid status (for correction during execution):
  - `V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.08_create_role_management_apis.md` currently shows `Status: InProgress` (invalid; must be `InProgress_By_[AgentName]`).
- This task must be completed before adding new tasks or automation that relies on strict parsing of `Status:` values.

## AI Agent Log
---
* 2026-01-02: Task created to enforce folder-tasks status compliance across `03_User_Service`.
