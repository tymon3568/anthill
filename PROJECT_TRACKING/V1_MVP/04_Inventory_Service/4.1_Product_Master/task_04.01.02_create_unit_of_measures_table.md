# Task: Create `unit_of_measures` Table

**Task ID:** V1_MVP/04_Inventory_Service/4.1_Product_Master/task_04.01.02_create_unit_of_measures_table.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.1_Product_Master
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Create the `unit_of_measures` (UoM) table to define all possible units for products (e.g., Piece, Box, Kg, Liter).

## Specific Sub-tasks:
- [ ] 1. Create a new SQL migration file for the `unit_of_measures` table.
- [ ] 2. Define all columns as specified: `uom_id`, `tenant_id`, `name`, `uom_type`, `category`, `rounding_precision`.
- [ ] 3. Add a foreign key for `tenant_id`.

## Acceptance Criteria:
- [ ] A new SQL migration is created for the `unit_of_measures` table.
- [ ] The table schema includes all specified fields.
- [ ] The migration runs successfully.

## Dependencies:
*   (Depends on Phase 2 Database Setup)

## Related Documents:
*   `docs/database-erd.dbml`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)