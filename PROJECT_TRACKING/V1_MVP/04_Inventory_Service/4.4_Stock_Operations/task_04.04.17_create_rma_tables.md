# Task: Create RMA Tables

**Task ID:** V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.17_create_rma_tables.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.4_Stock_Operations
**Priority:** Medium
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Create the necessary tables to manage Returned Merchandise Authorization (RMA) requests.

## Specific Sub-tasks:
- [ ] 1. Create a new SQL migration file.
- [ ] 2. Define the `rma_requests` table with columns: `rma_id`, `rma_number`, `tenant_id`, `customer_id`, `original_delivery_id`, `status`, `return_reason`, etc.
- [ ] 3. Define the `rma_items` table with columns: `rma_item_id`, `rma_id`, `product_id`, `quantity_returned`, `condition`, `action` (e.g., restock, scrap).
- [ ] 4. Add foreign key constraints.

## Acceptance Criteria:
- [ ] A new SQL migration is created for the `rma_requests` and `rma_items` tables.
- [ ] The table schemas are implemented as specified.
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
