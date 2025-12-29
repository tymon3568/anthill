# Task: Wire Real Services to Inventory Service Router

**Task ID:** `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.11_Technical_Implementation/task_04.11.10_wire_real_services_to_router.md`
**Status:** InProgress_By_Claude
**Priority:** P0 (Critical)
**Assignee:** Claude
**Last Updated:** 2025-12-28
**Phase:** V1_MVP
**Module:** 04_Inventory_Service → 4.11_Technical_Implementation

## Dependencies
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.11_Technical_Implementation/task_04.11.08_wire_inventory_service_routes.md` (Status: Done)
- `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.11_Technical_Implementation/task_04.11.09_cleanup_warnings_complete_routing.md` (Status: Done)

## Context
The inventory service router currently uses `UniversalDummyService` for ALL services instead of real implementations. This means:
- All API endpoints return placeholder/error responses
- No actual business logic is executed
- The service is non-functional despite having complete implementations in infra layer

The code in `routes/mod.rs` shows:
```rust
let dummy_service = Arc::new(UniversalDummyService);
// All services use dummy_service...
let state = crate::state::AppState {
    category_service: Arc::new(category_service), // Only category is real
    lot_serial_service: dummy_service.clone(),     // DUMMY
    picking_method_service: dummy_service.clone(), // DUMMY
    product_service: dummy_service.clone(),        // DUMMY
    // ... all others are DUMMY
};
```

And protected routes only wire categories:
```rust
let protected_routes = Router::new().nest("/api/v1/inventory/categories", Router::new());
```

## Goal
Replace all `UniversalDummyService` instances with real service implementations from `inventory_service_infra` and wire all API routes properly.

## Scope

### In Scope
1. Initialize real repository implementations from infra layer
2. Initialize real service implementations using repositories
3. Update `AppState` to use real services
4. Wire all route modules to the main router
5. Ensure all endpoints are accessible and functional

### Out of Scope
- Implementing new features
- Modifying business logic
- Database migrations
- Adding new endpoints

## Sub-tasks

### Phase 1: Repository Initialization
- [x] 1.1 Import all repository implementations from `inventory_service_infra::repositories`
- [x] 1.2 Initialize `CategoryRepositoryImpl` (already done)
- [x] 1.3 Initialize `ProductRepositoryImpl`
- [x] 1.4 Initialize `LotSerialRepositoryImpl`
- [x] 1.5 Initialize `WarehouseRepositoryImpl`
- [x] 1.6 Initialize `ReceiptRepositoryImpl`
- [ ] 1.7 Initialize `DeliveryOrderRepositoryImpl` (skipped - DeliveryServiceImpl disabled)
- [x] 1.8 Initialize `TransferRepositoryImpl`
- [x] 1.9 Initialize `StockTakeRepositoryImpl`
- [x] 1.10 Initialize `ReconciliationRepositoryImpl`
- [x] 1.11 Initialize `RmaRepositoryImpl`
- [x] 1.12 Initialize `ReplenishmentRepositoryImpl`
- [x] 1.13 Initialize `QualityRepositoryImpl`
- [x] 1.14 Initialize `PutawayRepositoryImpl`
- [x] 1.15 Initialize `PickingMethodRepositoryImpl`
- [ ] 1.16 Initialize `RemovalStrategyRepositoryImpl` (not needed for current routes)
- [x] 1.17 Initialize `ValuationRepositoryImpl`
- [x] 1.18 Initialize `StockRepositoryImpl` (PgStockMoveRepository, PgInventoryLevelRepository)
- [ ] 1.19 Initialize `EventRepositoryImpl` (for outbox - not needed for current routes)

### Phase 2: Service Initialization
- [x] 2.1 Import all service implementations from `inventory_service_infra::services`
- [x] 2.2 Initialize `CategoryServiceImpl` (already done)
- [x] 2.3 Initialize `ProductServiceImpl`
- [x] 2.4 Initialize `LotSerialServiceImpl`
- [x] 2.5 Initialize `ReceiptServiceImpl`
- [ ] 2.6 Initialize `DeliveryServiceImpl` (DISABLED - using StubDeliveryService)
- [x] 2.7 Initialize `TransferServiceImpl` (PgTransferService)
- [x] 2.8 Initialize `StockTakeServiceImpl` (PgStockTakeService)
- [x] 2.9 Initialize `ReconciliationServiceImpl` (PgStockReconciliationService)
- [x] 2.10 Initialize `RmaServiceImpl` (PgRmaService)
- [x] 2.11 Initialize `ReplenishmentServiceImpl` (PgReplenishmentService)
- [x] 2.12 Initialize `QualityServiceImpl` (PgQualityControlPointService)
- [x] 2.13 Initialize `PutawayServiceImpl` (PgPutawayService)
- [x] 2.14 Initialize `PickingMethodServiceImpl`
- [ ] 2.15 Initialize `RemovalStrategyServiceImpl` (not needed for current routes)
- [x] 2.16 Initialize `ValuationServiceImpl`
- [ ] 2.17 Initialize `InventoryServiceImpl` (not needed for current routes)
- [x] 2.18 Initialize `DistributedLockServiceImpl` (RedisDistributedLockService)
- [ ] 2.19 Initialize `CacheServiceImpl` (not needed for current routes)

### Phase 3: AppState Update
- [x] 3.1 Update `AppState` struct initialization with all real services
- [x] 3.2 Remove `UniversalDummyService` (replaced with StubDeliveryService for delivery only)
- [x] 3.3 Ensure all service fields use `Arc<dyn ServiceTrait>`
- [x] 3.4 Verify `Clone` implementation still works

### Phase 4: Route Wiring
- [x] 4.1 Create route builder functions for each domain (already exist in handlers)
- [x] 4.2 Wire `/api/v1/inventory/products` routes
- [x] 4.3 Wire `/api/v1/inventory/warehouses` routes
- [x] 4.4 Wire `/api/v1/inventory/receipts` routes
- [x] 4.5 Wire `/api/v1/inventory/deliveries` routes (stub service)
- [x] 4.6 Wire `/api/v1/inventory/transfers` routes
- [x] 4.7 Wire `/api/v1/inventory/stock-takes` routes
- [x] 4.8 Wire `/api/v1/inventory/reconciliations` routes
- [x] 4.9 Wire `/api/v1/inventory/rma` routes
- [x] 4.10 Wire `/api/v1/inventory/lots` (lot/serial) routes
- [x] 4.11 Wire `/api/v1/inventory/picking` routes
- [x] 4.12 Wire `/api/v1/inventory/putaway` routes
- [x] 4.13 Wire `/api/v1/inventory/valuation` routes
- [x] 4.14 Wire `/api/v1/inventory/quality` routes
- [x] 4.15 Wire `/api/v1/inventory/replenishment` routes
- [x] 4.16 Wire `/api/v1/inventory/reports` routes
- [x] 4.17 Wire `/api/v1/inventory/search` routes

### Phase 5: Cleanup & Validation
- [x] 5.1 Remove `UniversalDummyService` struct and all its trait implementations
- [x] 5.2 Replaced `DummyDeliveryService` with `StubDeliveryService` (returns ServiceUnavailable)
- [x] 5.3 Run `cargo check --workspace` - ensure no compilation errors ✓
- [x] 5.4 Run `cargo clippy --workspace` - fix any warnings ✓
- [ ] 5.5 Run `cargo test --workspace` - ensure tests pass (requires DB)
- [ ] 5.6 Verify service starts without panic (requires Redis + DB)
- [ ] 5.7 Test health endpoint works (requires running service)
- [ ] 5.8 Update OpenAPI spec to include all wired endpoints

## Pre-Implementation Checklist
- [x] Update this task header:
  - [x] `Status: InProgress_By_Claude`
  - [x] `Assignee: Claude`
  - [x] `Last Updated: 2025-01-10`
- [x] Add AI Agent Log entry: "Starting work + dependency check results"
- [x] Verify all dependencies are `Done`
- [x] Create feature branch: `feature/task-04-11-10-wire-real-services`

## Implementation Notes

### Files to Modify
1. `services/inventory_service/api/src/routes/mod.rs` - Main router setup
2. `services/inventory_service/api/Cargo.toml` - Ensure infra dependency is correct

### Files to Reference
1. `services/inventory_service/infra/src/lib.rs` - Check exported types
2. `services/inventory_service/infra/src/repositories/mod.rs` - Repository exports
3. `services/inventory_service/infra/src/services/mod.rs` - Service exports
4. `services/inventory_service/api/src/state.rs` - AppState definition
5. `services/inventory_service/api/src/handlers/*.rs` - Handler requirements

### Expected Code Pattern
```rust
// In create_router():

// 1. Initialize repositories
let product_repo = ProductRepositoryImpl::new(pool.clone());
let lot_serial_repo = LotSerialRepositoryImpl::new(pool.clone());
// ... more repositories

// 2. Initialize services with their dependencies
let product_service = Arc::new(ProductServiceImpl::new(product_repo));
let lot_serial_service = Arc::new(LotSerialServiceImpl::new(lot_serial_repo, pool.clone()));
// ... more services

// 3. Create AppState with real services
let state = crate::state::AppState {
    category_service: Arc::new(category_service),
    lot_serial_service,
    product_service,
    // ... all real services
};

// 4. Wire all routes
let protected_routes = Router::new()
    .nest("/api/v1/inventory/categories", category_routes(state.clone()))
    .nest("/api/v1/inventory/products", product_routes(state.clone()))
    .nest("/api/v1/inventory/warehouses", warehouse_routes(state.clone()))
    // ... all route modules
```

## Acceptance Criteria
- [x] All `UniversalDummyService` usages are removed
- [x] All services use real implementations from infra layer (except DeliveryService - stub)
- [x] All API endpoints are wired and accessible
- [x] `cargo check --workspace` passes
- [x] `cargo clippy --workspace` has no errors
- [ ] Service starts without panic (requires Redis + DB runtime)
- [ ] Health endpoint returns 200 OK (requires runtime test)
- [ ] At least one endpoint per domain responds correctly (requires runtime test)

## Risks & Mitigations
| Risk | Mitigation |
|------|------------|
| Missing repository/service implementations | Check infra exports first, stub if needed |
| Circular dependencies | Use Arc and proper layering |
| Runtime panics | Add proper error handling, test locally |
| Stack overflow (seen before) | Avoid recursive types, check utoipa schemas |

## AI Agent Log
* 2025-12-28 16:00: Task created by Claude
  - Identified critical issue: router uses dummy services
  - All business logic implementations exist but are not wired
  - Task aims to make inventory service fully functional
  - Status: Todo
  - Files modified: `PROJECT_TRACKING/V1_MVP/04_Inventory_Service/4.11_Technical_Implementation/task_04.11.10_wire_real_services_to_router.md`
---
* 2025-12-28 16:02: Task claimed by Claude
  - Dependencies verified: task_04.11.08 (Done), task_04.11.09 (Done) ✓
  - Created feature branch: `feature/task-04-11-10-wire-real-services`
  - Starting Phase 1: Repository Initialization
  - Status: InProgress_By_Claude
---
* 2025-01-10 09:30: Major refactoring completed by Claude
  - Removed all `UniversalDummyService` implementations (~800 lines of dummy code)
  - Wired all real services from `inventory_service_infra`:
    - CategoryServiceImpl, ProductServiceImpl, LotSerialServiceImpl
    - PickingMethodServiceImpl, ReceiptServiceImpl, PgTransferService
    - PgStockTakeService, PgStockReconciliationService, PgRmaService
    - PgReplenishmentService, PgQualityControlPointService, PgPutawayService
    - ValuationServiceImpl, RedisDistributedLockService
  - DeliveryService uses StubDeliveryService (returns ServiceUnavailable) 
    because DeliveryServiceImpl is commented out in infra
  - Wired all 17 route modules to the main router
  - Build verification: `cargo check` ✓, `cargo clippy` ✓
  - Following 3-crate pattern: dependency injection in `create_router()`
  - Status: InProgress_By_Claude (needs runtime testing)
---