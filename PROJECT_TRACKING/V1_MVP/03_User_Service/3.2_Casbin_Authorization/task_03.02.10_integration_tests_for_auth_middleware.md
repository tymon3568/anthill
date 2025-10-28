# Task: Integration Tests for Authorization Middleware

**Task ID:** V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.10_integration_tests_for_auth_middleware.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.2_Casbin_Authorization
**Priority:** High
**Status:** InProgress_By_Gemini
**Assignee:** Gemini
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-27

## Detailed Description:
Write integration tests for the authorization middleware. These tests will require a running instance of the service or a test server to hit the actual endpoints.

## Specific Sub-tasks:
- [ ] 1. Write a test to ensure an admin user can access admin-only endpoints.
- [ ] 2. Write a test to ensure a manager user is denied access to admin-only endpoints.
- [ ] 3. Write a test to ensure a basic user can only access their permitted read-only endpoints.
- [ ] 4. **Crucially**, write a test to simulate a user from `tenant_a` trying to access resources belonging to `tenant_b` and assert that they receive a 403 or 404 error.

## Acceptance Criteria:
- [ ] Integration tests are added to the `user_service_api` crate.
- [ ] Tests cover all roles and permission levels.
- [ ] A specific test for tenant isolation at the HTTP request level is implemented and passes.
- [ ] All tests pass successfully.

## Dependencies:
*   Task: `task_03.02.05_implement_axum_authorization_middleware.md`
*   Task: `task_03.02.07_seed_default_roles_and_policies.md`

## Related Documents:
*   `user_service/api/tests/`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá-trình thực hiện)

## AI Agent Log:
---
*   2025-10-27: Task claimed by Gemini.