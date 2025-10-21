# Task: Create `goods_receipt_items` Table

**Task ID:** V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.02_create_goods_receipt_items_table.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.4_Stock_Operations
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Create the `goods_receipt_items` table to store the line items for each Goods Receipt Note (GRN).

## Specific Sub-tasks:
- [ ] 1. Create a new SQL migration file.
- [ ] 2. Define all columns: `receipt_item_id`, `receipt_id`, `product_id`, `expected_quantity`, `received_quantity`, `unit_cost`, etc.
- [ ] 3. Add foreign key constraints to `goods_receipts` and `products`.

## Acceptance Criteria:
- [ ] A new SQL migration is created for the `goods_receipt_items` table.
- [ ] The table schema is implemented with correct foreign keys to `goods_receipts` and `products`.
- [ ] The migration runs successfully.

## Dependencies:
*   Task: `task_04.04.01_create_goods_receipts_table.md`

## Related Documents:
*   `docs/database-erd.dbml`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)