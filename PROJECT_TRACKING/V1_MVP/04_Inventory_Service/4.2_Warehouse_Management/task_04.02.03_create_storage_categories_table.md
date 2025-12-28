# Task: Create Storage Categories Table (Advanced Warehouse Config)

**Task ID:** `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.03_create_storage_categories_table.md`  
**Status:** Todo  
**Priority:** P1  
**Assignee:**  
**Last Updated:** 2025-12-28  
**Phase:** V1_MVP  
**Module:** 04_Inventory_Service → 4.2_Warehouse_Management  
**Dependencies:**
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.01_create_warehouse_hierarchy_api.md` (must be Done)

## Summary
Add database schema to support **storage categories** used by advanced warehouse processes such as putaway rules, location suitability, and picking/removal constraints. This is a foundational configuration layer for Warehouse Management and must be **tenant-scoped**.

Reference: `docs/INVENTORY_IMPROVE.md` (Storage categories / advanced warehouse config).

## Goal
Provide a tenant-safe configuration table that can be referenced by:
- Putaway rules (e.g., “store chilled items in cold zone”)
- Location suitability constraints (follow-up)
- Picking/removal strategy constraints (follow-up)

## Scope
### In scope
- Add SQL migration(s) to create `storage_categories` table (tenant-scoped).
- Add constraints and indexes (composite with `tenant_id`) to support safe lookups and uniqueness.
- Ensure timestamps use `TIMESTAMPTZ` and follow DB conventions.
- Decide whether to include `deleted_at` soft delete column based on existing project patterns (document decision).

### Out of scope
- API endpoints / CRUD handlers for storage categories.
- Attaching categories to locations (FK/join table) unless already required by existing schema.
- Putaway rule engine / picking methods implementation.

## Specific Sub-tasks (Style B Checklist)

### A) Task initialization (folder-tasks required)
- [ ] Verify the dependency task in the header is `Done` (open the dependency task file and confirm).
- [ ] Update this task header before starting implementation:
  - [ ] `Status: InProgress_By_[AgentName]`
  - [ ] `Assignee: [AgentName]`
  - [ ] `Last Updated: YYYY-MM-DD`
- [ ] Add a new entry to **AI Agent Log**: “Starting work + dependency check results”.

### B) Database schema (multi-tenant)
- [ ] Review existing schema naming for categories/config tables and confirm conventional column names (e.g., `*_id`, timestamps, soft delete).
- [ ] Add SQL migration to create `storage_categories` with at minimum:
  - [ ] `tenant_id UUID NOT NULL`
  - [ ] `storage_category_id UUID NOT NULL`
  - [ ] `name TEXT NOT NULL`
  - [ ] `code TEXT NULL` (optional, stable identifier)
  - [ ] `description TEXT NULL`
  - [ ] `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
  - [ ] `updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
  - [ ] `deleted_at TIMESTAMPTZ NULL` (only if the project standard uses soft delete for config tables)
- [ ] Keys/constraints:
  - [ ] Primary key uses composite tenant key (preferred): `PRIMARY KEY (tenant_id, storage_category_id)`
  - [ ] Uniqueness per tenant:
    - [ ] `UNIQUE (tenant_id, name)`
    - [ ] Optional: unique `(tenant_id, code)` where `code IS NOT NULL` (use partial unique index if needed)
- [ ] Indexes:
  - [ ] `(tenant_id, name)` (supports name lookups)
  - [ ] `(tenant_id, code)` (supports code lookups)
  - [ ] If `deleted_at` is used, add “active” partial indexes (e.g., `WHERE deleted_at IS NULL`)
- [ ] Add migration comments documenting:
  - [ ] Whether `deleted_at` is included and why
  - [ ] Which query patterns the indexes are intended for

### C) Core/Infra/API
- [ ] No code changes required for MVP schema-only task.
- [ ] If the repository requires a schema registry update or SQLx metadata refresh, do so and document in the log.

### D) Validation / checks
- [ ] Apply migrations on a dev database (e.g., `sqlx migrate run`) and verify:
  - [ ] Table exists with expected columns
  - [ ] Keys and indexes exist
  - [ ] Uniqueness behaves correctly per tenant
- [ ] Run quality gates (record results in AI Agent Log):
  - [ ] `cargo fmt`
  - [ ] `cargo check --workspace`
  - [ ] `cargo clippy --workspace -- -D warnings`
  - [ ] `cargo test --workspace` (only if applicable / if migrations are covered by tests)

## Acceptance Criteria
- [ ] A new migration exists that creates `storage_categories` with `tenant_id UUID NOT NULL`.
- [ ] Composite tenant key prevents cross-tenant collisions (PK and uniqueness constraints include `tenant_id`).
- [ ] Indexes support common lookups (`(tenant_id, name)` and optional `(tenant_id, code)`).
- [ ] If soft delete is used, “active records” query patterns are supported via partial indexes.
- [ ] Migration applies cleanly on a fresh database.
- [ ] Quality gates (as applicable) are run and recorded in the AI Agent Log.

## Notes / Follow-ups
- Follow-up tasks may introduce:
  - A join table `location_storage_categories` or a FK from locations to storage categories (only once location schema is confirmed).
  - CRUD API endpoints for managing categories (separate task, not included here).

## AI Agent Log
---
* 2025-12-28 00:00: Task created (planning) by AI_Agent
  - Converted to folder-tasks Style B format (header normalization + detailed checkbox checklist).
  - Status: Todo
  - Files modified: `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.03_create_storage_categories_table.md`
---
