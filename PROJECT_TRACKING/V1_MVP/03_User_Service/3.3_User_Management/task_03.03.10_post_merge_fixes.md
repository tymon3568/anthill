# Task: Post-Merge Fixes for PR #137 Remaining Issues

**Task ID:** V1_MVP/03_User_Service/3.3_User_Management/task_03.03.10_post_merge_fixes.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.3_User_Management
**Priority:** High
**Status:** Done
**Assignee:** Antigravity
**Created Date:** 2026-01-07
**Last Updated:** 2026-01-07
**Related PR:** https://github.com/tymon3568/anthill/pull/137

## Description
Address remaining unresolved issues from PR #137 (Feature/03.03.04 user invitation system) that were identified in the latest CodeRabbit review after the PR was merged. These issues were not fixed before merge and require post-merge attention.

## Completion Criteria
- [x] All critical issues are fixed
- [x] All warning-level issues are fixed or documented
- [x] cargo check --workspace passes
- [x] cargo clippy --workspace passes
- [x] cargo fmt --check passes
- [x] Changes committed and pushed to main branch

## Issues to Fix

### Critical Issues

- [x] **N+1 query in list_invitations handler** (Severity: Major, Reviewer: CodeRabbit)
  - File: `services/user_service/api/src/invitation_handlers.rs` (Lines 224-250)
  - Issue: Loop fetches user information for each invitation individually, causing N+1 DB queries.
  - Fix: Batch-fetch all inviter users in a single query, build a HashMap, then populate responses using the map.

- [x] **Remove multiplication by 3600 in lib.rs** (Severity: Critical, Reviewer: CodeRabbit)
  - File: `services/user_service/api/src/lib.rs` (Line 61)
  - Issue: `config.invitation_expiry_hours * 3600` causes invitations to expire in ~19.7 years instead of configured hours.
  - Fix: Remove the `* 3600` multiplication since `Duration::hours()` expects hours.

- [x] **Remove Debug derive from Invitation struct** (Severity: Major, Reviewer: CodeRabbit)
  - File: `services/user_service/core/src/domains/auth/domain/model.rs` (Line 150)
  - Issue: `Debug` derive can leak sensitive fields (token_hash, email, IP addresses) in logs.
  - Fix: Remove `Debug` from derive list or implement custom `Debug` that redacts sensitive fields.

### Warning Issues

- [x] **TOCTOU race condition on attempt limit check** (Severity: Major, Reviewer: CodeRabbit)
  - File: `services/user_service/infra/src/auth/invitation_service.rs` (Lines 152-160)
  - Issue: Check-then-increment pattern allows concurrent requests to bypass attempt limits.
  - Fix: Implement atomic repository method that checks and increments in single SQL statement.

- [x] **Casbin role assignment failure handling** (Severity: Major, Reviewer: CodeRabbit)
  - File: `services/user_service/infra/src/auth/invitation_service.rs` (Lines 237-253)
  - Issue: Silently logging role assignment failures leaves users without permissions.
  - Fix: Either propagate error to fail operation or implement retry/compensation logic.

## AI Agent Log:
---
* 2026-01-07 19:00: Task created by Antigravity
    - Identified remaining unresolved issues from latest CodeRabbit review on merged PR #137
    - Categorized 5 issues: 3 critical, 2 warning
    - Status: Todo

* 2026-01-07 20:00: All post-merge fixes completed by Antigravity
    - Fixed N+1 query by adding find_by_ids to UserRepository and using batch lookup in list_invitations
    - Removed multiplication by 3600 in invitation service initialization
    - Removed Debug derive from Invitation struct to prevent sensitive data leakage
    - Implemented atomic check_and_increment_accept_attempts to eliminate TOCTOU race condition
    - Made Casbin role assignment failure fatal to prevent inconsistent user state
    - All cargo quality checks pass (check, clippy, fmt)
    - Status: Done
