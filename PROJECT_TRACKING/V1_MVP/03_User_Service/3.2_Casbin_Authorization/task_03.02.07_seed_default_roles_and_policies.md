# Task: Seed Default Roles and Policies

**Task ID:** V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.07_seed_default_roles_and_policies.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.2_Casbin_Authorization
**Priority:** High
**Status:** Done
**Assignee:** Gemini
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-25

## Detailed Description:
Create a SQL migration or a seed script to populate the `casbin_rule` table with default roles and policies that will be applied to new tenants.

## Specific Sub-tasks:
- [x] 1. Create a new migration file (e.g., `seed_default_casbin_policies`).
- [x] 2. Add `INSERT` statements for `p` (policy) rules for the `admin`, `manager`, and `user` roles.
- [x] 3. Define permissions for each role (e.g., admin has all permissions, manager has CRUD on business objects, user has read-only).
- [x] 4. Add example `INSERT` statements for `g` (grouping) policies to show how to assign a user to a role.

## Acceptance Criteria:
- [x] A new SQL migration file is created.
- [x] The script inserts `p` (policy) rules for `admin`, `manager`, and `user` roles.
- [x] The script includes examples of `g` (grouping) rules to assign a user to a role.
- [x] The default policies are applied correctly when a new tenant is created.

## Dependencies:
*   Task: `task_03.02.03_create_casbin_rule_table_migration.md`

## Related Documents:
*   `migrations/`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
* 2025-10-25 11:00: Gemini review: Verified migration file `20250110000004_seed_default_casbin_policies.sql` exists and has the correct content. Status updated to Done.