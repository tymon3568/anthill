# Task: Enable Lot/Serial Tracking per Product

**Task ID:** V1_MVP/04_Inventory_Service/4.5_Lot_Serial_Tracking/task_04.05.02_enable_lot_serial_tracking_per_product.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.5_Lot_Serial_Tracking
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Modify the `products` table to add a `tracking_method` field. This will allow enabling lot or serial number tracking on a per-product basis.

## Specific Sub-tasks:
- [ ] 1. Create a new SQL migration to add a `tracking_method` column (e.g., with values `none`, `lot`, `serial`) to the `products` table.
- [ ] 2. Update the business logic for Goods Receipt to enforce lot/serial number assignment for tracked products.

## Acceptance Criteria:
- [ ] A new SQL migration is created to add the `tracking_method` column to the `products` table.
- [ ] The GRN process is updated to require lot/serial numbers for tracked products.
- [ ] The migration runs successfully.

## Dependencies:
*   Task: `task_04.01.01_create_products_table.md`
*   Task: `task_04.05.01_create_lots_serial_numbers_table.md`

## Related Documents:
*   `docs/database-erd.dbml`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)