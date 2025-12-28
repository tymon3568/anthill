# Task: Create Picking Methods Table (Batch/Cluster/Wave)

**Task ID:** `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.04_create_picking_methods_table.md`  
**Status:** Todo  
**Priority:** P1  
**Assignee:**  
**Last Updated:** 2025-12-28  
**Phase:** V1_MVP  
**Module:** 04_Inventory_Service → 4.2_Warehouse_Management  
**Dependencies:**  
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.01_create_warehouse_hierarchy_api.md` (must be Done)  
- Warehouse root representation exists (either `warehouses` table or a well-defined “warehouse root location” concept)  

## Context
Per `docs/INVENTORY_IMPROVE.md`, Anthill needs support for advanced picking approaches beyond FEFO-only selection. This task introduces **configuration tables** for picking methods (e.g., **batch**, **cluster**, **wave**) so later workflow endpoints (e.g., delivery order picking) can select a configured strategy per warehouse / operation type.

This task is scoped to **database schema + minimal supporting documentation** only. No API endpoints or picking execution logic belongs here.

## Goal
Add tenant-scoped tables to model picking methods and their assignment to warehouses/operation types, following Anthill multi-tenancy rules:
- every table has `tenant_id UUID NOT NULL`
- composite keys/indexes include `(tenant_id, ...)`
- timestamps use `TIMESTAMPTZ`
- no RLS; all filtering is application/repository-level

## Scope
### In scope
- SQL migration(s) creating:
  - `picking_methods` (catalog/config)
  - `warehouse_picking_method_assignments` (assignment per warehouse + operation type)
- Constraints + indexes for tenant isolation and fast lookup

### Out of scope
- Implementing picking algorithms (batch/cluster/wave execution)
- API CRUD endpoints for picking methods
- UI changes
- Consolidation of 4.3/4.4 Stock Operations modules

## Proposed Schema (high-level)
### 1) `picking_methods`
Represents a configurable picking method definition per tenant.

**Key fields**
- `tenant_id UUID NOT NULL`
- `picking_method_id UUID NOT NULL` (app-generated UUID v7; DB type UUID)
- `code TEXT NOT NULL` (stable identifier like `batch`, `cluster`, `wave`, `single`)
- `name TEXT NOT NULL`
- `method_type TEXT NOT NULL` (e.g., `batch|cluster|wave|single`)
- `settings JSONB NOT NULL DEFAULT '{}'::jsonb` (method-specific knobs; versioned by app)
- `is_active BOOLEAN NOT NULL DEFAULT TRUE`
- `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
- `deleted_at TIMESTAMPTZ` (optional; only if the project uses soft delete for config tables)

**Constraints**
- `PRIMARY KEY (tenant_id, picking_method_id)`
- `UNIQUE (tenant_id, code)`
- CHECK on `method_type` to allowed set (or use PostgreSQL enum if that’s the project style)

**Indexes**
- `(tenant_id, code)`
- `(tenant_id, is_active)` (optional)
- partial active index if using soft delete:
  - `... WHERE deleted_at IS NULL`

### 2) `warehouse_picking_method_assignments`
Assigns a picking method to a warehouse (and operation context).

**Key fields**
- `tenant_id UUID NOT NULL`
- `warehouse_id UUID NOT NULL`
- `operation_type TEXT NOT NULL` (e.g., `outbound_do|internal_transfer|rma_return`)
- `picking_method_id UUID NOT NULL`
- `priority INT NOT NULL DEFAULT 100` (lower = preferred)
- `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`

**Constraints**
- `PRIMARY KEY (tenant_id, warehouse_id, operation_type, picking_method_id)`
- `FOREIGN KEY (tenant_id, picking_method_id) REFERENCES picking_methods(tenant_id, picking_method_id)`
- If warehouses are in a `warehouses` table with composite key:
  - `FOREIGN KEY (tenant_id, warehouse_id) REFERENCES warehouses(tenant_id, warehouse_id)`

**Indexes**
- `(tenant_id, warehouse_id, operation_type, priority)`
- `(tenant_id, picking_method_id)`

## Specific Sub-tasks (Style B Checklist)
### A) Task initialization (folder-tasks required)
- [ ] Verify all **Dependencies** listed in the header are `Done` (open each dependency task file and confirm).
- [ ] Update this task header:
  - [ ] `Status: InProgress_By_[AgentName]`
  - [ ] `Assignee: [AgentName]`
  - [ ] `Last Updated: YYYY-MM-DD`
- [ ] Add a new entry to **AI Agent Log**: “Starting work + dependency check results”.

### B) Database schema (multi-tenant, indexed)
- [ ] Confirm how “warehouse” is represented in the current schema:
  - [ ] If a `warehouses` table exists: use it for `warehouse_id` FK
  - [ ] If only locations hierarchy exists: define `warehouse_id` as the root location id and reference the correct table/PK
- [ ] Create SQL migration(s) for:
  - [ ] `picking_methods`
  - [ ] `warehouse_picking_method_assignments`
- [ ] Ensure all new tables include `tenant_id UUID NOT NULL`.
- [ ] Ensure composite keys include `tenant_id`:
  - [ ] `PRIMARY KEY (tenant_id, picking_method_id)`
  - [ ] assignment PK includes `(tenant_id, warehouse_id, operation_type, picking_method_id)`
- [ ] Add composite foreign keys including `tenant_id` where referencing tenant-scoped tables:
  - [ ] `(tenant_id, picking_method_id) → picking_methods(tenant_id, picking_method_id)`
  - [ ] `(tenant_id, warehouse_id) → warehouses(tenant_id, warehouse_id)` (or equivalent)
- [ ] Add indexes for common lookups:
  - [ ] resolve assignment by `(tenant_id, warehouse_id, operation_type)` ordered by `priority`
  - [ ] list methods by `(tenant_id, code)` and `(tenant_id, is_active)`
- [ ] Decide (and document in migration comment/log) whether `deleted_at` is used for configs, or rely on `is_active` only.

### C) Validation / verification (DB-level)
- [ ] Run migration locally (e.g., `sqlx migrate run`) and confirm tables + indexes exist.
- [ ] Sanity-check the intended lookup query (documented in log), e.g.:
  - [ ] select assignment for tenant+warehouse+operation ordered by priority
  - [ ] ensure indexes match predicates

### D) Optional follow-up hooks (if repo standards require)
- [ ] If the codebase expects repository scaffolding for new tables, add minimal repo stubs in infra (tenant-filtered) without adding any API.

### E) Quality gates + task bookkeeping
- [ ] Run quality gates (if any code touched beyond migrations):
  - [ ] `cargo fmt`
  - [ ] `cargo check --workspace`
  - [ ] `cargo clippy --workspace -- -D warnings`
  - [ ] `cargo test --workspace` (if applicable)
- [ ] Update AI Agent Log with:
  - [ ] migration name(s)
  - [ ] table names/FKs used (exact referenced table names)
  - [ ] commands executed and results
- [ ] Set `Status: NeedsReview` once all acceptance criteria pass.

## Acceptance Criteria
- [ ] Migration applies cleanly on a fresh database.
- [ ] All new tables include `tenant_id` and use composite PKs / indexes.
- [ ] No cross-tenant references possible via FK design (where FKs are present).
- [ ] Query patterns supported:
  - [ ] fetch active picking methods for a tenant
  - [ ] resolve picking method for `(tenant_id, warehouse_id, operation_type)` ordered by priority
- [ ] Task file contains updated AI Agent Log entries when work is performed.

## Implementation Notes
- Prefer `TEXT + CHECK` for `method_type` unless your migrations consistently use Postgres enums.
- Do not introduce RLS; tenant isolation is application-level.
- IDs are UUID; prefer UUID v7 generated in application code.

## AI Agent Log
---
* 2025-12-28 00:00: Task created (planning)
  - Normalized header to folder-tasks style and expanded to Style B checklist for schema work.
  - Status: Todo
  - Files modified: `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.04_create_picking_methods_table.md`
---
