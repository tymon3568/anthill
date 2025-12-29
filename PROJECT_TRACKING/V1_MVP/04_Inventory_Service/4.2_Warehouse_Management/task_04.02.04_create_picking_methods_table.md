# Task: Create Picking Methods Table (Batch/Cluster/Wave)

**Task ID:** `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.04_create_picking_methods_table.md`  
**Status:** Done  
**Priority:** P1  
**Assignee:** Grok (via 4.10.02)  
**Last Updated:** 2025-01-13  
**Phase:** V1_MVP  
**Module:** 04_Inventory_Service → 4.2_Warehouse_Management  
**Dependencies:**  
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.01_create_warehouse_hierarchy_api.md` (Done)  
- Warehouse root representation exists (Done)  

## Context
Per `docs/INVENTORY_IMPROVE.md`, Anthill needs support for advanced picking approaches beyond FEFO-only selection. This task introduces **configuration tables** for picking methods (e.g., **batch**, **cluster**, **wave**) so later workflow endpoints (e.g., delivery order picking) can select a configured strategy per warehouse / operation type.

This task is scoped to **database schema + minimal supporting documentation** only. No API endpoints or picking execution logic belongs here.

## Resolution

**This task has been superseded by `task_04.10.02_implement_advanced_picking_methods.md`**, which implemented the complete picking methods feature including:
- Database migration: `20251206000001_create_picking_methods_table.sql`
- Core domain models and DTOs
- Repository traits and implementations
- Service layer with picking optimization logic (batch, cluster, wave, single)
- API handlers and routes for CRUD operations
- Picking plan generation and confirmation endpoints

See `4.10_Advanced_Warehouse/task_04.10.02_implement_advanced_picking_methods.md` for full implementation details.

## Goal
Add tenant-scoped tables to model picking methods and their assignment to warehouses/operation types, following Anthill multi-tenancy rules:
- every table has `tenant_id UUID NOT NULL`
- composite keys/indexes include `(tenant_id, ...)`
- timestamps use `TIMESTAMPTZ`
- no RLS; all filtering is application/repository-level

## Scope
### In scope
- ✅ SQL migration(s) creating:
  - `picking_methods` (catalog/config)
  - `warehouse_picking_method_assignments` (assignment per warehouse + operation type)
- ✅ Constraints + indexes for tenant isolation and fast lookup

### Out of scope
- Implementing picking algorithms (batch/cluster/wave execution) - Done in 4.10.02
- API CRUD endpoints for picking methods - Done in 4.10.02
- UI changes
- Consolidation of 4.3/4.4 Stock Operations modules

## Database Design (Implemented)
### Table: `picking_methods`
Migration: `migrations/20251206000001_create_picking_methods_table.sql`

## Specific Sub-tasks
- [x] Confirm how "warehouse" is represented in the current schema
- [x] Create SQL migration(s) for `picking_methods`
- [x] Create SQL migration(s) for `warehouse_picking_method_assignments`
- [x] Ensure all new tables include `tenant_id UUID NOT NULL`
- [x] Ensure composite keys include `tenant_id`
- [x] Add composite foreign keys including `tenant_id` where referencing tenant-scoped tables
- [x] Add indexes for common lookups
- [x] Run migration locally and confirm tables + indexes exist

## Acceptance Criteria
- [x] Migration applies cleanly on a fresh database.
- [x] All new tables include `tenant_id` and use composite PKs / indexes.
- [x] No cross-tenant references possible via FK design (where FKs are present).
- [x] Query patterns supported:
  - [x] fetch active picking methods for a tenant
  - [x] resolve picking method for `(tenant_id, warehouse_id, operation_type)` ordered by priority
- [x] Task file contains updated AI Agent Log entries when work is performed.

## Implementation Notes
- Implemented using `TEXT + CHECK` for `method_type`
- No RLS; tenant isolation is application-level
- IDs are UUID v7 generated in application code

## AI Agent Log
---
* 2025-12-28 00:00: Task created (planning)
  - Normalized header to folder-tasks style and expanded to Style B checklist for schema work.
  - Status: Todo
  - Files modified: `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.04_create_picking_methods_table.md`
---
* 2025-01-13 10:00: Task marked as Done by Claude
  - This task was superseded by `task_04.10.02_implement_advanced_picking_methods.md` which fully implemented the picking methods feature
  - Migration exists: `20251206000001_create_picking_methods_table.sql`
  - Full implementation includes: core models, DTOs, repository, service (batch/cluster/wave/single), API handlers
  - All 18 PR review issues from 4.10.02 were resolved
  - Status: Done
  - Files modified: `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.04_create_picking_methods_table.md`
---
