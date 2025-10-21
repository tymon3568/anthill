# Task: Seed Default Roles and Policies

**Task ID:** V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.07_seed_default_roles_and_policies.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.2_Casbin_Authorization
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Create a SQL migration or a seed script to populate the `casbin_rule` table with default roles and policies that will be applied to new tenants.

## Specific Sub-tasks:
- [ ] 1. Create a new migration file (e.g., `seed_default_casbin_policies`).
- [ ] 2. Add `INSERT` statements for `p` (policy) rules for the `admin`, `manager`, and `user` roles.
- [ ] 3. Define permissions for each role (e.g., admin has all permissions, manager has CRUD on business objects, user has read-only).
- [ ] 4. Add example `INSERT` statements for `g` (grouping) policies to show how to assign a user to a role.

## Acceptance Criteria:
- [ ] A new SQL migration file is created.
- [ ] The script inserts `p` (policy) rules for `admin`, `manager`, and `user` roles.
- [ ] The script includes examples of `g` (grouping) rules to assign a user to a role.
- [ ] The default policies are applied correctly when a new tenant is created.

## Dependencies:
*   Task: `task_03.02.03_create_casbin_rule_table_migration.md`

## Related Documents:
*   `migrations/`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)