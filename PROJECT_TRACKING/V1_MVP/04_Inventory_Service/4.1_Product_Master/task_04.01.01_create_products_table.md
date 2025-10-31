# Task: Create `products` Table (Item Master)

**Task ID:** V1_MVP/04_Inventory_Service/4.1_Product_Master/task_04.01.01_create_products_table.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.1_Product_Master
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Create the `products` table, which will serve as the Item Master - the single source of truth for all product data.

## Specific Sub-tasks:
- [ ] 1. Create a new SQL migration file for the `products` table.
- [ ] 2. Define all columns as specified in `TODO.md`: `product_id`, `tenant_id`, `sku`, `name`, `product_type`, `track_inventory`, `default_uom_id`, etc.
- [ ] 3. Ensure `sku` has a unique constraint per `tenant_id`.
- [ ] 4. Add foreign key constraints for `tenant_id`, `item_group_id`, `default_uom_id`.
- [ ] 5. Add indexes on `tenant_id`, `sku`, and `item_group_id`.

## Acceptance Criteria:
- [ ] A new SQL migration is created for the `products` table.
- [ ] The table schema includes all specified fields with correct data types and constraints.
- [ ] Indexes are created for `tenant_id`, `sku`, and `item_group_id`.
- [ ] The migration runs successfully.

## Dependencies:
*   (Depends on Phase 2 Database Setup)

## Related Documents:
*   `docs/database-erd.dbml`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
