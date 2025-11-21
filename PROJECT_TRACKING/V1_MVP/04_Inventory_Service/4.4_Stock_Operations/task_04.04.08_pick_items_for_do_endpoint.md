# Task: Pick Items for DO Endpoint

**Task ID:** V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.08_pick_items_for_do_endpoint.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.4_Stock_Operations
**Priority:** High
**Status:** NeedsReview
**Assignee:** Claude
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-29

## Detailed Description:
Implement the endpoint for warehouse staff to pick items for a Delivery Order.

## Specific Sub-tasks:
- [x] 1. Implement the handler for `POST /api/v1/inventory/deliveries/:id/pick`.
- [x] 2. Update the `picked_quantity` on the relevant `delivery_order_items`.
- [x] 3. Update the main DO status to `picked`.
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
*   2025-10-29 10:00: Task claimed by Claude
    - Verified dependencies: task_04.04.07_create_do_from_order_endpoint.md is completed
    - Starting work on implementing POST /api/v1/inventory/deliveries/:id/pick endpoint
    - Will update picked_quantity and DO status

*   2025-10-29 11:00: Implementation completed by Claude
    - Created delivery service trait and implementation
    - Added Picked status to DeliveryOrderStatus enum
    - Implemented POST /api/v1/inventory/deliveries/:id/pick handler
    - Added validation for quantities and status transitions
    - Updated picked_quantity for delivery items and set DO status to Picked
    - Added proper error handling and authorization
    - Ready for review
