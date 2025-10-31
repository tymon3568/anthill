# Task: Create `goods_receipts` Table

**Task ID:** V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.01_create_goods_receipts_table.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.4_Stock_Operations
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Create the `goods_receipts` table to manage goods receipt notes (GRN), which are documents that record the receiving of goods into the warehouse.

## Specific Sub-tasks:
- [ ] 1. Create a new SQL migration file for the `goods_receipts` table.
- [ ] 2. Define all columns: `receipt_id`, `receipt_number`, `tenant_id`, `warehouse_id`, `supplier_id`, `status`, etc.
- [ ] 3. Design a mechanism for auto-generating the `receipt_number` (e.g., `GRN-2025-00001`).

## Acceptance Criteria:
- [ ] A new SQL migration is created for the `goods_receipts` table.
- [ ] The table schema is implemented as specified.
- [ ] An auto-incrementing or sequence-based mechanism is designed for `receipt_number`.
- [ ] The migration runs successfully.

## Dependencies:
*   (Sẽ được cập nhật khi cần)

## Related Documents:
*   `docs/database-erd.dbml`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
