# Task: Create Role Management APIs

**Task ID:** V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.08_create_role_management_apis.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.2_Casbin_Authorization
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Implement API endpoints for administrators to manage roles and permissions dynamically. These endpoints should only be accessible to users with an `admin` role.

## Specific Sub-tasks:
- [ ] 1. Implement `POST /api/v1/admin/roles` to create a custom role.
- [ ] 2. Implement `POST /api/v1/admin/policies` to add a policy to a role.
- [ ] 3. Implement `DELETE /api/v1/admin/policies` to remove a policy from a role.
- [ ] 4. Implement `POST /api/v1/admin/users/:user_id/roles` to assign a role to a user.
- [ ] 5. Implement `DELETE /api/v1/admin/users/:user_id/roles` to revoke a role from a user.
- [ ] 6. Ensure all endpoints are protected by an admin-only authorization check.

## Acceptance Criteria:
- [ ] All five endpoints are implemented and documented in OpenAPI.
- [ ] Access to these endpoints is strictly limited to users with the `admin` role.
- [ ] The endpoints correctly modify the policies in the `casbin_rule` table.

## Dependencies:
*   Task: `task_03.02.06_create_axum_extractors_for_rbac.md`

## Related Documents:
*   `user_service/api/src/handlers.rs`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)