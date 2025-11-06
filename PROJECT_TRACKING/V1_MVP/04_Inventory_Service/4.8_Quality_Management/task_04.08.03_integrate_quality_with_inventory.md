# Task: Integrate Quality Management with Inventory Operations

**Task ID:** V1_MVP/04_Inventory_Service/4.8_Quality_Management/task_04.08.03_integrate_quality_with_inventory.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.8_Quality_Management
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-29
**Last Updated:** 2025-10-29

## Detailed Description:
Integrate the quality management system with core inventory operations. This ensures that quality checks are automatically triggered for relevant inventory transactions and that stock movements are controlled based on quality approval status.

## Specific Sub-tasks:
- [ ] 1. Modify goods receipt creation to check for required QC:
   - Add QC requirement flag to products table
   - Auto-create QC checks when GRN contains QC-required items
   - Move stock to quarantine location until QC approval
- [ ] 2. Integrate QC with stock moves:
   - Prevent stock moves from quarantine until QC passes
   - Update stock move status based on QC results
   - Maintain audit trail of QC decisions
- [ ] 3. Implement delivery order QC integration:
   - Check outgoing QC requirements before shipping
   - Block shipments until outgoing QC passes
   - Support different QC types (incoming vs outgoing)
- [ ] 4. Add QC status tracking to inventory reports:
   - Include QC status in stock level reports
   - Show quarantined vs available stock
   - Track QC performance metrics
- [ ] 5. Implement QC workflow automation:
   - Auto-assign QC tasks based on rules
   - Send notifications for pending/failed QC
   - Escalate critical QC failures

## Acceptance Criteria:
- [ ] Goods receipts automatically trigger QC for applicable products
- [ ] Stock remains in quarantine until QC approval
- [ ] Delivery orders blocked until outgoing QC passes
- [ ] Inventory reports show QC status and quarantined stock
- [ ] QC workflow notifications work correctly
- [ ] All integrations maintain data consistency
- [ ] Performance impact is minimal

## Dependencies:
*   Task: `task_04.08.02_implement_quality_check_endpoints.md`
*   Task: `task_04.04.01_create_goods_receipts_table.md`
*   Task: `task_04.04.05_create_delivery_orders_table.md`
*   Task: `task_04.03.01_create_stock_moves_table.md`

## Related Documents:
*   `docs/database-erd.dbml` - Integration points
*   `ARCHITECTURE.md` - Event-driven integration patterns

## Notes / Discussion:
---
*   Critical integration point between quality and inventory operations
*   Ensures product quality throughout the supply chain
*   Supports both incoming and outgoing quality controls
*   Maintains stock accuracy with QC status tracking

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
