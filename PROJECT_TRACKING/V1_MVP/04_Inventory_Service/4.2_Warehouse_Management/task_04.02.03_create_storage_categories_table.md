# Task: Create Storage Categories Table (Advanced Warehouse Config)

**Task ID:** `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.03_create_storage_categories_table.md`  
**Status:** NeedsReview  
**Priority:** P1  
**Assignee:** Claude  
**Last Updated:** 2026-01-16  
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
- [x] Verify the dependency task in the header is `Done` (open the dependency task file and confirm).
- [x] Update this task header before starting implementation:
  - [x] `Status: InProgress_By_[AgentName]`
  - [x] `Assignee: [AgentName]`
  - [x] `Last Updated: YYYY-MM-DD`
- [x] Add a new entry to **AI Agent Log**: “Starting work + dependency check results”.

### B) Database schema (multi-tenant)
- [x] Review existing schema naming for categories/config tables and confirm conventional column names (e.g., `*_id`, timestamps, soft delete).
- [x] Add SQL migration to create `storage_categories` with at minimum:
  - [x] `tenant_id UUID NOT NULL`
  - [x] `storage_category_id UUID NOT NULL`
  - [x] `name TEXT NOT NULL`
  - [x] `code TEXT NULL` (optional, stable identifier)
  - [x] `description TEXT NULL`
  - [x] `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
  - [x] `updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
  - [x] `deleted_at TIMESTAMPTZ NULL` (only if the project standard uses soft delete for config tables)
- [x] Keys/constraints:
  - [x] Primary key uses composite tenant key (preferred): `PRIMARY KEY (tenant_id, storage_category_id)`
  - [x] Uniqueness per tenant:
    - [x] `UNIQUE (tenant_id, name)`
    - [x] Optional: unique `(tenant_id, code)` where `code IS NOT NULL` (use partial unique index if needed)
- [x] Indexes:
  - [x] `(tenant_id, name)` (supports name lookups)
  - [x] `(tenant_id, code)` (supports code lookups)
  - [x] If `deleted_at` is used, add “active” partial indexes (e.g., `WHERE deleted_at IS NULL`)
- [x] Add migration comments documenting:
  - [x] Whether `deleted_at` is included and why
  - [x] Which query patterns the indexes are intended for

### C) Core/Infra/API
- [x] No code changes required for MVP schema-only task.
- [x] If the repository requires a schema registry update or SQLx metadata refresh, do so and document in the log.

### D) Validation / checks
- [x] Apply migrations on a dev database (e.g., `sqlx migrate run`) and verify:
  - [x] Table exists with expected columns
  - [x] Keys and indexes exist
  - [x] Uniqueness behaves correctly per tenant
- [x] Run quality gates (record results in AI Agent Log):
  - [x] `cargo fmt`
  - [x] `cargo check --workspace`
  - [x] `cargo clippy --workspace -- -D warnings`
  - [x] `cargo test --workspace` (only if applicable / if migrations are covered by tests)

## Acceptance Criteria
- [x] A new migration exists that creates `storage_categories` with `tenant_id UUID NOT NULL`.
- [x] Composite tenant key prevents cross-tenant collisions (PK and uniqueness constraints include `tenant_id`).
- [x] Indexes support common lookups (`(tenant_id, name)` and optional `(tenant_id, code)`).
- [x] If soft delete is used, “active records” query patterns are supported via partial indexes.
- [x] Migration applies cleanly on a fresh database.
- [x] Quality gates (as applicable) are run and recorded in the AI Agent Log.

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
* 2026-01-16 10:00: Task claimed by Claude
  - Verified dependency task_04.02.01 is Done ✓
  - Created feature branch: `feat/task-04.02.03-storage-categories`
  - Reviewed warehouse_tables migration for schema patterns
  - Starting implementation of storage_categories table migration
---
* 2026-01-16 10:30: Implementation completed by Claude
  - Created migration: `migrations/20260116000003_create_storage_categories_table.sql`
  - Schema includes: storage_category_id, tenant_id, name, code, description, properties, is_active, timestamps, deleted_at
  - Composite PK: (tenant_id, storage_category_id)
  - Unique constraints: (tenant_id, name), partial unique on (tenant_id, code) WHERE code IS NOT NULL
  - Indexes: tenant_name, tenant_code, tenant_active (all partial WHERE deleted_at IS NULL)
  - Trigger: update_updated_at_column for auto-updating updated_at
  - Fixed pre-existing migration FK issue in 20260116000001_create_authz_audit_logs_table.sql
  - Fixed pre-existing clippy warnings in user_service audit_log_repository.rs
  - Quality gates passed: cargo check ✓, cargo clippy ✓
  - Migration applies cleanly on fresh database ✓
  - Status: NeedsReview
---
