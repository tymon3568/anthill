# Task: Create Quality Control Tables

**Task ID:** V1_MVP/04_Inventory_Service/4.8_Quality_Management/task_04.08.01_create_quality_control_tables.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.8_Quality_Management
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-29
**Last Updated:** 2025-10-29

## Detailed Description:
Create the core tables for a comprehensive Quality Management System that integrates with inventory operations. This includes quality control points, quality checks, and quality alerts to ensure product quality throughout the supply chain.

## Specific Sub-tasks:
- [ ] 1. Create `quality_control_points` table with columns: `qc_point_id`, `tenant_id`, `name`, `type` (incoming, outgoing, internal), `product_id`, `warehouse_id`, `active`, `created_at`, `updated_at`
- [ ] 2. Create `quality_checks` table with columns: `qc_id`, `tenant_id`, `qc_point_id`, `reference_type`, `reference_id`, `product_id`, `lot_serial_id`, `status` (pending, passed, failed), `inspector_id`, `notes`, `created_at`, `updated_at`
- [ ] 3. Create `quality_check_lines` table with columns: `qc_line_id`, `qc_id`, `test_type` (pass_fail, measure, picture), `name`, `value`, `min_value`, `max_value`, `uom_id`, `result`, `notes`
- [ ] 4. Create `quality_alerts` table with columns: `alert_id`, `tenant_id`, `qc_id`, `title`, `description`, `priority`, `status`, `assigned_to`, `resolution`, `created_at`, `updated_at`
- [ ] 5. Add foreign key constraints and composite indexes for performance
- [ ] 6. Create database migration file

## Acceptance Criteria:
- [ ] All quality control tables are created with proper schema and constraints
- [ ] Foreign key relationships are established with products, warehouses, and users
- [ ] Critical indexes are added for query performance (tenant_id combinations)
- [ ] Migration runs successfully without errors
- [ ] Tables support multi-tenancy with tenant_id isolation

## Dependencies:
*   Task: `task_04.01.01_create_products_table.md`
*   Task: `task_04.02.01_create_warehouse_hierarchy_api.md`
*   Task: `task_04.05.01_create_lots_serial_numbers_table.md`

## Related Documents:
*   `docs/database-erd.dbml`
*   `ARCHITECTURE.md` - Multi-tenancy implementation

## Notes / Discussion:
---
*   This replaces the old quality control implementation that was misplaced in Inventory Valuation
*   Supports comprehensive QC workflow: incoming goods inspection, outgoing quality checks, internal quality audits
*   Integrates with lot/serial tracking for traceability
*   Follows Anthill's multi-tenancy patterns with tenant_id isolation

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
