# Task: Create Stock Take Endpoints

**Task ID:** V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.16_create_stock_take_endpoints.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.4_Stock_Operations
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Implement the API endpoints for the physical inventory counting workflow.

## Specific Sub-tasks:
- [ ] 1. Implement `POST /api/v1/inventory/stock-takes` to create a new stock take session and snapshot quantities.
- [ ] 2. Implement `POST /api/v1/inventory/stock-takes/:id/count` to submit counted quantities for items.
- [ ] 3. Implement `POST /api/v1/inventory/stock-takes/:id/finalize` to approve the count and auto-generate `stock_adjustments` for discrepancies.

## Acceptance Criteria:
- [ ] All three endpoints are implemented and authorized.
- [ ] Each endpoint performs the correct state transitions and side effects.
- [ ] The `finalize` endpoint correctly adjusts inventory levels based on the count.
- [ ] The entire workflow is covered by integration tests.

## Dependencies:
*   Task: `task_04.04.15_create_stock_take_lines_table.md`

## Related Documents:
*   `inventory_service/api/src/handlers.rs`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)