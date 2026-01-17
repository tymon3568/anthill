# Task: Implement Admin Role & Permission Management UI

**Task ID:** V1_MVP/08_Frontend/8.8_Admin_Console/task_08.08.02_admin_role_management.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.8_Admin_Console
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-01-26
**Last Updated:** 2025-01-26

## Detailed Description:
Create an interface for managing custom roles and permissions. This is a critical multi-tenant feature allowing admins to define granular access controls for their organization.

## Specific Sub-tasks:
- [ ] 1. Create Role List View (`src/routes/(app)/admin/roles/+page.svelte`)
    - [ ] Fetch roles from `GET /api/v1/admin/roles`
    - [ ] Display roles, user counts, and basic details
- [ ] 2. Create "Role Editor" Interface
    - [ ] Create/Edit Form: Role Name (create only), Description
    - [ ] Permission Matrix Component:
        - [ ] Fetch available permissions from `GET /api/v1/admin/permissions`
        - [ ] Render grouped by Resource (Users, Products, etc.)
        - [ ] Checkboxes for each Action (Read, Write, Delete, etc.)
    - [ ] API Integration:
        - [ ] Create: `POST /api/v1/admin/roles`
        - [ ] Update: `PUT /api/v1/admin/roles/{role_name}`
- [ ] 3. Implement Role Deletion
    - [ ] API: `DELETE /api/v1/admin/roles/{role_name}`
    - [ ] Protection: Prevent deleting system roles or roles with assigned users (handle 409 Conflict)

## Acceptance Criteria:
- [ ] Admin can view all roles and their descriptions.
- [ ] Admin can create a new custom role with specific permissions.
- [ ] Permission selection is intuitive (grouped by resource).
- [ ] Admin can update permissions for existing custom roles.
- [ ] System roles (admin, user) are read-only / protected.
- [ ] Role deletion is handled safely with proper error messages.

## Dependencies:
- V1_MVP/08_Frontend/8.8_Admin_Console/task_08.08.01_admin_user_management.md

## Related Documents:
- `services/user_service/ADMIN_ROLE_API.md`

## Notes / Discussion:
---
* The Permission Matrix should be reusable if possible.
* Visual distinction between System Roles and Custom Roles is important.

## AI Agent Log:
---
