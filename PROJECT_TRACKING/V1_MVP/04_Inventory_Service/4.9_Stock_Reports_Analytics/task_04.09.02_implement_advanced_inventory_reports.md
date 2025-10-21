# Task: Implement Advanced Inventory Reports

**Task ID:** V1_MVP/04_Inventory_Service/4.9_Stock_Reports_Analytics/task_04.09.02_implement_advanced_inventory_reports.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.9_Stock_Reports_Analytics
**Priority:** Medium
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Implement a set of advanced inventory reports to provide business insights.

## Specific Sub-tasks:
- [ ] 1. **Stock Aging Report**: `GET /api/v1/inventory/reports/aging`
    - Classify stock into aging buckets (e.g., 0-30, 31-60, 61-90, >90 days).
    - Helps identify slow-moving and dead stock.
- [ ] 2. **Inventory Turnover Report**: `GET /api/v1/inventory/reports/turnover`
    - Calculate `COGS / Average Inventory Value`.
    - Provides a key metric for inventory performance.
- [ ] 3. **Low Stock Report**: `GET /api/v1/inventory/reports/low-stock`
    - List all products currently below their defined reorder point.
- [ ] 4. **Dead Stock Report**: `GET /api/v1/inventory/reports/dead-stock`
    - List products that have had no outbound movement for a specified period (e.g., >90 days).

## Acceptance Criteria:
- [ ] All specified report endpoints are implemented.
- [ ] Each report provides accurate data based on stock history.
- [ ] The reports are reasonably performant.

## Dependencies:
*   Task: `task_04.03.01_create_stock_moves_table.md`

## Related Documents:
*   `inventory_service/api/src/handlers.rs`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
