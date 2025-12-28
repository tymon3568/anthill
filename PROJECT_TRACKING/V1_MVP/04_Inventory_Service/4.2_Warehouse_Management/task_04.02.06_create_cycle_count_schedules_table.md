# Task: Create Cycle Count Schedules Table (Schema)

**Task ID:** `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.06_create_cycle_count_schedules_table.md`  
**Status:** Todo  
**Priority:** P1  
**Assignee:**  
**Last Updated:** 2025-12-28  
**Phase:** V1_MVP  
**Module:** 04_Inventory_Service → 4.2_Warehouse_Management  
**Dependencies:**  
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.01_create_warehouse_hierarchy_api.md`  
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.1_Product_Master/task_04.01.05_create_product_categories_api.md` (only if enforcing FK to categories)

## Objective
Introduce database schema to support **cycle counting schedules** (recurring inventory counts) for multi-tenant warehouse operations. This enables planned counting by location/category with optional ABC classification and feeds later workflows (stock take generation, count execution, reconciliation).

This task is **schema-only** (migrations + indexes + constraints). API endpoints and execution workflows are handled in later tasks (e.g., `04.14.01_implement_cycle_counting.md`).

## Scope
### In Scope
- Create new tables for:
  - `cycle_count_schedules` (header)
  - `cycle_count_schedule_locations` (location targets / scopes)
  - `cycle_count_schedule_categories` (optional product category scopes)
- Multi-tenant columns and indexes
- Timestamps and (optional) soft delete
- Constraints to prevent cross-tenant leakage

### Out of Scope
- API endpoints to manage schedules
- Job/cron to auto-generate stock takes
- Cycle count execution flows and reconciliation logic

## Acceptance Criteria
- [ ] A new migration exists and applies cleanly.
- [ ] All tables include `tenant_id UUID NOT NULL`.
- [ ] Composite indexes exist for common access patterns (e.g., `(tenant_id, warehouse_id)`, `(tenant_id, next_run_at)`).
- [ ] Any foreign keys referencing tenant-scoped tables include `tenant_id` as part of the FK (composite FK).
- [ ] SQL follows project standards (`TIMESTAMPTZ`, defaults, no floats for money).
- [ ] Task includes a Style B checklist and is updated as sub-tasks complete.

## Specific Sub-tasks (Style B Checklist)

### A) Task initialization (folder-tasks required)
- [ ] Verify all **Dependencies** listed in the header are `Done` (open each dependency task file and confirm).
- [ ] Update this task header:
  - [ ] `Status: InProgress_By_[AgentName]`
  - [ ] `Assignee: [AgentName]`
  - [ ] `Last Updated: YYYY-MM-DD`
- [ ] Add an entry to **AI Agent Log**: “Starting work + dependency check results”.

### B) Schema design decisions (document before writing SQL)
- [ ] Confirm the actual table/PK names used by the warehouse model:
  - [ ] warehouse table name + PK (e.g., `warehouses(tenant_id, warehouse_id)`), or warehouse-as-root-location (if no warehouses table exists)
  - [ ] locations table name + PK (e.g., `locations(tenant_id, location_id)` / `warehouse_locations(...)`)
- [ ] Decide whether to enforce product category FK now:
  - [ ] If categories are tenant-scoped DB tables with stable PKs → add composite FK
  - [ ] If categories are not persisted/ready → keep `cycle_count_schedule_categories` table but omit FK until follow-up

### C) Database migrations (schema-only)
- [ ] Add SQL migration(s) to create:
  - [ ] `cycle_count_schedules`
    - [ ] composite PK: `(tenant_id, schedule_id)`
    - [ ] required timestamps: `created_at`, `updated_at` (`TIMESTAMPTZ` with default `NOW()`)
    - [ ] optional soft delete: `deleted_at TIMESTAMPTZ NULL` (only if consistent with project convention)
    - [ ] scheduling fields: `frequency`, `interval_days`, `timezone`, `start_at`, `next_run_at`, `end_at`
    - [ ] operational flags: `is_active`, `auto_create_stock_take`
  - [ ] `cycle_count_schedule_locations`
    - [ ] composite PK: `(tenant_id, schedule_id, location_id)`
    - [ ] composite FK to schedules: `(tenant_id, schedule_id)`
    - [ ] composite FK to locations: `(tenant_id, location_id)`
  - [ ] `cycle_count_schedule_categories` (optional scope)
    - [ ] composite PK: `(tenant_id, schedule_id, category_id)`
    - [ ] composite FK to schedules: `(tenant_id, schedule_id)`
    - [ ] composite FK to categories only if category table exists and is tenant-scoped
- [ ] Add check constraints (recommended):
  - [ ] `frequency != 'custom' OR interval_days IS NOT NULL`
  - [ ] `interval_days IS NULL OR interval_days > 0`
- [ ] Add indexes for common queries:
  - [ ] `(tenant_id, next_run_at)` with partial filter for active schedules if soft-delete is used
  - [ ] `(tenant_id, warehouse_id)` with partial filter if soft-delete is used
  - [ ] `(tenant_id, location_id)` on `cycle_count_schedule_locations`
  - [ ] `(tenant_id, category_id)` on `cycle_count_schedule_categories`

### D) Verification (DB-level)
- [ ] Run migration locally (e.g., `sqlx migrate run`) and verify tables and indexes exist.
- [ ] Validate that no cross-tenant references are possible via FK design (composite FKs where applicable).
- [ ] Record the final table names and FK targets you used in the AI Agent Log (so later tasks can rely on them).

### E) Quality gates (before setting `NeedsReview`)
- [ ] If code changes are required (usually not for schema-only tasks), run:
  - [ ] `cargo fmt`
  - [ ] `cargo check --workspace`
  - [ ] `cargo clippy --workspace -- -D warnings`
  - [ ] `cargo test --workspace`

## Proposed Schema (SQL Draft)
> Adjust referenced table/column names to match your actual warehouse/location schema.

```/dev/null/task_04.02.06_schema.sql#L1-200
-- Cycle Count Schedules (multi-tenant)
-- NOTE: Replace `warehouses` / `locations` / `product_categories` names if different.

CREATE TABLE IF NOT EXISTS cycle_count_schedules (
  tenant_id              UUID        NOT NULL,
  schedule_id            UUID        NOT NULL,
  name                   TEXT        NOT NULL,
  warehouse_id           UUID        NOT NULL,

  -- Scheduling
  frequency              TEXT        NOT NULL, -- e.g. 'daily' | 'weekly' | 'monthly' | 'custom'
  interval_days          INT         NULL,     -- for 'custom' or day-based intervals
  timezone               TEXT        NOT NULL DEFAULT 'UTC',
  start_at               TIMESTAMPTZ NOT NULL,
  next_run_at            TIMESTAMPTZ NOT NULL,
  end_at                 TIMESTAMPTZ NULL,

  -- Optional classification (ABC etc.)
  abc_class              TEXT        NULL,     -- 'A' | 'B' | 'C' etc.
  min_value_cents        BIGINT      NULL,     -- inventory value threshold (money in cents)
  min_qty                BIGINT      NULL,

  -- Operational flags
  is_active              BOOLEAN     NOT NULL DEFAULT TRUE,
  auto_create_stock_take BOOLEAN     NOT NULL DEFAULT FALSE,
  notes                  TEXT        NULL,

  created_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  deleted_at             TIMESTAMPTZ NULL,

  PRIMARY KEY (tenant_id, schedule_id),

  -- If warehouses are tenant-scoped: composite FK
  FOREIGN KEY (tenant_id, warehouse_id)
    REFERENCES warehouses(tenant_id, warehouse_id)
);

-- Target locations (scope)
CREATE TABLE IF NOT EXISTS cycle_count_schedule_locations (
  tenant_id     UUID NOT NULL,
  schedule_id   UUID NOT NULL,
  location_id   UUID NOT NULL,

  created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

  PRIMARY KEY (tenant_id, schedule_id, location_id),

  FOREIGN KEY (tenant_id, schedule_id)
    REFERENCES cycle_count_schedules(tenant_id, schedule_id),

  FOREIGN KEY (tenant_id, location_id)
    REFERENCES locations(tenant_id, location_id)
);

-- Optional: target product categories (scope)
CREATE TABLE IF NOT EXISTS cycle_count_schedule_categories (
  tenant_id        UUID NOT NULL,
  schedule_id      UUID NOT NULL,
  category_id      UUID NOT NULL,

  created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

  PRIMARY KEY (tenant_id, schedule_id, category_id),

  FOREIGN KEY (tenant_id, schedule_id)
    REFERENCES cycle_count_schedules(tenant_id, schedule_id)

  -- Uncomment only if product_categories is a tenant-scoped table with composite PK
  -- FOREIGN KEY (tenant_id, category_id)
  --   REFERENCES product_categories(tenant_id, category_id)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_cycle_count_schedules_next_run
  ON cycle_count_schedules(tenant_id, next_run_at)
  WHERE deleted_at IS NULL AND is_active = TRUE;

CREATE INDEX IF NOT EXISTS idx_cycle_count_schedules_warehouse
  ON cycle_count_schedules(tenant_id, warehouse_id)
  WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_cycle_count_schedule_locations_location
  ON cycle_count_schedule_locations(tenant_id, location_id);

CREATE INDEX IF NOT EXISTS idx_cycle_count_schedule_categories_category
  ON cycle_count_schedule_categories(tenant_id, category_id);
```

## Implementation Notes
- Use **UUID v7** (`Uuid::now_v7()`) for `schedule_id` when created in application code; migrations only define UUID columns.
- Do not introduce Postgres RLS; multi-tenancy is enforced at the application/repository layer.
- If `warehouses` does not exist and warehouse is represented as a root `location`, adjust schema to reference that root entity consistently.

## AI Agent Log
---
* 2025-12-28 00:00: Task created by AI_Agent
  - Added schema task for cycle counting schedules (multi-tenant tables + indexes).
  - Status: Todo
  - Files modified: `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.2_Warehouse_Management/task_04.02.06_create_cycle_count_schedules_table.md`
