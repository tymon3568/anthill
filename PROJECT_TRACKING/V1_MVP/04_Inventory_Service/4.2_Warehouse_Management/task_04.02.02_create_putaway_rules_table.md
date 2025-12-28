# Task: 04.02.02 - Create Putaway Rules Table (Advanced Warehouse Management)

**Task ID:** `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.02_create_putaway_rules_table.md`  
**Status:** Todo  
**Priority:** P1  
**Assignee:**  
**Last Updated:** 2025-12-28  
**Dependencies:**  
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.01_create_warehouse_hierarchy_api.md` (must be Done)  
- Product master tables from 4.1 (must exist for optional `product_id` targeting)  

## Context
To support advanced warehouse operations (putaway optimization similar to Odoo), Inventory Service needs configurable **putaway rules** that determine where inbound stock should be stored based on product/location/storage category and constraints.

This task adds the database schema (migrations) required to store putaway rules and integrates with existing multi-tenant constraints.

Reference: `docs/INVENTORY_IMPROVE.md` (“Putaway Rules: Cần thêm”).

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
- SQL migration file(s) creating:
  - `putaway_rules` table (and optionally enums/check constraints as needed).
- Updated `migrations/README.md` only if new conventions are introduced (optional).

## Specific Sub-tasks
- [ ] Review existing warehouse/location schema and confirm the correct referenced tables/PKs for locations/warehouses (to ensure composite FKs can be enforced with `(tenant_id, ...)`).
- [ ] Draft the SQL migration to create `putaway_rules` with required columns (tenant_id, rule_id, priority, active flag, timestamps, and condition fields).
- [ ] Add composite primary key `(tenant_id, putaway_rule_id)` and tenant-safe foreign keys for every referenced entity that is tenant-scoped.
- [ ] Add performance indexes for rule resolution:
  - [ ] `(tenant_id, is_active, priority)`
  - [ ] `(tenant_id, product_id)`
  - [ ] `(tenant_id, product_category_id)`
  - [ ] `(tenant_id, source_location_id)`
  - [ ] `(tenant_id, destination_location_id)`
- [ ] Decide and document (in migration comments) whether `storage_category_id` FK is included now or left nullable without FK until `04.02.03` is Done.
- [ ] Run migration locally (`sqlx migrate run`) and verify the schema and indexes exist.
- [ ] Add a brief verification note (in this task log) describing what table names/FKs were used (so later tasks can rely on it).

## Database Design (Proposed)
### Table: `putaway_rules`
Tenant-scoped rule definition with priority ordering and optional qualifiers.

**Recommended columns:**
- `tenant_id UUID NOT NULL`
- `putaway_rule_id UUID NOT NULL` (UUID v7 in app-layer when creating via API later; DB just stores UUID)
- `name TEXT NOT NULL`
- `priority INT NOT NULL DEFAULT 100` (lower = higher priority)
- `is_active BOOLEAN NOT NULL DEFAULT TRUE`

**Conditions (nullable where appropriate):**
- `product_id UUID NULL` (apply to a specific product)
- `product_category_id UUID NULL` (apply to a category)
- `source_location_id UUID NULL` (where goods come from; e.g., receiving dock)
- `destination_location_id UUID NULL` (target location/bin)
- `storage_category_id UUID NULL` (if storage categories are introduced in `04.02.03`)

**Constraints / metadata:**
- `min_qty BIGINT NULL` / `max_qty BIGINT NULL` (support rule thresholds; units handled at app-layer)
- `notes TEXT NULL`
- `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`

**Keys and indexes (required):**
- Primary Key: `(tenant_id, putaway_rule_id)`
- Indexes:
  - `(tenant_id, is_active, priority)`
  - `(tenant_id, product_id)`
  - `(tenant_id, product_category_id)`
  - `(tenant_id, source_location_id)`
  - `(tenant_id, destination_location_id)`

**Foreign keys (tenant-aware composite where applicable):**
- `(tenant_id, product_id)` → `products(tenant_id, product_id)`
- `(tenant_id, product_category_id)` → `product_categories(tenant_id, product_category_id)` (if table exists)
- `(tenant_id, source_location_id)` → `warehouse_locations(tenant_id, location_id)` (or actual locations table)
- `(tenant_id, destination_location_id)` → same locations table
- `(tenant_id, storage_category_id)` → `storage_categories(tenant_id, storage_category_id)` (after 04.02.03)

> Note: Use DEFERRABLE constraints only if the project migration conventions require it; otherwise keep consistent with existing migrations in repo.

## Acceptance Criteria
- [ ] A new migration exists that creates `putaway_rules` with `tenant_id UUID NOT NULL`.
- [ ] Composite primary key includes `tenant_id`.
- [ ] All foreign keys that target tenant-scoped tables include `tenant_id` in the FK.
- [ ] Appropriate composite indexes exist for rule lookup by tenant and priority.
- [ ] The migration is reversible (down migration) if your migration system supports it.
- [ ] No RLS is introduced; isolation is application-level.

## Validation Steps
- [ ] Run `sqlx migrate run` against a dev database.
- [ ] Confirm indexes are created and query planner can use `(tenant_id, is_active, priority)` for rule selection.
- [ ] (Optional) Add a minimal SQL snippet to verify FK integrity with a sample tenant.

## Notes / Follow-ups
- Follow-up tasks expected:
  - `04.02.03_create_storage_categories_table.md`
  - `04.10.01_implement_putaway_rules.md` (service/algorithm + API)
- Consider adding a uniqueness constraint like `(tenant_id, name)` if names must be unique per tenant (decision deferred).

## AI Agent Log
---
*   2025-12-28 00:00: Task file created (schema planning only)
    - Added task definition for migration to support putaway rules.
    - Status: Todo
    - Files modified: `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.02_create_putaway_rules_table.md`
---
