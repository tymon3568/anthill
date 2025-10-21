# Task: Create GRN Endpoint

**Task ID:** V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.03_create_grn_endpoint.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.4_Stock_Operations
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Implement the endpoint to create a new Goods Receipt Note (GRN).

## Specific Sub-tasks:
- [ ] 1. Implement the handler for `POST /api/v1/inventory/receipts`.
- [ ] 2. Implement idempotency checks using the `X-Idempotency-Key` header.
- [ ] 3. In a single transaction, create the `goods_receipts` and `goods_receipt_items` records.
- [ ] 4. Create a corresponding `stock_move` record from a `Supplier` virtual location to the destination warehouse.
- [ ] 5. Publish an `inventory.receipt.created` event using the outbox pattern.

## Acceptance Criteria:
- [ ] The `POST /api/v1/inventory/receipts` endpoint is implemented.
- [ ] The endpoint correctly creates records in `goods_receipts` and `goods_receipt_items`.
- [ ] Idempotency key handling is implemented.
- [ ] The endpoint is protected by authorization.
- [ ] An integration test verifies the creation and side effects (stock moves, events).

## Dependencies:
*   Task: `task_04.04.02_create_goods_receipt_items_table.md`

## Related Documents:
*   `inventory_service/api/src/handlers.rs`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)