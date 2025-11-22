# Task: Create Stock Transfer Endpoints

**Task ID:** V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.13_create_stock_transfer_endpoints.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.4_Stock_Operations
**Priority:** High
**Status:** InProgress_By_Grok
**Assignee:** Grok
**Created Date:** 2025-10-21
**Last Updated:** 2025-11-22

## Detailed Description:
Implement the API endpoints to manage the stock transfer workflow.

## Specific Sub-tasks:
- [ ] 1. Implement `POST /api/v1/inventory/transfers` to create a transfer in `draft` status.
- [ ] 2. Implement `POST /api/v1/inventory/transfers/:id/confirm` to move stock to an `In-Transit` location.
- [ ] 3. Implement `POST /api/v1/inventory/transfers/:id/receive` to add stock to the destination warehouse.
- [ ] 4. Ensure the process creates the correct `stock_moves` and publishes an `inventory.transfer.completed` event.

## Acceptance Criteria:
- [ ] All three endpoints are implemented and protected by authorization.
- [ ] Each endpoint correctly modifies the state of the transfer and the inventory.
- [ ] The final `receive` step creates the correct `stock_moves` and publishes an event.
- [ ] The entire flow is covered by integration tests.

## Dependencies:
*   Task: `task_04.04.12_create_stock_transfer_items_table.md`

## Related Documents:
*   `inventory_service/api/src/handlers.rs`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   2025-11-22 04:15: Task claimed by Grok
    - Verified dependency task_04.04.12_create_stock_transfer_items_table.md is Done
    - Starting work on implementing stock transfer endpoints
