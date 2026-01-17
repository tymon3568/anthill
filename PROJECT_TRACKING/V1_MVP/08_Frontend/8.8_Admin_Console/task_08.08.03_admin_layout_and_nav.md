# Task: Implement Admin Console Layout & Navigation

**Task ID:** V1_MVP/08_Frontend/8.8_Admin_Console/task_08.08.03_admin_layout_and_nav.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.8_Admin_Console
**Priority:** Medium
**Status:** Done
**Assignee:** Opus
**Created Date:** 2025-01-26
**Last Updated:** 2026-01-17

## Detailed Description:
Create the layout wrapper for the Admin Console area (`/admin`). This should likely reuse the main app layout but with a distinct sidebar or navigation section focused on administrative tasks.

## Specific Sub-tasks:
- [x] 1. Create Admin Layout (`src/routes/(protected)/admin/+layout.svelte`)
    - [x] Reuse `src/routes/(protected)/+layout.svelte` structure (nested inside protected route group).
    - [x] Ensure "Admin" section is highlighted in main nav via `adminOnly` flag in navigation config.
- [x] 2. Create Admin Sidebar/Sub-navigation
    - [x] Links to:
        - [x] Users (`/admin/users`)
        - [x] Roles (`/admin/roles`)
        - [x] Invitations (`/admin/invitations`)
- [x] 3. Implement Admin Route Guard
    - [x] `src/routes/(protected)/admin/+layout.server.ts` checks for admin role via user groups.
    - [x] Redirect non-admins to `/dashboard?error=unauthorized`.

## Acceptance Criteria:
- [x] `/admin/*` routes are protected (server-side check via groups containing 'admin' or 'owner').
- [x] Admin navigation is clear and accessible (tab-based navigation within admin area).
- [x] Seamless transition between "User App" and "Admin Console" (Admin nav item in sidebar, filtered by role).

## Dependencies:
- V1_MVP/08_Frontend/8.3_Dashboard/task_08.03.01_create_main_dashboard_layout.md

## Notes / Discussion:
---
* Ideally, the Admin Console is just a section of the main app, but visually distinct (e.g., "Admin" badge or section header).
* Implementation uses Kanidm groups for role checking (groups containing 'admin' or 'owner').
* Added 'owner' role to User type union for consistency.

## AI Agent Log:
---
*   2026-01-17 14:10: Task claimed by Opus
    - Verified dependency task_08.03.01 (dashboard layout) exists
    - No existing admin routes in codebase
    - Will create admin layout with role-based access control
    - Starting implementation of sub-task 1: Admin Layout
*   2026-01-17 14:35: Implementation completed by Opus
    - Created `/frontend/src/routes/(protected)/admin/+layout.server.ts` - Server-side admin role guard using Kanidm groups
    - Created `/frontend/src/routes/(protected)/admin/+layout.svelte` - Admin layout with tab navigation
    - Created `/frontend/src/routes/(protected)/admin/+page.server.ts` - Redirect to /admin/users
    - Created `/frontend/src/routes/(protected)/admin/users/+page.svelte` - Placeholder for user management
    - Created `/frontend/src/routes/(protected)/admin/roles/+page.svelte` - Placeholder for role management
    - Created `/frontend/src/routes/(protected)/admin/invitations/+page.svelte` - Placeholder for invitation management
    - Modified `/frontend/src/lib/config/navigation.ts` - Added Admin nav item with `adminOnly` flag
    - Modified `/frontend/src/lib/components/app-sidebar.svelte` - Filter admin-only nav items by user role
    - Added 'owner' role to User type in `/frontend/src/lib/types/index.ts`
    - Updated role type casts in auth stores and hooks
    - All typecheck passes (0 errors, 0 warnings)