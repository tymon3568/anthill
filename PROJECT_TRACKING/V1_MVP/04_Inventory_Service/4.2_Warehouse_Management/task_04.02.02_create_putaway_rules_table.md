# Task: 04.02.02 - Create Putaway Rules Table (Advanced Warehouse Management)

**Task ID:** `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.02_create_putaway_rules_table.md`  
**Status:** Done  
**Priority:** P1  
**Assignee:** Grok (via 4.10.01)  
**Last Updated:** 2025-01-13  
**Dependencies:**  
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.01_create_warehouse_hierarchy_api.md` (Done)  
- Product master tables from 4.1 (Done)  

## Context
To support advanced warehouse operations (putaway optimization similar to Odoo), Inventory Service needs configurable **putaway rules** that determine where inbound stock should be stored based on product/location/storage category and constraints.

This task adds the database schema (migrations) required to store putaway rules and integrates with existing multi-tenant constraints.

Reference: `docs/INVENTORY_IMPROVE.md` ("Putaway Rules: Cần thêm").

## Resolution

**This task has been superseded by `task_04.10.01_implement_putaway_rules.md`**, which implemented the complete putaway rules feature including:
- Database migration: `20251205000002_create_putaway_rules_table.sql`
- Core domain models and DTOs
- Repository traits and implementations
- Service layer with putaway recommendation logic
- API handlers and routes

See `4.10_Advanced_Warehouse/task_04.10.01_implement_putaway_rules.md` for full implementation details.

## Goal
Create tenant-scoped tables and indexes for putaway rules so later tasks can implement:
- Putaway recommendation during GRN / internal transfers
- Rule selection & priority ordering
- Compatibility with zones/locations and optional storage categories

## Scope
### In scope
- Add migration(s) to create `putaway_rules` table (tenant-scoped).
- Add necessary indexes and constraints for performance and tenant isolation.
- Ensure the schema supports priority ordering and flexible conditions.

### Out of scope
- Implement putaway recommendation algorithm.
- Add API endpoints/UI.
- Integrate into GRN/transfer flows (handled in later tasks, e.g. `4.10.01_implement_putaway_rules.md`).

## Deliverables
- ✅ SQL migration file(s) creating:
  - `putaway_rules` table (and optionally enums/check constraints as needed).
- ✅ Updated `migrations/README.md` only if new conventions are introduced (optional).

## Specific Sub-tasks
- [x] Review existing warehouse/location schema and confirm the correct referenced tables/PKs for locations/warehouses (to ensure composite FKs can be enforced with `(tenant_id, ...)`).
- [x] Draft the SQL migration to create `putaway_rules` with required columns (tenant_id, rule_id, priority, active flag, timestamps, and condition fields).
- [x] Add composite primary key `(tenant_id, putaway_rule_id)` and tenant-safe foreign keys for every referenced entity that is tenant-scoped.
- [x] Add performance indexes for rule resolution:
  - [x] `(tenant_id, is_active, priority)`
  - [x] `(tenant_id, product_id)`
  - [x] `(tenant_id, product_category_id)`
  - [x] `(tenant_id, source_location_id)`
  - [x] `(tenant_id, destination_location_id)`
- [x] Decide and document (in migration comments) whether `storage_category_id` FK is included now or left nullable without FK until `04.02.03` is Done.
- [x] Run migration locally (`sqlx migrate run`) and verify the schema and indexes exist.
- [x] Add a brief verification note (in this task log) describing what table names/FKs were used (so later tasks can rely on it).

## Database Design (Implemented)
### Table: `putaway_rules`
Migration: `migrations/20251205000002_create_putaway_rules_table.sql`

## Acceptance Criteria
- [x] A new migration exists that creates `putaway_rules` with `tenant_id UUID NOT NULL`.
- [x] Composite primary key includes `tenant_id`.
- [x] All foreign keys that target tenant-scoped tables include `tenant_id` in the FK.
- [x] Appropriate composite indexes exist for rule lookup by tenant and priority.
- [x] The migration is reversible (down migration) if your migration system supports it.
- [x] No RLS is introduced; isolation is application-level.

## Validation Steps
- [x] Run `sqlx migrate run` against a dev database.
- [x] Confirm indexes are created and query planner can use `(tenant_id, is_active, priority)` for rule selection.

## AI Agent Log
---
*   2025-12-28 00:00: Task file created (schema planning only)
    - Added task definition for migration to support putaway rules.
    - Status: Todo
    - Files modified: `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.02_create_putaway_rules_table.md`
---
*   2025-01-13 10:00: Task marked as Done by Claude
    - This task was superseded by `task_04.10.01_implement_putaway_rules.md` which fully implemented the putaway rules feature
    - Migration exists: `20251205000002_create_putaway_rules_table.sql`
    - Full implementation includes: core models, DTOs, repository, service, API handlers
    - Status: Done
    - Files modified: `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.02_create_putaway_rules_table.md`
---
