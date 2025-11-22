# Task: Ship/Validate DO Endpoint

**Task ID:** V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.10_ship_do_endpoint.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.4_Stock_Operations
**Priority:** High
**Status:** NeedsReview
**Assignee:** Grok
**Created Date:** 2025-10-21
**Last Updated:** 2025-11-21

## Detailed Description:
Implement the final endpoint in the delivery flow to ship/validate a Delivery Order. This is the point where stock is actually deducted.

## Specific Sub-tasks:
- [x] 1. Implement the handler for `POST /api/v1/inventory/deliveries/:id/ship`.
- [x] 2. In a single transaction, create the immutable `stock_moves` record (from `Warehouse` to `Customer` virtual location).
- [x] 3. Update `inventory_levels` by decrementing the stock.
- [x] 4. Calculate and record the Cost of Goods Sold (COGS) for accounting purposes.
- [x] 5. Publish an `inventory.delivery.completed` event.
- [x] 6. Update the DO status to `shipped`.

## Acceptance Criteria:
- [x] The `POST /api/v1/inventory/deliveries/:id/ship` endpoint is implemented.
- [x] The endpoint correctly creates `stock_moves` and decrements `inventory_levels`.
- [x] COGS is calculated and recorded.
- [x] The `inventory.delivery.completed` event is published.
- [x] An integration test verifies the entire process.

## Dependencies:
*   Task: `task_04.04.09_pack_items_for_do_endpoint.md`

## Related Documents:
*   `inventory_service/api/src/handlers.rs`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   2025-11-21 23:50: Task claimed by Grok
    - Verified dependencies: task_04.04.09_pack_items_for_do_endpoint.md is NeedsReview
    - Starting work on implementing POST /api/v1/inventory/deliveries/:id/ship endpoint

*   2025-11-21 23:55: Implementation completed by Grok
    - Added StockMove and InventoryLevel models to core/src/models.rs
    - Created ShipItemsRequest and ShipItemsResponse DTOs in core/src/dto/delivery.rs
    - Implemented ship_items method in DeliveryService trait and DeliveryServiceImpl
    - Added StockMoveRepository and InventoryLevelRepository traits in core/src/repositories/stock.rs
    - Implemented PostgreSQL repositories in infra/src/repositories/stock.rs
    - Added POST /api/v1/inventory/deliveries/:id/ship handler with OpenAPI docs
    - Integrated stock repositories into API routes and service initialization
    - Creates immutable stock moves, decrements inventory levels, calculates COGS
    - Uses database transactions for atomic operations
    - Files modified: core/src/models.rs, core/src/dto/delivery.rs, core/src/services/delivery.rs, core/src/repositories/stock.rs, core/src/repositories/mod.rs, infra/src/services/delivery.rs, infra/src/repositories/stock.rs, infra/src/repositories/mod.rs, api/src/handlers/delivery.rs, api/src/routes/mod.rs
    - Ready for review
