# Task: Create `stock_take_lines` Table

**Task ID:** V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.15_create_stock_take_lines_table.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.4_Stock_Operations
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Create the `stock_take_lines` table to record the details of a stock take.

## Specific Sub-tasks:
- [ ] 1. Create a new SQL migration file.
- [ ] 2. Define all columns: `line_id`, `stock_take_id`, `product_id`, `expected_quantity`, `actual_quantity`, `difference_quantity`, etc.
- [ ] 3. Add a foreign key constraint to the `stock_takes` table.

## Acceptance Criteria:
- [ ] A new SQL migration is created for the `stock_take_lines` table.
- [ ] The table schema is implemented as specified.
- [ ] The migration runs successfully.

## Dependencies:
*   Task: `task_04.04.14_create_stock_takes_table.md`

## Related Documents:
*   `docs/database-erd.dbml`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)