# Task: Create DO from Order Endpoint

**Task ID:** V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.07_create_do_from_order_endpoint.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.4_Stock_Operations
**Priority:** High
**Status:** Done
**Assignee:** Grok
**Created Date:** 2025-10-21
**Last Updated:** 2025-11-20

## Detailed Description:
Implement the mechanism to create a Delivery Order (DO) from a confirmed sales order. This is a critical link between the Order Service and Inventory Service.

## Specific Sub-tasks:
- [x] 1. Implement a NATS subscriber in the Inventory Service to listen for `order.confirmed` events.
- [x] 2. When an event is received, create a new Delivery Order record.
- [x] 3. For each item in the order, create a `delivery_order_items` record.
- [x] 4. Implement the logic to reserve stock in the `inventory_levels` table.
- [x] 5. Set the initial status of the DO to `reserved`.

## Acceptance Criteria:
- [x] The Inventory Service subscribes to the `order.confirmed` topic.
- [x] An event handler is implemented to process the event.
- [x] A new DO and its line items are created based on the order data.
- [x] The system correctly reserves stock in the `inventory_levels` table.
- [x] An integration test verifies the end-to-end flow from event to stock reservation.

## Dependencies:
*   Task: `task_04.04.06_create_delivery_order_items_table.md`

## Related Documents:
*   `inventory_service/src/consumers.rs` (or similar)

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   2025-11-20 07:00: Task claimed by Grok
    - Verified dependencies: task_04.04.06_create_delivery_order_items_table.md (Status: Done)
    - Updated Status to InProgress_By_Grok
    - Starting work on: Implement NATS subscriber for order.confirmed events
*   2025-11-20 08:00: Implementation completed by Grok
    - Created shared/events crate with NATS client and OrderConfirmed event
    - Implemented DeliveryOrder and DeliveryOrderItem models in core
    - Added repository traits and PostgreSQL implementations in infra
    - Created NATS consumer in inventory_service/api to handle order.confirmed events
    - Integrated event consumers initialization in main.rs
    - Added NATS_URL config field
    - All sub-tasks and acceptance criteria completed
    - Status: Done
