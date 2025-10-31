# Task: Implement Tenant Resolution on Login

**Task ID:** V1_MVP/03_User_Service/3.1_Core_Authentication/task_03.01.02_implement_tenant_resolution_on_login.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.1_Core_Authentication
**Priority:** Medium
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Currently, the registration endpoint creates a new tenant for the first user. The login endpoint needs to be updated to handle existing users and resolve their tenant correctly. This is a crucial step for supporting multiple users within the same tenant.

## Specific Sub-tasks:
- [ ] 1. Modify the login handler in `user_service_api`.
- [ ] 2. In the handler, after verifying the user's password, query the database to find the `tenant_id` associated with the user.
- [ ] 3. Ensure the `tenant_id` is included in the JWT claims when the token is generated.
- [ ] 4. Write an integration test where a second user registers (or is invited to) an existing tenant and can successfully log in.

## Acceptance Criteria:
- [ ] The login handler is updated to find the user's tenant.
- [ ] The JWT generated upon login contains the correct `tenant_id` for the existing tenant.
- [ ] A user can successfully log in to an existing tenant.
- [ ] An integration test is created to verify that two users from the same tenant can log in and receive JWTs with the same `tenant_id`.

## Dependencies:
*   (Sẽ được cập nhật khi cần)

## Related Documents:
*   `user_service/api/src/handlers.rs`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
