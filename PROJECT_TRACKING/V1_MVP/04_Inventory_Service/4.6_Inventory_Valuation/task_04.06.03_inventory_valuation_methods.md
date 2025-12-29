# Task: Inventory Valuation Methods (FIFO/LIFO/Avg/Standard)

**Task ID:** `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.6_Inventory_Valuation/task_04.06.03_inventory_valuation_methods.md`  
**Status:** Todo  
**Priority:** P1  
**Assignee:**  
**Last Updated:** 2025-12-28  
**Phase:** V1_MVP  
**Module:** 04_Inventory_Service → 4.6_Inventory_Valuation  
**Dependencies:**  
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.3_Stock_Foundation/task_04.03.01_create_stock_moves_table.md`  
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.3_Stock_Foundation/task_04.03.03_implement_inventory_valuation.md`  
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.11_Technical_Implementation/task_04.11.01_implement_idempotency_and_concurrency.md`  
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.10_ship_do_endpoint.md` (required for realistic outbound flows)  
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.03_create_grn_endpoint.md` (required for realistic inbound flows)  
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.6_Inventory_Valuation/task_04.06.02_implement_landed_costs.md` (recommended; can be deferred if method rollout excludes landed cost impact)

## Summary
Implement configurable inventory valuation methods per tenant (and optionally per product category/product) in the Inventory Service. Supported methods include:

- Standard Cost
- Weighted Average Cost (WAC)
- FIFO
- LIFO (optional if time-constrained; keep schema extensible)

This task covers valuation configuration + algorithm implementation + persistence/audit model, and must preserve:
- Multi-tenancy isolation (tenant_id in every query)
- Concurrency safety (transactional layer consumption for FIFO/LIFO)
- Money correctness (BIGINT cents; no floats)
- 3-crate architecture (api → infra → core → shared/*)

## Scope
### In scope
- Persisted, tenant-scoped valuation configuration (default and optional overrides)
- Valuation algorithms for inbound/outbound and resulting valuation entries:
  - Standard Cost
  - Weighted Average Cost (WAC)
  - FIFO (layers)
  - LIFO (layers) (optional for MVP, but schema must allow)
- API endpoints to get/set valuation method configuration
- Transaction-safe application of valuation during stock operations (inbound/outbound/adjustments/RMA)
- Deterministic, testable behavior (unit + integration tests)

### Out of scope
- Accounting journal posting (belongs to accounting service/integration)
- Multi-currency conversion (unless already present and required)
- Retroactive method switching that re-writes historical valuation (explicitly not MVP)

## Architecture & Constraints
- Follow 3-crate pattern: `api → infra → core → shared/*`
- Core crate must not depend on DB/network libraries.
- All tenant-scoped data must include `tenant_id` and be filtered at repository layer.
- Money represented as `BIGINT` cents (`i64`).
- No `unwrap()`/`expect()` in production code; use `AppError`.

## Acceptance Criteria
- [ ] Tenant can configure valuation method with a persisted setting.
- [ ] Default valuation method is applied when not configured.
- [ ] Inventory valuation and COGS are computed correctly for:
  - [ ] Standard Cost
  - [ ] Weighted Average Cost
  - [ ] FIFO
  - [ ] LIFO (if implemented)
- [ ] FIFO/LIFO maintain consistent valuation layers across:
  - [ ] GRN receiving (inbound)
  - [ ] DO shipping (outbound)
  - [ ] Stock transfers (no value change or tracked consistently by location if required)
  - [ ] Stock adjustments / stock takes reconciliation
  - [ ] RMA returns (reverse COGS / re-layering rules documented)
- [ ] All computations are tenant-isolated and concurrency-safe (plays well with idempotency/concurrency work in `4.11`).
- [ ] `cargo check --workspace`, `cargo clippy --workspace`, and relevant `cargo test` pass.

## Specific Sub-tasks (Style B Checklist)

### A) Task initialization (folder-tasks required)
- [ ] Verify all **Dependencies** listed in the header are `Done` (open each dependency task file and confirm).
- [ ] Update this task header before implementation:
  - [ ] `Status: InProgress_By_[AgentName]`
  - [ ] `Assignee: [AgentName]`
  - [ ] `Last Updated: YYYY-MM-DD`
- [ ] Add a new entry to **AI Agent Log**: starting work + dependency check results.

### B) Design decisions (freeze before coding)
- [ ] Decide valuation scope rules for MVP:
  - [ ] Tenant default only, or include per-product/per-category overrides
  - [ ] Document precedence if overrides exist (product > category > tenant default)
- [ ] Decide whether valuation is tracked per location or globally per product:
  - [ ] If per-location is required for FIFO/LIFO layers, document why and lock schema accordingly
- [ ] Define method-switching policy (MVP-safe):
  - [ ] Switching affects future moves only (recommended)
  - [ ] Explicitly prohibit revaluing historical moves in MVP (document behavior)

### C) Database migrations (tenant-scoped, audit-ready)
- [ ] Add migration(s) for configuration:
  - [ ] `inventory_valuation_settings` with composite uniqueness `(tenant_id, scope_type, scope_id)`
  - [ ] `method` stored as TEXT with CHECK constraint or enum (choose convention and document)
- [ ] Add migration(s) for FIFO/LIFO layers:
  - [ ] `inventory_valuation_layers`
    - [ ] `tenant_id UUID NOT NULL`
    - [ ] `layer_id UUID NOT NULL`
    - [ ] `product_id UUID NOT NULL`
    - [ ] `location_id UUID NOT NULL` (only if your design is per-location)
    - [ ] `source_move_id UUID NOT NULL`
    - [ ] `qty_remaining BIGINT NOT NULL CHECK (qty_remaining >= 0)`
    - [ ] `unit_cost_cents BIGINT NOT NULL`
    - [ ] `received_at TIMESTAMPTZ NOT NULL`
    - [ ] timestamps `created_at` (and `updated_at` if needed)
    - [ ] composite PK includes `tenant_id` (e.g., `(tenant_id, layer_id)`)
  - [ ] Add indexes to support deterministic consumption ordering:
    - [ ] `(tenant_id, product_id, received_at)` (FIFO)
    - [ ] `(tenant_id, product_id, location_id, received_at)` (if per-location)
- [ ] Add migration(s) for valuation audit entries (recommended):
  - [ ] `inventory_valuation_entries` (or align with existing valuation table if already present)
  - [ ] Ensure every monetary column is BIGINT cents, no floats
  - [ ] Index `(tenant_id, move_id)` for traceability

### D) Core crate (domain + traits; zero infra deps)
- [ ] Add/extend enums + DTOs:
  - [ ] `ValuationMethod` (`Standard`, `WeightedAverage`, `Fifo`, `Lifo`)
  - [ ] `ValuationScope` (tenant/category/product as decided)
  - [ ] DTOs: get/set valuation method request/response
- [ ] Define service trait(s):
  - [ ] `InventoryValuationService::get_method(ctx, scope)`
  - [ ] `InventoryValuationService::set_method(ctx, req)`
  - [ ] Domain methods/contracts to apply inbound/outbound valuation (exact names aligned to existing core patterns)
- [ ] Add domain validation rules:
  - [ ] prevent invalid switches if policy forbids mid-stream changes for a product with existing layers (document)
  - [ ] enforce non-negative costs, non-zero quantities where applicable

### E) Infra crate (repositories + algorithm implementation; tenant filtered)
- [ ] Implement repos for valuation settings, layers, and entries using `sqlx::query_as!` where possible
  - [ ] Every query includes `tenant_id`
  - [ ] Apply soft-delete filters if the underlying model uses them
- [ ] Implement methods:
  - [ ] Standard Cost
    - [ ] define source of standard cost (existing product cost field?) or return ValidationError if missing
  - [ ] WAC
    - [ ] define storage of running average (table/columns) or compute from entries deterministically (document trade-off)
    - [ ] implement rounding rules and ensure determinism
  - [ ] FIFO/LIFO
    - [ ] inbound: create layers
    - [ ] outbound: consume layers in deterministic order under transaction
    - [ ] use row locking strategy that avoids deadlocks (document chosen approach)
- [ ] Integrate with stock operations pipeline:
  - [ ] inbound (GRN) produces valuation entries
  - [ ] outbound (DO ship) produces COGS/valuation entries
  - [ ] adjustments/stock-takes and RMA: define and implement behavior (at minimum, documented and tested)

### F) API crate (Axum handlers + routing + OpenAPI)
- [ ] Implement endpoints (auth required):
  - [ ] `GET /api/v1/inventory/valuation/method`
  - [ ] `PUT /api/v1/inventory/valuation/method`
- [ ] Add `#[utoipa::path]` for each endpoint with globally unique `operation_id`
- [ ] Validate inputs (method, scope) and map errors to `AppError`

### G) Tests + quality gates
- [ ] Unit tests (core/domain):
  - [ ] method selection precedence (if scopes exist)
  - [ ] validation rules for switching policy
- [ ] Integration tests (infra/api) with deterministic fixtures:
  - [ ] FIFO: multiple receipts, partial issues, exact layer consumption
  - [ ] WAC: multiple receipts, issues, rounding stability
  - [ ] Standard cost: outbound valuation uses configured standard cost
  - [ ] Tenant isolation: tenant B cannot read/affect tenant A settings/layers/entries
- [ ] Run and record quality gates in AI Agent Log:
  - [ ] `cargo fmt`
  - [ ] `cargo check --workspace`
  - [ ] `cargo clippy --workspace -- -D warnings`
  - [ ] `cargo test --workspace`

## Implementation Plan
### 1) Domain model (core)
Add/extend:
- `ValuationMethod` enum: `Standard`, `WeightedAverage`, `Fifo`, `Lifo`
- DTOs:
  - `SetValuationMethodReq { method, scope }`
  - `GetValuationMethodResp { method, scope }`
- `ValuationScope`:
  - Tenant-wide default
  - Optional: `ProductCategory`, `Product` (if needed; keep schema extensible)
- Service trait, e.g. `InventoryValuationService`:
  - `get_method(ctx) -> method`
  - `set_method(ctx, dto) -> ()`
  - `apply_inbound(ctx, move_id, qty, unit_cost) -> valuation_entries`
  - `apply_outbound(ctx, move_id, qty) -> valuation_entries + cogs`

### 2) Persistence model (infra) + migrations
Create tables for configuration + (optionally) layers.

**Configuration table (required):**
- `inventory_valuation_settings`
  - `tenant_id UUID NOT NULL`
  - `scope_type TEXT NOT NULL` (tenant/product/product_category)
  - `scope_id UUID NULL` (nullable for tenant default)
  - `method TEXT NOT NULL`
  - `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
  - `updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
  - PK / unique: `(tenant_id, scope_type, scope_id)`

**Valuation layers table (required for FIFO/LIFO):**
- `inventory_valuation_layers`
  - `tenant_id UUID NOT NULL`
  - `layer_id UUID NOT NULL` (UUID v7 at app layer)
  - `product_id UUID NOT NULL`
  - `location_id UUID NOT NULL` (if valuation is location-specific; decide and document)
  - `source_move_id UUID NOT NULL` (link to stock move)
  - `qty_remaining BIGINT NOT NULL`
  - `unit_cost_cents BIGINT NOT NULL`
  - `received_at TIMESTAMPTZ NOT NULL`
  - `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
  - Indexes:
    - `(tenant_id, product_id)`
    - `(tenant_id, product_id, location_id)`
    - `(tenant_id, product_id, received_at)` for FIFO ordering
  - Constraints:
    - `qty_remaining >= 0`

**Valuation entries table (optional but recommended for audit):**
- `inventory_valuation_entries`
  - `tenant_id UUID NOT NULL`
  - `entry_id UUID NOT NULL`
  - `move_id UUID NOT NULL`
  - `product_id UUID NOT NULL`
  - `qty BIGINT NOT NULL` (positive inbound, negative outbound)
  - `unit_cost_cents BIGINT NOT NULL`
  - `total_cost_cents BIGINT NOT NULL`
  - `method TEXT NOT NULL`
  - `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
  - Index: `(tenant_id, move_id)`

> Note: If your existing `stock_moves` already capture cost/value, align and avoid duplicated truth. Prefer `valuation_entries` as the canonical audit for cost derivation.

### 3) Valuation algorithms
- **Standard Cost**
  - Read configured standard cost per product (if exists); otherwise reject outbound valuation with clear `AppError::ValidationError`.
  - Inbound does not change standard cost unless policy allows (out of scope, document).
- **Weighted Average Cost**
  - Maintain per (tenant, product[, location]) average:
    - `avg_cost = (prev_value + inbound_value) / (prev_qty + inbound_qty)`
  - Ensure rounding rules (banker’s rounding or floor) are consistent and documented.
- **FIFO/LIFO**
  - Inbound: create a new layer with inbound `qty` and `unit_cost`.
  - Outbound: consume layers ordered by `received_at` ascending (FIFO) or descending (LIFO).
  - Generate valuation entries for each consumed layer chunk.

### 4) API (api crate)
Add endpoints (auth required):
- `GET /api/v1/inventory/valuation/method`
- `PUT /api/v1/inventory/valuation/method`
  - Body: `{ "method": "...", "scope": { ... } }`

OpenAPI:
- Unique `operation_id`s across workspace.
- Document method effects and any constraints (e.g., switching methods impacts future moves only).

### 5) Concurrency & idempotency
- All inbound/outbound valuation application must be transaction-bound.
- For FIFO/LIFO consumption:
  - Lock layers rows in a deterministic order (e.g., `FOR UPDATE SKIP LOCKED` strategy) to avoid deadlocks.
  - Ensure idempotency key (from `4.11.01`) prevents double application.

### 6) Testing
Add deterministic tests for:
- Inbound sequence then outbound partial consumption (FIFO/LIFO)
- WAC with multiple receipts and partial issues
- Standard cost outbound
- Switching methods: document behavior (recommended: only affects future moves)

## Dependencies
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.3_Stock_Foundation/task_04.03.01_create_stock_moves_table.md`
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.3_Stock_Foundation/task_04.03.03_implement_inventory_valuation.md`
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.11_Technical_Implementation/task_04.11.01_implement_idempotency_and_concurrency.md`

Optional/Related:
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.6_Inventory_Valuation/task_04.06.02_implement_landed_costs.md`

## Deliverables
- Task-level design notes (in this file log)
- Migrations for valuation settings and layers (as needed)
- Service/repository implementations
- API handlers + OpenAPI updates
- Unit + integration tests

## AI Agent Log:
---
* 2025-12-28 00:00: Task created (planning)
    - Normalized header to folder-tasks metadata format (Task ID / Phase / Module / Dependencies).
    - Added Style B checkbox-based sub-task checklist (DB/Core/Infra/API/Tests/Quality gates).
    - Status: Todo
    - Files modified: `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.6_Inventory_Valuation/task_04.06.03_inventory_valuation_methods.md`
