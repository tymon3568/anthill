# Task: Implement Quality Control Integration

**Task ID:** V1_MVP/04_Inventory_Service/4.6_Inventory_Valuation/task_04.06.01_implement_quality_control.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.6_Inventory_Valuation
**Priority:** Medium
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Integrate a Quality Control (QC) workflow into the goods receipt process.

## Specific Sub-tasks:
- [ ] 1. Create the `quality_checks` table with columns: `qc_id`, `tenant_id`, `receipt_id`, `product_id`, `lot_serial_id`, `status` (pending, passed, rejected), `inspector_id`, etc.
- [ ] 2. Modify the GRN process: when a GRN is created for a QC-required item, its status becomes `waiting_qc` and the stock is moved to a virtual `Quarantine` warehouse.
- [ ] 3. Implement a `POST /api/v1/inventory/quality-checks` endpoint to submit QC results.
- [ ] 4. If QC passes, create a stock move from `Quarantine` to the main warehouse.
- [ ] 5. If QC fails, the stock remains in `Quarantine` for further action (e.g., return to supplier).

## Acceptance Criteria:
- [ ] `quality_checks` table is created.
- [ ] The GRN workflow correctly handles QC items by moving them to a quarantine location.
- [ ] The QC endpoint correctly updates status and moves stock upon passing inspection.
- [ ] The process is covered by integration tests.

## Dependencies:
*   Task: `task_04.04.01_create_goods_receipts_table.md`

## Related Documents:
*   `docs/database-erd.dbml`

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
