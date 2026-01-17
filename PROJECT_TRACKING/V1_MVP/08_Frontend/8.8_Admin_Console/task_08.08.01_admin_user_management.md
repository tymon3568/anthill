# Task: Implement Admin User Management UI

**Task ID:** V1_MVP/08_Frontend/8.8_Admin_Console/task_08.08.01_admin_user_management.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.8_Admin_Console
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-01-26
**Last Updated:** 2025-01-26

## Detailed Description:
Create a comprehensive user management interface for Tenant Administrators. This module allows admins to list, search, create, and manage users within their tenant, including role assignments.

## Specific Sub-tasks:
- [ ] 1. Create User List View (`src/routes/(app)/admin/users/+page.svelte`)
    - [ ] Fetch users from `GET /api/v1/admin/users`
    - [ ] Implement data table with columns: Name, Email, Role, Status, Created At
    - [ ] Add pagination and search functionality
- [ ] 2. Create "Add User" Modal/Page
    - [ ] Form for Email, Password, Full Name, Initial Role
    - [ ] Validation (Password strength, Email format)
    - [ ] API integration: `POST /api/v1/admin/users`
- [ ] 3. Implement "Assign Role" functionality
    - [ ] UI to select roles for a user
    - [ ] API integration: `POST /api/v1/admin/users/{id}/roles/assign`
    - [ ] API integration: `DELETE /api/v1/admin/users/{id}/roles/{role}/remove`
- [ ] 4. User Detail View (Optional/Drawer)
    - [ ] Show user profile summary
    - [ ] List assigned roles
- [ ] 5. Implement Error Handling & Success Notifications
    - [ ] Handle 403 Forbidden (Non-admins)
    - [ ] Handle 409 Conflict (Email exists)

## Acceptance Criteria:
- [ ] Admin can see a paginated list of all users in their tenant.
- [ ] Admin can create a new user with a specific role.
- [ ] Admin can assign and remove roles for existing users.
- [ ] UI gracefully handles validation errors and API failures.
- [ ] Access is restricted to users with `admin` role (frontend guard + backend enforcement).

## Dependencies:
- V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.03_auth_api_client.md (Status: Done)
- V1_MVP/08_Frontend/8.7_Settings/task_08.07.05_user_service_api_client.md (Status: NeedsReview)
- V1_MVP/08_Frontend/8.8_Admin_Console/task_08.08.03_admin_layout_and_nav.md (Status: Todo)

## Related Documents:
- `services/user_service/ADMIN_USER_API.md`

## Notes / Discussion:
---
* Use shadcn-svelte Table and Dialog components.
* Ensure strict type safety for User and Role interfaces.
* Refer to `ADMIN_USER_API.md` for exact payload structures.

## AI Agent Log:
---