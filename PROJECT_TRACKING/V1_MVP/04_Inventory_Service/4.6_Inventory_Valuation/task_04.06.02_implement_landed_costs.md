# Task: Implement Landed Costs

**Task ID:** `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.6_Inventory_Valuation/task_04.06.02_implement_landed_costs.md`  
**Status:** Complete_PR_Submitted  
**Priority:** P1  
**Phase:** V1_MVP  
**Module:** 04_Inventory_Service / 4.6_Inventory_Valuation  
**Assignee:** Claude  
**Last Updated:** 2026-01-16  
**Dependencies:**  
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.01_create_goods_receipts_table.md`  
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.02_create_goods_receipt_items_table.md`  
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.3_Stock_Foundation/task_04.03.03_implement_inventory_valuation.md`

## Context
We currently support basic inventory valuation (`4.3` stock moves + valuation), but we lack *landed cost* allocation, which is required to match real-life costing (freight, customs, handling, insurance, etc.) similar to Odoo/ERPNext. This task introduces the minimum viable landed-cost feature that can allocate additional costs onto incoming stock (typically GRN receipts) and adjust inventory valuation accordingly.

This must follow Anthill rules:
- 3-crate pattern (`api → infra → core → shared/*`)
- Use `AppError` for error handling (no `unwrap`/`expect`)
- Multi-tenancy enforced in all queries (`tenant_id`)
- Money stored as `BIGINT` cents
- Prefer UUID v7 for ids (`Uuid::now_v7()`)

## Goal
Add landed-cost documents that can be posted to allocate costs to receipt lines (or stock move lines) and generate valuation adjustments consistently, with auditability and tenant isolation.

## Scope (MVP)
- Create schema for landed costs + allocation lines
- Implement APIs to:
  - Create landed cost document (draft)
  - Add cost lines (amount, currency if any; store as cents in base currency for MVP)
  - Link landed cost to a GRN (or a set of inbound stock moves)
  - Compute allocations (simple proportional methods)
  - Post landed cost → create valuation adjustment entries
- Provide reporting fields to reconcile landed cost impact

## Out of Scope (for this task)
- Multi-currency conversion (assume amounts are already in tenant base currency)
- Complex allocation methods (volume/weight), vendor bill integration
- Retroactive landed cost on already-consumed lots (requires deeper COGS adjustments)

---

## Deliverables
1. DB migrations for landed costs & allocations (tenant-scoped, indexed)
2. Core models + services traits (no infra deps)
3. Infra repositories + service implementations (Postgres/sqlx)
4. API routes/handlers + OpenAPI docs
5. Minimal tests for allocation logic and posting idempotency

## Specific Sub-tasks
- [ ] Confirm dependency task statuses are `Done` (GRN tables + valuation baseline). If not, set `Status: Blocked_By_Dependency_<task_id>_Not_Done` and log why.
- [ ] DB: Create migration(s) for:
  - [ ] `landed_costs` (tenant-scoped, status, optional `grn_id`, timestamps)
  - [ ] `landed_cost_lines` (tenant-scoped, amount in cents, allocation_method)
  - [ ] `landed_cost_allocations` (tenant-scoped, unique constraint to prevent duplicates)
  - [ ] Composite PKs/unique constraints include `tenant_id`
  - [ ] Composite FKs include `(tenant_id, ...)` where referencing tenant-scoped tables
  - [ ] Indexes for common lookups: `(tenant_id, status)`, `(tenant_id, grn_id)`, `(tenant_id, landed_cost_id)`
- [ ] Core: Add domain models/DTOs:
  - [ ] `LandedCost`, `LandedCostLine`, `LandedCostAllocation`
  - [ ] request/response DTOs for create/add-line/compute/post/get
  - [ ] enums: `LandedCostStatus`, `AllocationMethod`, `TargetType`
- [ ] Core: Add `LandedCostService` trait with methods:
  - [ ] `create_draft(ctx, req)`
  - [ ] `add_line(ctx, landed_cost_id, req)`
  - [ ] `compute_allocations(ctx, landed_cost_id, req)`
  - [ ] `post(ctx, landed_cost_id, req)` (idempotent)
  - [ ] `get_by_id(ctx, landed_cost_id)`
- [ ] Infra: Implement repositories with tenant filtering in every query:
  - [ ] `landed_costs_repository`
  - [ ] `landed_cost_lines_repository`
  - [ ] `landed_cost_allocations_repository`
- [ ] Infra: Implement allocation computation:
  - [ ] Resolve targets for a GRN (or inbound stock moves) for the tenant
  - [ ] Validate `total_value_cents > 0`
  - [ ] Compute proportional allocation (by_value) with documented rounding strategy
  - [ ] Ensure allocations sum exactly to cost total (distribute remainder deterministically)
  - [ ] Idempotent recompute: replace existing draft allocations for that landed cost
- [ ] Infra: Implement posting flow (single DB transaction):
  - [ ] Lock landed cost row, validate status is `draft`
  - [ ] Ensure allocations exist
  - [ ] Create valuation adjustment entries / link to stock valuation pipeline
  - [ ] Mark landed cost `posted` and persist `posted_at`
  - [ ] Ensure retry safety (no double-application) via status check + uniqueness where applicable
- [ ] API: Add routes + handlers:
  - [ ] `POST /api/v1/inventory/landed-costs`
  - [ ] `POST /api/v1/inventory/landed-costs/{landed_cost_id}/lines`
  - [ ] `POST /api/v1/inventory/landed-costs/{landed_cost_id}/compute`
  - [ ] `POST /api/v1/inventory/landed-costs/{landed_cost_id}/post`
  - [ ] `GET /api/v1/inventory/landed-costs/{landed_cost_id}`
  - [ ] Add `#[utoipa::path]` with globally unique `operation_id`s for each handler
- [ ] Tests:
  - [ ] Unit tests: allocation rounding sums match total; zero/empty targets rejected
  - [ ] Integration tests: posting is idempotent; tenant isolation; GRN-linked allocation works
- [ ] Quality gates:
  - [ ] `cargo fmt`
  - [ ] `cargo check --workspace`
  - [ ] `cargo clippy --workspace -- -D warnings` (or project policy)
  - [ ] `cargo test --workspace` (or affected crates)
- [ ] Update this file:
  - [ ] Mark sub-tasks as completed (`[x]`) as you finish them
  - [ ] Add detailed AI Agent Log entries per meaningful milestone
  - [ ] Set `Status: NeedsReview` when all acceptance criteria + quality gates pass

---

## Data Model (Proposed)
### Tables
#### `landed_costs`
- `tenant_id UUID NOT NULL`
- `landed_cost_id UUID NOT NULL` (UUID v7)
- `reference TEXT NULL` (human readable)
- `status TEXT NOT NULL` (`draft` | `posted` | `cancelled`)
- `grn_id UUID NULL` (link to goods receipt, optional if we support direct move linking)
- `posted_at TIMESTAMPTZ NULL`
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`

Indexes:
- PK `(tenant_id, landed_cost_id)`
- `(tenant_id, status)`
- `(tenant_id, grn_id)` (if used)

#### `landed_cost_lines`
- `tenant_id UUID NOT NULL`
- `landed_cost_line_id UUID NOT NULL` (UUID v7)
- `landed_cost_id UUID NOT NULL`
- `cost_type TEXT NOT NULL` (e.g. `freight`, `customs`, `handling`)
- `amount_cents BIGINT NOT NULL` (money in cents)
- `allocation_method TEXT NOT NULL` (`by_value` default for MVP)
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`

Indexes/constraints:
- PK `(tenant_id, landed_cost_line_id)`
- FK `(tenant_id, landed_cost_id)` → `landed_costs(tenant_id, landed_cost_id)`
- `(tenant_id, landed_cost_id)`

#### `landed_cost_allocations`
Represents computed allocation result per target line (e.g. GRN item / stock move).
- `tenant_id UUID NOT NULL`
- `landed_cost_allocation_id UUID NOT NULL` (UUID v7)
- `landed_cost_id UUID NOT NULL`
- `landed_cost_line_id UUID NOT NULL`
- `target_type TEXT NOT NULL` (`grn_item` | `stock_move`), keep flexible
- `target_id UUID NOT NULL`
- `allocated_amount_cents BIGINT NOT NULL`
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`

Indexes/constraints:
- PK `(tenant_id, landed_cost_allocation_id)`
- Unique `(tenant_id, landed_cost_line_id, target_type, target_id)` to prevent duplicates
- FK composite to `landed_costs` / `landed_cost_lines`

> Note: If GRN item tables already exist, targets should be consistent with their PKs and include `tenant_id` in any foreign key.

---

## API (Proposed)
Base: `/api/v1/inventory/landed-costs`

### Endpoints
1. `POST /api/v1/inventory/landed-costs`
   - Create landed cost (draft)
2. `POST /api/v1/inventory/landed-costs/{landed_cost_id}/lines`
   - Add cost line
3. `POST /api/v1/inventory/landed-costs/{landed_cost_id}/compute`
   - Compute allocations (idempotent; replaces previous draft allocations)
4. `POST /api/v1/inventory/landed-costs/{landed_cost_id}/post`
   - Post landed cost:
     - status → `posted`
     - create valuation adjustments (stock ledger/valuation impact)
5. `GET /api/v1/inventory/landed-costs/{landed_cost_id}`
   - Fetch landed cost + lines + allocations

Auth:
- Require authenticated tenant user (and permission check if available).

OpenAPI:
- Use unique `operation_id` values across the workspace.

---

## Allocation Logic (MVP)
Default method: `by_value` (proportional to line value in cents).
- Determine target lines (GRN items or inbound stock moves linked to GRN)
- Compute `total_value_cents = sum(target_line.value_cents)`
- For each target:
  - `allocated = round_half_up(cost_line.amount_cents * target.value_cents / total_value_cents)`
- Ensure total allocated equals cost total:
  - Use remainder distribution strategy: allocate rounding remainder to the largest-value lines first.

Edge cases:
- If `total_value_cents == 0`: return `AppError::ValidationError("Cannot allocate landed costs: total target value is 0")`
- If no targets: validation error
- Posting must be idempotent: posting twice should not double-apply adjustments

---

## Dependencies
- Must have GRN schema and endpoints available:
  - `04.04.01_create_goods_receipts_table`
  - `04.04.02_create_goods_receipt_items_table`
- Must have inventory valuation baseline:
  - `04.03.03_implement_inventory_valuation`

(If any dependency is not `Done`, mark this task blocked accordingly when claimed.)

---

## Sub-Tasks Checklist
- [ ] Define DB migrations for landed costs + lines + allocations (tenant-scoped, indexes, composite FKs)
- [ ] Add core models/DTOs for landed cost, line, allocation, and request/response payloads
- [ ] Add core service trait(s): `LandedCostService`
- [ ] Implement infra repositories + service implementation using Postgres/sqlx
- [ ] Implement API handlers/routes + OpenAPI annotations
- [ ] Implement allocation computation + rounding reconciliation
- [ ] Implement posting flow that writes valuation adjustments atomically (transaction)
- [ ] Add tests:
  - allocation rounding totals match
  - cannot allocate with empty targets / zero total
  - posting idempotency

---

## Acceptance Criteria
- Landed cost document can be created in `draft` for a tenant.
- Cost lines can be added with `amount_cents` as `BIGINT` and validated (non-negative, non-zero if required).
- Allocations can be computed deterministically and stored.
- Posting produces valuation adjustment entries without breaking tenant isolation.
- Posting is idempotent and safe under retries (works with idempotency patterns if available).
- No `unwrap()`/`expect()` in production code paths.
- All queries include `tenant_id` filtering.
- OpenAPI documentation generated with unique operation IDs.

---

## Testing & Quality Gates (Before NeedsReview)
- [ ] `cargo fmt`
- [ ] `cargo check --workspace`
- [ ] `cargo clippy --workspace -- -D warnings`
- [ ] `cargo test --workspace` (or at least affected crates)

---

## AI Agent Log:
---
* 2025-12-28 00:00: Task file created by AI
    - Added MVP scope and acceptance criteria for landed costs feature
    - Status: Todo
    - Files modified: `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.6_Inventory_Valuation/task_04.06.02_implement_landed_costs.md`
