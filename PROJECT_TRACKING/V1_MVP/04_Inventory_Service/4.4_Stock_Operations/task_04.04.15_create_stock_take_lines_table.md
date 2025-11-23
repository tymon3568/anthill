# Task: Create `stock_take_lines` Table

**Task ID:** V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.15_create_stock_take_lines_table.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.4_Stock_Operations
**Priority:** High
**Status:** NeedsReview
**Assignee:** Grok
**Created Date:** 2025-10-21
**Last Updated:** 2025-11-23

## Detailed Description:
Create the `stock_take_lines` table to record the details of a stock take.

## Specific Sub-tasks:
- [x] 1. Create a new SQL migration file.
- [x] 2. Define all columns: `line_id`, `stock_take_id`, `product_id`, `expected_quantity`, `actual_quantity`, `difference_quantity`, etc.
- [x] 3. Add a foreign key constraint to the `stock_takes` table.

## Acceptance Criteria:
- [x] A new SQL migration is created for the `stock_take_lines` table.
- [x] The table schema is implemented as specified.
- [x] The migration runs successfully.

## Dependencies:
*   Task: `task_04.04.14_create_stock_takes_table.md`

## Related Documents:
*   `docs/database-erd.dbml`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   2025-11-23 01:48: Task claimed by Grok
    - Verified all dependencies are Done (task_04.04.14_create_stock_takes_table.md)
    - Updated Status to InProgress_By_Grok
    - Beginning work on creating stock_take_lines migration
*   2025-11-23 01:50: Completed migration creation by Grok
    - Created migrations/20250121000007_create_stock_take_lines_table.sql
    - Defined stock_take_lines table with UUID v7 primary key, multi-tenancy, foreign keys to stock_takes and products
    - Included expected_quantity, actual_quantity, difference_quantity, counting user and timestamp
    - Added comprehensive indexes, constraints, and documentation
    - All sub-tasks and acceptance criteria completed
    - Ready for review and testing
