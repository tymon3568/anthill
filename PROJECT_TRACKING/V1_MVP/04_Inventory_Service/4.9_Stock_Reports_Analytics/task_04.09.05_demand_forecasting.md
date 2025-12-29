# Task: Demand Forecasting (MVP Simplified)

**Task ID:** `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.9_Stock_Reports_Analytics/task_04.09.05_demand_forecasting.md`
**Status:** Todo
**Priority:** P2
**Assignee:**
**Last Updated:** 2025-12-28
**Phase:** V1_MVP
**Module:** 04_Inventory_Service → 4.9_Stock_Reports_Analytics
**Dependencies:**
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.9_Stock_Reports_Analytics/task_04.09.01_implement_stock_ledger_report.md`
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.9_Stock_Reports_Analytics/task_04.09.02_implement_advanced_inventory_reports.md`
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.3_Stock_Foundation/task_04.03.01_create_stock_moves_table.md` (demand signal source; must be Done)
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.4_Stock_Operations/task_04.04.10_ship_do_endpoint.md` (if shipments are the demand signal; must be Done)

## Context
Reference: `docs/INVENTORY_IMPROVE.md` (move forecasting from 4.12 → 4.9).

Implement **simple demand forecasting** for inventory planning using deterministic, explainable methods (SMA/WMA). This task intentionally avoids ML and complex multi-echelon optimization.

## Goal
Provide a baseline forecast per `(tenant_id, product_id, optional warehouse/location scope, horizon)`:
- Deterministic output for identical inputs
- Tenant-safe (all queries filtered by `tenant_id`)
- Fast enough for analytics usage

## Scope
### In scope (MVP)
- Demand signal based on historical consumption/shipment events (must choose and document the canonical source)
- Methods:
  - Simple Moving Average (SMA) (required)
  - Weighted Moving Average (WMA) (optional but preferred)
- Granularity:
  - Weekly buckets by default (day buckets optional)
- Output:
  - Forecast quantities for next N buckets (default 4 weeks)
  - Diagnostics metadata: method, bucket size, history window, last demand date, generated_at

### Out of scope (later)
- ML forecasting
- Seasonality/promo modeling
- Multi-echelon planning and lead-time/service-level optimization

## Specific Sub-tasks (Style B Checklist)

### A) Task initialization (folder-tasks required)
- [ ] Verify all **Dependencies** listed in the header are `Done` (open each dependency task file and confirm).
- [ ] Update this task header before code work:
  - [ ] `Status: InProgress_By_[AgentName]`
  - [ ] `Assignee: [AgentName]`
  - [ ] `Last Updated: YYYY-MM-DD`
- [ ] Add an entry to **AI Agent Log**: “Starting work + dependency check results”.

### B) Define demand signal + bucket semantics (must be explicit)
- [ ] Decide the single canonical demand source for MVP and document it in this task:
  - [ ] Option 1: Outbound stock moves of specific types (recommended)
  - [ ] Option 2: Shipped Delivery Order lines (if that is canonical and reliable)
- [ ] Define inclusion/exclusion rules:
  - [ ] Which move/transaction types count as “demand”
  - [ ] How returns/negative quantities are handled (exclude vs net; document choice)
- [ ] Define time bucketing:
  - [ ] Bucket unit: `week` (default) and optionally `day`
  - [ ] Timezone policy: bucket in UTC using `TIMESTAMPTZ` fields
  - [ ] A request is deterministic given `(as_of, history_window, horizon, bucket, method, filters)`

### C) Core crate (`inventory_service_core`) — pure forecasting logic (no infra deps)
- [ ] Add DTOs:
  - [ ] `ForecastRequest` (product_id, warehouse_id/location_id optional, bucket, history_window, horizon, method, as_of optional)
  - [ ] `ForecastPoint` (period_start, period_end, qty)
  - [ ] `ForecastResponse` (metadata + points)
  - [ ] Enums: `ForecastMethod` (`Sma`, `Wma`), `ForecastBucket` (`Week`, `Day`)
- [ ] Implement pure algorithm module:
  - [ ] SMA implementation
  - [ ] WMA implementation (if included), with validation for weights
  - [ ] Ensure safe integer arithmetic on quantities (BIGINT / i64)
- [ ] Add unit tests for edge cases:
  - [ ] empty history → zeros
  - [ ] constant series → constant forecast
  - [ ] sparse series/spike → no panic, deterministic output
  - [ ] WMA weight validation (if applicable)

### D) Infra crate (`inventory_service_infra`) — demand history query (tenant-safe)
- [ ] Implement repository query to fetch historical demand series:
  - [ ] Accept `TenantContext` and always filter `tenant_id`
  - [ ] Apply warehouse/location scoping if provided (location subtree rules if required)
  - [ ] Ensure soft-delete filters are applied if underlying tables use `deleted_at`
- [ ] Implement bucketing (SQL or Rust):
  - [ ] Convert raw events into bucketed time series in a deterministic way
  - [ ] Document the approach (SQL bucketing recommended for performance)
- [ ] Add/verify indexes on underlying event tables to support queries:
  - [ ] e.g. `(tenant_id, product_id, occurred_at)` and/or `(tenant_id, location_id, occurred_at)`
  - [ ] Exact columns depend on your stock ledger/move schema; document what you used in the log

### E) API crate (`inventory_service_api`) — endpoint + OpenAPI
- [ ] Add endpoint:
  - [ ] `GET /api/v1/inventory/reports/demand-forecast`
- [ ] Parse/validate query params:
  - [ ] `product_id` required
  - [ ] optional: `warehouse_id` or `location_id`
  - [ ] `bucket` default `week`
  - [ ] `history_weeks` default 12–26
  - [ ] `horizon_weeks` default 4
  - [ ] `method` default `sma`
  - [ ] optional `as_of` (RFC3339)
- [ ] Auth + multi-tenancy:
  - [ ] Use auth extractor and build `TenantContext`
- [ ] OpenAPI:
  - [ ] Add `#[utoipa::path]` with globally unique `operation_id` (e.g., `inventory_report_demand_forecast`)

### F) Tests + quality gates
- [ ] Integration/API tests:
  - [ ] tenant isolation (tenant B cannot see tenant A data)
  - [ ] deterministic output for same request params
  - [ ] validation failures (missing product_id, invalid bucket/method)
- [ ] Quality gates (before setting `NeedsReview`):
  - [ ] `cargo fmt`
  - [ ] `cargo check --workspace`
  - [ ] `cargo clippy --workspace -- -D warnings`
  - [ ] `cargo test --workspace` (or affected crates)

## Acceptance Criteria
- [ ] Forecast endpoint exists and is documented in OpenAPI with a unique `operation_id`.
- [ ] All queries are tenant-safe (repository filters `tenant_id`).
- [ ] Forecast output is deterministic for identical inputs.
- [ ] SMA is implemented and covered by unit tests.
- [ ] (Optional) WMA is implemented with weight validation and tests.
- [ ] Integration tests cover at least one happy path and tenant isolation.
- [ ] Quality gates pass and results are recorded in the AI Agent Log.

## AI Agent Log
---
* 2025-12-28 00:00: Task file created
  - Added MVP-scoped demand forecasting task under 4.9 analytics per `docs/INVENTORY_IMPROVE.md`.
  - Status: Todo
  - Files modified: `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.9_Stock_Reports_Analytics/task_04.09.05_demand_forecasting.md`
---
