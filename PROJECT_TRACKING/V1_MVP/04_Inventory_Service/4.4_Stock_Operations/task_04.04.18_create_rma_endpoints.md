# Task: Create RMA Endpoints

**Task ID:** V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.18_create_rma_endpoints.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.4_Stock_Operations
**Priority:** Medium
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Implement the API endpoints for the RMA (Returned Merchandise Authorization) workflow.

## Specific Sub-tasks:
- [ ] 1. Implement `POST /api/v1/inventory/rma` to create a new RMA request.
- [ ] 2. Implement `POST /api/v1/inventory/rma/:id/approve` to approve or reject an RMA request.
- [ ] 3. Implement `POST /api/v1/inventory/rma/:id/receive` to process the receipt of returned goods.
- [ ] 4. The `receive` endpoint should create a `stock_move` from the `Customer` virtual location back into a `Quarantine` or main warehouse.

## Acceptance Criteria:
- [ ] All three endpoints are implemented and authorized.
- [ ] The endpoints correctly manage the RMA status and trigger the appropriate stock movements.
- [ ] The workflow is covered by integration tests.

## Dependencies:
*   Task: `task_04.04.17_create_rma_tables.md`

## Related Documents:
*   `inventory_service/api/src/handlers.rs`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
