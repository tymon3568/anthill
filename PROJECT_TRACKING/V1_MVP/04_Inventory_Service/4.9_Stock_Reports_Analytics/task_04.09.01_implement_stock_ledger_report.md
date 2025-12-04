# Task: Implement Stock Ledger Report

**Task ID:** V1_MVP/04_Inventory_Service/4.9_Stock_Reports_Analytics/task_04.09.01_implement_stock_ledger_report.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.9_Stock_Reports_Analytics
**Priority:** High
**Status:** InProgress_By_AI_Agent
**Assignee:** AI_Agent
**Created Date:** 2025-10-21
**Last Updated:** 2025-12-04

## Detailed Description:
Create the Stock Ledger Report, a critical ERP-style report that provides a complete, chronological audit trail of all movements for a product.

## Specific Sub-tasks:
- [ ] 1. Implement the handler for `GET /api/v1/inventory/reports/stock-ledger`.
- [ ] 2. Write a complex SQL query that joins `stock_moves` with product information.
- [ ] 3. Implement a running balance calculation over the query results, likely using window functions in SQL or post-processing in Rust.
- [ ] 4. Add query parameters for filtering by `product_id`, `warehouse_id`, and `date_range`.
- [ ] 5. Ensure the report is performant, even with millions of rows in `stock_moves`.

## Acceptance Criteria:
- [ ] The `GET /api/v1/inventory/reports/stock-ledger` endpoint is implemented.
- [ ] The endpoint generates an accurate, ordered list of all stock moves for a given filter.
- [ ] The running balance columns are calculated correctly.
- [ ] The report is performant.

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
    - Initiating fixes for PR #87 review issues: missing FromRow derive, warehouse balance logic, unused imports, missing ToSchema, markdown formatting.
    - Status updated to InProgress_By_AI_Agent.
