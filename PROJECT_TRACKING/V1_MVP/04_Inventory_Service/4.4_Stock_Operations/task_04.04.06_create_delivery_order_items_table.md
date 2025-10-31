# Task: Create `delivery_order_items` Table

**Task ID:** V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.06_create_delivery_order_items_table.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.4_Stock_Operations
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Create the `delivery_order_items` table to store the line items for each Delivery Order (DO).

## Specific Sub-tasks:
- [ ] 1. Create a new SQL migration file.
- [ ] 2. Define all columns: `delivery_item_id`, `delivery_id`, `product_id`, `ordered_quantity`, `picked_quantity`, `delivered_quantity`, etc.
- [ ] 3. Add foreign key constraints to `delivery_orders` and `products`.

## Acceptance Criteria:
- [ ] A new SQL migration is created for the `delivery_order_items` table.
- [ ] The table schema is implemented with correct foreign keys.
- [ ] The migration runs successfully.

## Dependencies:
*   Task: `task_04.04.05_create_delivery_orders_table.md`

## Related Documents:
*   `docs/database-erd.dbml`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
