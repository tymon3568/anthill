# Task: Complete/Validate GRN Endpoint

**Task ID:** V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.04_validate_grn_endpoint.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.4_Stock_Operations
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Implement the endpoint to complete or validate a Goods Receipt Note (GRN). This action confirms the receipt of goods and makes them available in stock.

## Specific Sub-tasks:
- [ ] 1. Implement the handler for `POST /api/v1/inventory/receipts/:id/validate`.
- [ ] 2. Change the GRN status to `completed`.
- [ ] 3. Create the final, immutable `stock_moves` records.
- [ ] 4. Update inventory valuation layers (for FIFO/AVCO costing).
- [ ] 5. Publish an `inventory.receipt.completed` event to NATS.

## Acceptance Criteria:
- [ ] The `POST /api/v1/inventory/receipts/:id/validate` endpoint is implemented.
- [ ] The endpoint correctly updates the GRN status.
- [ ] It creates the final `stock_moves` and updates inventory valuation.
- [ ] An integration test verifies the validation process.

## Dependencies:
*   Task: `task_04.04.03_create_grn_endpoint.md`

## Related Documents:
*   `inventory_service/api/src/handlers.rs`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)