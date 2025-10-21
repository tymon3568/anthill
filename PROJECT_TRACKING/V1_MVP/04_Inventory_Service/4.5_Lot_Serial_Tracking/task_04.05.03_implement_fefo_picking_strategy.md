# Task: Implement FEFO Picking Strategy

**Task ID:** V1_MVP/04_Inventory_Service/4.5_Lot_Serial_Tracking/task_04.05.03_implement_fefo_picking_strategy.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.5_Lot_Serial_Tracking
**Priority:** Medium
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Implement a FEFO (First Expiry, First Out) picking strategy. When creating a delivery order or pick list, the system should automatically suggest or allocate stock from the lots with the nearest expiry date first.

## Specific Sub-tasks:
- [ ] 1. Modify the delivery order creation logic.
- [ ] 2. When reserving stock for lot-tracked items, query the `lots_serial_numbers` table to find available lots, ordered by `expiry_date` ascending.
- [ ] 3. The system should raise a warning or prevent picking from an already expired lot.
- [ ] 4. (Optional) Implement a background job to automatically move expired items to a `Quarantine` location.

## Acceptance Criteria:
- [ ] The picking logic correctly prioritizes lots with the earliest expiry dates.
- [ ] The system prevents the use of expired stock.
- [ ] Tests are written to verify the FEFO logic.

## Dependencies:
*   Task: `task_04.05.01_create_lots_serial_numbers_table.md`

## Related Documents:
*   `inventory_service/core/src/domains/picking.rs` (or similar)

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
