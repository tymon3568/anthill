# Task: Ship/Validate DO Endpoint

**Task ID:** V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.10_ship_do_endpoint.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.4_Stock_Operations
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Implement the final endpoint in the delivery flow to ship/validate a Delivery Order. This is the point where stock is actually deducted.

## Specific Sub-tasks:
- [ ] 1. Implement the handler for `POST /api/v1/inventory/deliveries/:id/ship`.
- [ ] 2. In a single transaction, create the immutable `stock_moves` record (from `Warehouse` to `Customer` virtual location).
- [ ] 3. Update `inventory_levels` by decrementing the stock.
- [ ] 4. Calculate and record the Cost of Goods Sold (COGS) for accounting purposes.
- [ ] 5. Publish an `inventory.delivery.completed` event.
- [ ] 6. Update the DO status to `shipped`.

## Acceptance Criteria:
- [ ] The `POST /api/v1/inventory/deliveries/:id/ship` endpoint is implemented.
- [ ] The endpoint correctly creates `stock_moves` and decrements `inventory_levels`.
- [ ] COGS is calculated and recorded.
- [ ] The `inventory.delivery.completed` event is published.
- [ ] An integration test verifies the entire process.

## Dependencies:
*   Task: `task_04.04.09_pack_items_for_do_endpoint.md`

## Related Documents:
*   `inventory_service/api/src/handlers.rs`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
