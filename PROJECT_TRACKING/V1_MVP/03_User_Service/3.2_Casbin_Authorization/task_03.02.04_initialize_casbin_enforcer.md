# Task: Initialize Casbin Enforcer

**Task ID:** V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.04_initialize_casbin_enforcer.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.2_Casbin_Authorization
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Implement a function within the `shared/auth` crate to initialize the Casbin `Enforcer`. This function should take a database pool, create a `SqlxAdapter`, and instantiate the `Enforcer` with the `model.conf` file and the adapter.

## Specific Sub-tasks:
- [ ] 1. Create a new public async function `create_enforcer(db_pool: PgPool) -> Result<Enforcer, ...>`.
- [ ] 2. Inside the function, initialize `SqlxAdapter` with the database pool.
- [ ] 3. Initialize the `Enforcer` with the model file (`model.conf`) and the created adapter.
- [ ] 4. Return the `Ok(enforcer)`.

## Acceptance Criteria:
- [ ] A function `create_enforcer` is implemented in `shared/auth`.
- [ ] The function correctly initializes `SqlxAdapter` and `Enforcer`.
- [ ] The function is asynchronous and returns a `Result<Enforcer, ...>`.

## Dependencies:
*   Task: `task_03.02.01_add_casbin_dependencies.md`
*   Task: `task_03.02.03_create_casbin_rule_table_migration.md`

## Related Documents:
*   `shared/auth/src/lib.rs` (or similar)

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)