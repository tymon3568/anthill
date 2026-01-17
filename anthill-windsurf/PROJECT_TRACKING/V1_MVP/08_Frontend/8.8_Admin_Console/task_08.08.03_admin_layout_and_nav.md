# Task: Implement Admin Console Layout & Navigation

**Task ID:** V1_MVP/08_Frontend/8.8_Admin_Console/task_08.08.03_admin_layout_and_nav.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.8_Admin_Console
**Priority:** Medium
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-01-26
**Last Updated:** 2025-01-26

## Detailed Description:
Create the layout wrapper for the Admin Console area (`/admin`). This should likely reuse the main app layout but with a distinct sidebar or navigation section focused on administrative tasks.

## Specific Sub-tasks:
- [ ] 1. Create Admin Layout (`src/routes/(app)/admin/+layout.svelte`)
    - [ ] Reuse `src/routes/(app)/+layout.svelte` structure or extend it.
    - [ ] Ensure "Admin" section is highlighted in main nav.
- [ ] 2. Create Admin Sidebar/Sub-navigation
    - [ ] Links to:
        - [ ] Users (`/admin/users`)
        - [ ] Roles (`/admin/roles`)
        - [ ] Tenant Settings (`/settings/tenant` - verify if this belongs here or in main settings)
- [ ] 3. Implement Admin Route Guard
    - [ ] Ensure `src/routes/(app)/admin/+layout.server.ts` checks for `admin` role.
    - [ ] Redirect non-admins to `/dashboard` or show 403.

## Acceptance Criteria:
- [ ] `/admin/*` routes are protected (server-side check).
- [ ] Admin navigation is clear and accessible.
- [ ] Seamless transition between "User App" and "Admin Console".

## Dependencies:
- V1_MVP/08_Frontend/8.3_Dashboard/task_08.03.01_create_main_dashboard_layout.md

## Notes / Discussion:
---
* Ideally, the Admin Console is just a section of the main app, but visually distinct (e.g., "Admin" badge or section header).

## AI Agent Log:
---
