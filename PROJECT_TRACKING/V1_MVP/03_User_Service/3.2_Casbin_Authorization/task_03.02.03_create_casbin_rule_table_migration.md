# Task: Create Casbin Rule Table Migration

**Task ID:** V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.03_create_casbin_rule_table_migration.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.2_Casbin_Authorization
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Create a new SQL migration file to define the `casbin_rule` table. This table will store all Casbin policies and role assignments in the PostgreSQL database.

## Specific Sub-tasks:
- [ ] 1. Generate a new migration file using `sqlx migrate add create_casbin_rule_table`.
- [ ] 2. Define the `casbin_rule` table with columns: `id`, `ptype`, `v0`, `v1`, `v2`, `v3`, `v4`, `v5`.
- [ ] 3. Add indexes on `ptype` and `v1` (tenant_id) for performance.

## Acceptance Criteria:
- [ ] A new SQL migration file is created in the `migrations/` directory.
- [ ] The migration script contains the correct SQL to create the `casbin_rule` table and its indexes.
- [ ] The migration runs successfully against the local database.

## Dependencies:
*   Task: `task_03.02.02_create_casbin_model_file.md`

## Related Documents:
*   `migrations/`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)