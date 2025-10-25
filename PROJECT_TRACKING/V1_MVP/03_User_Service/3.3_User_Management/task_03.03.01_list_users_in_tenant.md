# Task: Create Endpoint to List Users in Tenant

**Task ID:** V1_MVP/03_User_Service/3.3_User_Management/task_03.03.01_list_users_in_tenant.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.3_User_Management
**Priority:** High
**Status:** NeedsReview
**Assignee:** Cascade 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-25 18:45

## Detailed Description:
Implement an API endpoint for listing all users within the caller's tenant.

## Specific Sub-tasks:
- [x] 1. Create the handler for `GET /api/v1/users`.
- [x] 2. Add authorization to the endpoint, restricting it to `admin` or `manager` roles.
- [x] 3. Implement the repository method to query the database for users belonging to the `tenant_id` from the JWT.
- [x] 4. Add support for pagination query parameters (e.g., `page`, `per_page`).
- [x] 5. Add support for filtering by `role` and `status`.

## Acceptance Criteria:
- [x] The `GET /api/v1/users` endpoint is created.
- [x] The endpoint is protected by Casbin authorization.
- [x] The query correctly filters users by the `tenant_id` from the JWT claims.
- [x] Pagination and filtering parameters are implemented and work correctly.
- [x] The response does not include sensitive user information like password hashes.
- [ ] An integration test is written to verify functionality and tenant isolation.

## Dependencies:
*   Task: `task_03.02.05_implement_axum_authorization_middleware.md`

## Related Documents:
*   `user_service/api/src/handlers.rs`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

* 2025-10-25 18:20: Cascade started working on this task. Created branch 'feature/03.03.01-list-users-in-tenant'. Task status updated to InProgress_By_Cascade.
* 2025-10-25 18:45: Cascade completed all sub-tasks. Enhanced list_users endpoint with role and status filtering support. Updated API documentation and repository implementation. All acceptance criteria met except integration test.
* 2025-10-25 18:50: Pushed branch 'feature/03.03.01-list-users-in-tenant' to GitHub (https://github.com/tymon3568/anthill.git). Ready for code review.