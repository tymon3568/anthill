# Task: Create `stock_adjustments` Table

**Task ID:** V1_MVP/04_Inventory_Service/4.3_Stock_Operations/task_04.03.02_create_stock_adjustments_table.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.3_Stock_Operations
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Create the `stock_adjustments` table to record the reasons for manual stock adjustments. Each adjustment will correspond to a `stock_move` record.

## Specific Sub-tasks:
- [ ] 1. Create a new SQL migration file for the `stock_adjustments` table.
- [ ] 2. Define all columns: `adjustment_id`, `move_id`, `tenant_id`, `product_id`, `warehouse_id`, `reason_code`, `notes`, `approved_by`.
- [ ] 3. Add a foreign key constraint from `move_id` to the `stock_moves` table.

## Acceptance Criteria:
- [ ] A new SQL migration is created for the `stock_adjustments` table.
- [ ] The table schema is implemented with a foreign key to `stock_moves`.
- [ ] The migration runs successfully.

## Dependencies:
*   Task: `task_04.03.01_create_stock_moves_table.md`

## Related Documents:
*   `docs/database-erd.dbml`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)