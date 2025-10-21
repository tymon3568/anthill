# Task: Create `product_variants` Table

**Task ID:** V1_MVP/04_Inventory_Service/4.1_Product_Master/task_04.01.04_create_product_variants_table.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.1_Product_Master
**Priority:** Medium
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Create the `product_variants` table to support products that have variations, such as color or size. Each variant will be a distinct record linked to a parent product and can have its own SKU, barcode, and inventory level.

## Specific Sub-tasks:
- [ ] 1. Create a new SQL migration file.
- [ ] 2. Define the `product_variants` table with columns: `variant_id`, `parent_product_id`, `tenant_id`, `variant_attributes` (JSONB), `sku`, `barcode`, `price_difference`.
- [ ] 3. Add a foreign key constraint from `parent_product_id` to the `products` table.
- [ ] 4. Ensure the `sku` is unique per tenant.

## Acceptance Criteria:
- [ ] A new SQL migration is created for the `product_variants` table.
- [ ] The table schema is implemented as specified.
- [ ] The migration runs successfully.

## Dependencies:
*   Task: `task_04.01.01_create_products_table.md`

## Related Documents:
*   `docs/database-erd.dbml`

## Notes / Discussion:
---
*   The `variant_attributes` JSONB field will store key-value pairs like `{"color": "red", "size": "L"}`.

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
