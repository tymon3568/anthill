# Task: Inventory Turnover Analysis

**Task ID:** `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.9_Stock_Reports_Analytics/task_04.09.04_inventory_turnover_analysis.md`  
**Status:** Todo  
**Priority:** P1  
**Assignee:**  
**Last Updated:** 2025-12-28  
**Phase:** V1_MVP  
**Module:** 04_Inventory_Service → 4.9_Stock_Reports_Analytics  
**Dependencies:**  
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.9_Stock_Reports_Analytics/task_04.09.01_implement_stock_ledger_report.md` (must be Done)  
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.3_Stock_Foundation/task_04.03.03_implement_inventory_valuation.md` (must be Done)  

## Summary
Implement an **Inventory Turnover Analysis** report for the Inventory Service to measure how efficiently inventory is being sold/consumed over time. This includes key KPIs:
- Inventory Turnover Ratio
- Days Inventory Outstanding (DIO)
- Average Inventory Value / Average On-hand Quantity
- COGS/Consumption Value over the period

Reference: `docs/INVENTORY_IMPROVE.md` (Inventory Turnover Analysis).

## Goal
Provide a tenant-scoped analytics endpoint that computes turnover metrics from existing canonical data (stock ledger/moves + valuation), with correct money handling (BIGINT cents), and strong tenant isolation.

## Scope
### In scope
- Define report inputs (time range, grouping, filters)
- Compute turnover metrics
- API endpoint + OpenAPI docs (`utoipa`, unique `operation_id`)
- Strict multi-tenancy enforcement (`tenant_id` on every query)
- Tests (unit for math + integration for endpoint + tenant isolation)

### Out of scope (for MVP)
- Demand forecasting (separate task)
- Full multi-echelon aggregation beyond filters
- Real-time dashboards

## Technical Notes / Proposed Approach
### Data sources
Prefer existing canonical sources:
- Stock ledger / stock moves for **consumption/COGS** (outgoing moves representing sales/usage)
- Inventory valuation for **opening/closing inventory value** and average value

### Definitions (MVP-safe defaults)
- **COGS/Consumption**: sum of outgoing movement value within `[from, to)` (in cents)
- **Opening inventory value**: snapshot at `from`
- **Closing inventory value**: snapshot at `to`
- **Average inventory value**: `(opening + closing) / 2` (MVP default)
- **Turnover ratio**: `cogs / avg_inventory_value` (handle divide-by-zero)
- **DIO**: `days_in_period / turnover_ratio` (handle turnover=0)

### Multi-tenancy
All repository queries MUST accept `TenantContext` and filter:
- `WHERE tenant_id = $1 ...`

### Money
All monetary values in **BIGINT cents** (`i64`). Never use floats.

### Pagination
If grouping by product and dataset can be large:
- support `limit` / `offset` (optional for MVP; document if omitted)

## Specific Sub-tasks (Style B Checklist)

### A) Task initialization (folder-tasks required)
- [ ] Verify all **Dependencies** in the header are `Done` (open each dependency task file and confirm).
- [ ] Update header before code work:
  - [ ] `Status: InProgress_By_[AgentName]`
  - [ ] `Assignee: [AgentName]`
  - [ ] `Last Updated: YYYY-MM-DD`
- [ ] Add an AI Agent Log entry: dependency check results + planned approach.

### B) Core (domain DTOs + pure calculations; zero infra deps)
- [ ] Add DTOs in `inventory_service_core`:
  - [ ] `TurnoverReportQuery` (from/to, filters, group_by, limit/offset if used)
  - [ ] `TurnoverReportRow` (metrics per group)
  - [ ] enums: `GroupBy` (`Product`, `Category`, `Warehouse`), optionally `Granularity`
- [ ] Implement pure turnover computations in core (no DB):
  - [ ] DIO/turnover ratio helpers with divide-by-zero handling
  - [ ] rounding/precision rules documented for derived metrics (ratios vs cents)
- [ ] Add/extend report service trait (e.g., `ReportsService::inventory_turnover(ctx, query)`).

### C) Infra (SQL queries + service implementation)
- [ ] Confirm canonical definitions for consumption moves (which move types count as COGS).
- [ ] Implement repository query/queries with strict tenant filtering:
  - [ ] COGS/consumption value in `[from, to)` (cents)
  - [ ] opening inventory value at `from` (cents)
  - [ ] closing inventory value at `to` (cents)
  - [ ] average inventory value (either SQL-side or computed in Rust from opening/closing)
  - [ ] grouped aggregation (`product`, `category`, `warehouse`)
- [ ] Ensure all money values are computed as `BIGINT` cents end-to-end.
- [ ] Add/verify indexes on underlying tables if needed (document exact index changes in log).
- [ ] Apply soft-delete filters if applicable (e.g., `deleted_at IS NULL`).

### D) API (Axum endpoint + OpenAPI)
- [ ] Add handler: `GET /api/v1/inventory/reports/turnover`
- [ ] Enforce auth extraction and build `TenantContext`
- [ ] Map query params → `TurnoverReportQuery` with validation
- [ ] Add `#[utoipa::path]` with globally unique `operation_id` (e.g., `inventory_report_turnover`)
- [ ] Wire route into router / state properly

### E) Tests
- [ ] Unit tests (core):
  - [ ] turnover ratio divide-by-zero behavior
  - [ ] DIO behavior when turnover=0
  - [ ] deterministic computations from fixed inputs
- [ ] Integration tests (api/infra):
  - [ ] seeded tenant A vs tenant B isolation (B cannot see A)
  - [ ] known fixture data produces expected metrics
  - [ ] endpoint auth/permission requirements (as applicable)

### F) Quality gates + task bookkeeping
- [ ] Run and record results in AI Agent Log:
  - [ ] `cargo fmt`
  - [ ] `cargo check --workspace`
  - [ ] `cargo clippy --workspace -- -D warnings` (or project policy)
  - [ ] `cargo test --workspace` (or affected crates)
- [ ] Update this task file:
  - [ ] mark completed sub-tasks as `[x]`
  - [ ] add detailed AI Agent Log entries per milestone
  - [ ] set `Status: NeedsReview` when acceptance criteria + gates pass

## Acceptance Criteria
- [ ] API endpoint exists: `GET /api/v1/inventory/reports/turnover`
- [ ] Supports filters:
  - [ ] `from` / `to` (required)
  - [ ] `warehouse_id` (optional)
  - [ ] `location_id` (optional)
  - [ ] `product_id` (optional)
  - [ ] `category_id` (optional)
  - [ ] `group_by` (optional): `product`, `category`, `warehouse` (at least `product`)
- [ ] Response returns per group:
  - [ ] `cogs_value_cents` (or `consumption_value_cents`)
  - [ ] `avg_inventory_value_cents`
  - [ ] `turnover_ratio`
  - [ ] `days_inventory_outstanding`
  - [ ] `opening_inventory_value_cents`
  - [ ] `closing_inventory_value_cents`
- [ ] All repository queries filter by `tenant_id` (no cross-tenant leakage)
- [ ] No `unwrap()`/`expect()` in production code paths; errors use `AppError`
- [ ] OpenAPI annotation exists with a globally unique `operation_id`
- [ ] Tests exist for math correctness and tenant isolation
- [ ] Quality gates pass and are recorded in AI Agent Log

## AI Agent Log
---
* 2025-12-28 00:00: Task file created
  - Normalized header and expanded to folder-tasks Style B checklist.
  - Status: Todo
  - Files modified: `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.9_Stock_Reports_Analytics/task_04.09.04_inventory_turnover_analysis.md`
---
