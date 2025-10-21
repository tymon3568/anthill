# Task: Unit Tests for Casbin Enforcer

**Task ID:** V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.09_unit_tests_for_casbin_enforcer.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.2_Casbin_Authorization
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Write unit tests for the Casbin enforcer logic. These tests should run in-memory or against a test database and should not require a running server.

## Specific Sub-tasks:
- [ ] 1. Write a test for role assignments (`g` policies).
- [ ] 2. Write a test for permission checks (`p` policies).
- [ ] 3. Write a test to ensure a user in one tenant cannot have their permissions applied to another tenant.
- [ ] 4. Write a test to verify that an admin role grants access and a non-admin role denies it.

## Acceptance Criteria:
- [ ] Unit tests are created in the `shared/auth` crate.
- [ ] Tests cover adding policies, checking permissions, and verifying tenant isolation.
- [ ] All tests pass successfully.

## Dependencies:
*   Task: `task_03.02.04_initialize_casbin_enforcer.md`

## Related Documents:
*   `shared/auth/tests/`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)