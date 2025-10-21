# Task: Implement Automated Stock Replenishment

**Task ID:** V1_MVP/04_Inventory_Service/4.7_Stock_Replenishment/task_04.07.01_implement_automated_replenishment.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.7_Stock_Replenishment
**Priority:** Medium
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Implement a system for automated stock replenishment based on reorder points.

## Specific Sub-tasks:
- [ ] 1. Create the `reorder_rules` table with columns: `rule_id`, `tenant_id`, `product_id`, `warehouse_id`, `reorder_point`, `min_quantity`, `max_quantity`, `lead_time_days`, `safety_stock`.
- [ ] 2. Create a background job (e.g., a cron job) that runs periodically (e.g., daily).
- [ ] 3. The job calculates the `projected_qty` (on_hand + incoming - reserved) for each product.
- [ ] 4. If `projected_qty` falls below the `reorder_point`, the system should trigger a reorder action.
- [ ] 5. The action could be creating a draft Purchase Order, a Material Request, or sending a notification to the procurement team.
- [ ] 6. Publish an `inventory.reorder.triggered` event.

## Acceptance Criteria:
- [ ] The `reorder_rules` table is created.
- [ ] A background job is implemented to check stock levels against reorder points.
- [ ] The system correctly triggers a reorder action when stock is low.
- [ ] Tests are written to verify the reorder calculation and trigger.

## Dependencies:
*   (Requires inventory levels to be tracked)

## Related Documents:
*   `docs/database-erd.dbml`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
