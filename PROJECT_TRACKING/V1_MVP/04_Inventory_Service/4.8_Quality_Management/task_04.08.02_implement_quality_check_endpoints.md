# Task: Implement Quality Check API Endpoints

**Task ID:** V1_MVP/04_Inventory_Service/4.8_Quality_Management/task_04.08.02_implement_quality_check_endpoints.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.8_Quality_Management
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-29
**Last Updated:** 2025-10-29

## Detailed Description:
Implement REST API endpoints for quality management operations including creating quality checks, submitting results, and managing quality alerts. This enables integration with inventory operations like goods receipts and deliveries.

## Specific Sub-tasks:
- [ ] 1. Create quality control points CRUD endpoints:
   - `GET/POST/PUT/DELETE /api/v1/quality/points` - Manage QC points
   - Support filtering by type (incoming/outgoing/internal), product, warehouse
- [ ] 2. Create quality checks CRUD endpoints:
   - `GET/POST /api/v1/quality/checks` - List and create QC checks
   - `GET/PUT/DELETE /api/v1/quality/checks/{id}` - Manage individual checks
   - `POST /api/v1/quality/checks/{id}/submit` - Submit QC results
- [ ] 3. Create quality alerts endpoints:
   - `GET/POST /api/v1/quality/alerts` - List and create alerts
   - `PUT /api/v1/quality/alerts/{id}/resolve` - Resolve alerts
- [ ] 4. Implement business logic for QC workflow:
   - Auto-create QC checks when goods receipts require inspection
   - Block stock movements until QC passes
   - Create alerts for failed inspections
- [ ] 5. Add OpenAPI documentation and validation
- [ ] 6. Implement proper error handling and tenant isolation

## Acceptance Criteria:
- [ ] All CRUD endpoints implemented with proper HTTP status codes
- [ ] Quality checks auto-created for applicable inventory transactions
- [ ] Stock movements blocked until QC approval
- [ ] Quality alerts generated for failures
- [ ] OpenAPI specs generated and validated
- [ ] Integration tests pass for all endpoints
- [ ] Multi-tenant isolation enforced

## Dependencies:
*   Task: `task_04.08.01_create_quality_control_tables.md`
*   Task: `task_04.04.01_create_goods_receipts_table.md`
*   Task: `task_04.04.05_create_delivery_orders_table.md`

## Related Documents:
*   `ARCHITECTURE.md` - API design patterns
*   `docs/database-erd.dbml` - Quality tables schema

## Notes / Discussion:
---
*   Integrates with GRN workflow - items requiring QC go to quarantine location
*   Supports different QC types: pass/fail, measurements, picture verification
*   Quality alerts notify relevant personnel of issues
*   Follows Anthill's 3-crate pattern (api/core/infra)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
