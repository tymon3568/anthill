# Task: Implement Advanced Inventory Reports

**Task ID:** V1_MVP/04_Inventory_Service/4.9_Stock_Reports_Analytics/task_04.09.02_implement_advanced_inventory_reports.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.9_Stock_Reports_Analytics
**Priority:** Medium
**Status:** NeedsReview
**Assignee:** AI_Agent
**Created Date:** 2025-10-21
**Last Updated:** 2025-12-04

## Detailed Description:
Implement a set of advanced inventory reports to provide business insights.

## Specific Sub-tasks:
- [x] 1. **Stock Aging Report**: `GET /api/v1/inventory/reports/aging`
    - Classify stock into aging buckets (e.g., 0-30, 31-60, 61-90, >90 days).
    - Helps identify slow-moving and dead stock.
- [x] 2. **Inventory Turnover Report**: `GET /api/v1/inventory/reports/turnover`
    - Calculate `COGS / Average Inventory Value`.
    - Provides a key metric for inventory performance.
- [x] 3. **Low Stock Report**: `GET /api/v1/inventory/reports/low-stock`
    - List all products currently below their defined reorder point.
- [x] 4. **Dead Stock Report**: `GET /api/v1/inventory/reports/dead-stock`
    - List products that have had no outbound movement for a specified period (e.g., >90 days).

## Acceptance Criteria:
- [x] All specified report endpoints are implemented.
- [x] Each report provides accurate data based on stock history.
- [x] The reports are reasonably performant.

## Dependencies:
*   Task: `task_04.03.01_create_stock_moves_table.md`

## Related Documents:
*   `inventory_service/api/src/handlers.rs`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   2025-12-04 12:00: [Started] by AI_Agent
    - Claimed task for implementing advanced inventory reports
    - Status updated to InProgress_By_AI_Agent
    - Will implement 4 report endpoints: aging, turnover, low-stock, dead-stock
    - Dependencies checked: stock_moves table exists

*   2025-12-04 14:00: [Completed] by AI_Agent
    - All 4 report endpoints implemented: aging, turnover, low-stock, dead-stock
    - All sub-tasks and acceptance criteria completed
    - Task status set to Done, ready for PR review
    - Files modified: handlers/reports.rs, routes/reports.rs
