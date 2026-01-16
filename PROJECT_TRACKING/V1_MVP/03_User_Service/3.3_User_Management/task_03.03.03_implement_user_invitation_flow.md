# Task: Implement User Invitation Flow

**Task ID:** V1_MVP/03_User_Service/3.3_User_Management/task_03.03.03_implement_user_invitation_flow.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.3_User_Management
**Priority:** Medium
**Status:** Superseded
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2026-01-16

## Note
**This task has been superseded by `task_03.03.04_create_user_invitation_system.md`** which provides a more comprehensive implementation with enhanced security requirements (token hash-at-rest, rate limiting, audit logging, etc.).

## Detailed Description:
Implement a full user invitation flow to allow existing users (e.g., admins) to invite new users to their tenant.

## Specific Sub-tasks:
- [ ] 1. Create a SQL migration for the `user_invitations` table.
- [ ] 2. Implement the `POST /api/v1/users/invite` endpoint.
    - [ ] 2.1. This endpoint should be restricted to admins.
    - [ ] 2.2. Generate a secure, random, URL-safe token.
    - [ ] 2.3. Store the hashed token, email, role, and expiry in the `user_invitations` table.
    - [ ] 2.4. Trigger an email to be sent to the user (can be a mock/log for now).
- [ ] 3. Implement the `POST /api/v1/users/accept-invite` endpoint.
    - [ ] 3.1. This endpoint takes the token and new user details (e.g., password).
    - [ ] 3.2. Validate the token (it exists, is not expired, and has not been used).
    - [ ] 3.3. Create the new user in the `users` table, linking them to the correct tenant and role.
    - [ ] 3.4. Mark the invitation token as used.

## Acceptance Criteria:
- [ ] `user_invitations` table migration is created.
- [ ] `POST /api/v1/users/invite` endpoint is implemented and sends an email (mocked in tests).
- [ ] `POST /api/v1/users/accept-invite` endpoint is implemented and correctly creates the new user.
- [ ] The entire flow is covered by integration tests.

## Dependencies:
*   Task: `task_03.03.01_list_users_in_tenant.md`

## Related Documents:
*   `user_service/api/src/handlers.rs`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
