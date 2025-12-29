# Task: Create Removal Strategies Table (Multi-Tenant)

**Task ID:** `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.05_create_removal_strategies_table.md`  
**Status:** Done  
**Priority:** P1  
**Assignee:** Grok_SoftwareEngineer (via 4.10.03)  
**Last Updated:** 2025-01-13  
**Phase:** V1_MVP  
**Module:** 04_Inventory_Service → 4.2_Warehouse_Management  
**Dependencies:**  
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.01_create_warehouse_hierarchy_api.md` (Done)

## Context
Per `docs/INVENTORY_IMPROVE.md`, Anthill needs configurable removal strategies similar to Odoo (FIFO/LIFO/FEFO/Closest). Today the system relies mainly on FEFO logic, which is insufficient for real WMS scenarios and blocks advanced picking methods.

This task is **schema-only** (DB tables + constraints + indexes). Service/API wiring and algorithm changes should be implemented in follow-up tasks (e.g., `4.10_Advanced_Warehouse/task_04.10.03_implement_removal_strategies.md`).

## Resolution

**This task has been superseded by `task_04.10.03_implement_removal_strategies.md`**, which implemented the complete removal strategies feature including:
- Database migration: `20251208000001_create_removal_strategies_table.sql`
- Core domain models and DTOs
- Repository traits and implementations
- Service layer with removal strategy engine (FIFO, LIFO, FEFO, closest_location, least_packages)
- API handlers and routes
- Suggest removal functionality

See `4.10_Advanced_Warehouse/task_04.10.03_implement_removal_strategies.md` for full implementation details.

## Goal
Introduce tenant-scoped configuration tables that allow selecting a removal strategy at different scopes (tenant default / warehouse / location / product category / product), with clear precedence and performant lookups.

## Scope
### In scope
- ✅ Create tables:
  - `removal_strategies` (catalog/config per tenant)
  - `removal_strategy_assignments` (assignment by scope with priority)
- ✅ Enforce strict multi-tenancy:
  - every table includes `tenant_id UUID NOT NULL`
  - composite keys and indexes include `(tenant_id, ...)`
  - composite foreign keys include `tenant_id` where referencing tenant-scoped tables
- ✅ Add constraints and indexes for common resolution queries.

### Out of scope
- API endpoints and CRUD - Done in 4.10.03
- Picking algorithm implementation (FIFO/LIFO/Closest logic) - Done in 4.10.03
- UI
- Postgres RLS (we use application filtering)

## Database Design (Implemented)
### Table: `removal_strategies`
Migration: `migrations/20251208000001_create_removal_strategies_table.sql`

Implemented strategies:
- FIFO (First In First Out)
- LIFO (Last In First Out)
- FEFO (First Expired First Out)
- Closest Location
- Least Packages

## Specific Sub-tasks
- [x] Verify dependency status is `Done`
- [x] Scan existing schema to confirm correct referenced table names/PKs
- [x] Draft SQL migration(s) to create `removal_strategies`
- [x] Draft SQL migration(s) to create `removal_strategy_assignments`
- [x] Add scope+priority indexes for resolution queries
- [x] Run migrations locally and verify tables + indexes exist
- [x] Implement service layer with strategy engine
- [x] Add API handlers and routes

## Acceptance Criteria
- [x] Migration(s) exist creating `removal_strategies` and `removal_strategy_assignments`.
- [x] All tenant-scoped tables include `tenant_id UUID NOT NULL`.
- [x] All keys/indexes are tenant-aware (composite with `tenant_id`).
- [x] Composite FKs include `tenant_id` for tenant-scoped targets.
- [x] No RLS introduced.
- [x] Migration applies cleanly on a fresh/dev database.
- [x] Task uses a checkbox-based sub-task list and can be executed sequentially.

## AI Agent Log
---
* 2025-12-28 00:00: Task created (schema planning) by AI
  - Expanded task to folder-tasks Style B checklist with normalized metadata header.
  - Status: Todo
  - Files modified: `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.05_create_removal_strategies_table.md`
---
* 2025-01-13 10:00: Task marked as Done by Claude
  - This task was superseded by `task_04.10.03_implement_removal_strategies.md` which fully implemented the removal strategies feature
  - Migration exists: `20251208000001_create_removal_strategies_table.sql`
  - Full implementation includes: core models, DTOs, repository, service (FIFO/LIFO/FEFO/closest/least_packages), API handlers
  - All PR review issues from 4.10.03 were resolved (sub-tasks 1-21 completed)
  - Status: Done
  - Files modified: `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.05_create_removal_strategies_table.md`
---
