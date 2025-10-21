# Task: Pick Items for DO Endpoint

**Task ID:** V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.08_pick_items_for_do_endpoint.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.4_Stock_Operations
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Implement the endpoint for warehouse staff to pick items for a Delivery Order.

## Specific Sub-tasks:
- [ ] 1. Implement the handler for `POST /api/v1/inventory/deliveries/:id/pick`.
- [ ] 2. Update the `picked_quantity` on the relevant `delivery_order_items`.
- [ ] 3. Update the main DO status to `picked`.
- [ ] 4. (Optional) Implement logic to generate a `pick_list` document.

## Acceptance Criteria:
- [ ] The `POST /api/v1/inventory/deliveries/:id/pick` endpoint is implemented.
- [ ] The endpoint updates the quantities and status as specified.
- [ ] The action is protected by authorization.
- [ ] An integration test verifies the picking process.

## Dependencies:
*   Task: `task_04.04.07_create_do_from_order_endpoint.md`

## Related Documents:
*   `inventory_service/api/src/handlers.rs`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)