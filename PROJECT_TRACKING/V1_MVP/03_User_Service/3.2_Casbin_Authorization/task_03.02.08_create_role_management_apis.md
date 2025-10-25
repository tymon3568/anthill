# Task: Create Role Management APIs

**Task ID:** V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.08_create_role_management_apis.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.2_Casbin_Authorization
**Priority:** High
**Status:** InProgress
**Assignee:** Gemini
**Last Updated:** 2025-10-25

## Detailed Description:
Implement API endpoints for administrators to manage roles and permissions dynamically. These endpoints should only be accessible to users with an `admin` role.

## Specific Sub-tasks:
- [ ] 1. Implement `POST /api/v1/admin/roles` to create a custom role.
- [x] 2. Implement `POST /api/v1/admin/policies` to add a policy to a role.
- [x] 3. Implement `DELETE /api/v1/admin/policies` to remove a policy from a role.
- [x] 4. Implement `POST /api/v1/admin/users/:user_id/roles` to assign a role to a user.
- [x] 5. Implement `DELETE /api/v1/admin/users/:user_id/roles` to revoke a role from a user.
- [x] 6. Ensure all endpoints are protected by an admin-only authorization check.

## Acceptance Criteria:
- [ ] All five endpoints are implemented and documented in OpenAPI.
- [x] Access to these endpoints is strictly limited to users with the `admin` role.
- [x] The endpoints correctly modify the policies in the `casbin_rule` table.

## Dependencies:
*   Task: `task_03.02.06_create_axum_extractors_for_rbac.md`

## Related Documents:
*   `user_service/api/src/handlers.rs`

## Notes / Discussion:
---
*   The endpoint for creating a role (`POST /api/v1/admin/roles`) has not been implemented because a role in Casbin is implicitly created when it is used in a policy. If an explicit list of roles is needed, it should be managed in the application's own database.

## AI Agent Log:
---
* 2025-10-25 13:00: Gemini started working on the task. Researched Casbin API for policy and role management. Implemented handlers and routes for policy and user role management. Updated OpenAPI documentation.