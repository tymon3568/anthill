# Task: Create `uom_conversions` Table

**Task ID:** V1_MVP/04_Inventory_Service/4.1_Product_Master/task_04.01.03_create_uom_conversions_table.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.1_Product_Master
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Create the `uom_conversions` table to manage conversion factors between different units of measure for a specific product (e.g., 1 Box = 12 Pieces).

## Specific Sub-tasks:
- [ ] 1. Create a new SQL migration file for the `uom_conversions` table.
- [ ] 2. Define all columns: `conversion_id`, `tenant_id`, `product_id`, `from_uom_id`, `to_uom_id`, `conversion_factor`.
- [ ] 3. Add foreign key constraints to `products` and `unit_of_measures` tables.

## Acceptance Criteria:
- [ ] A new SQL migration is created for the `uom_conversions` table.
- [ ] The table schema includes all specified fields and foreign key constraints.
- [ ] The migration runs successfully.

## Dependencies:
*   Task: `task_04.01.01_create_products_table.md`
*   Task: `task_04.01.02_create_unit_of_measures_table.md`

## Related Documents:
*   `docs/database-erd.dbml`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
