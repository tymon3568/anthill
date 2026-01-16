#![allow(dead_code, unused_imports, clippy::single_component_path_imports)]

use std::sync::Arc;

use axum::Router;
use sqlx::{migrate::Migrator, PgPool};

use inventory_service_api::routes::StubDeliveryService;

use inventory_service_infra::repositories::{
    CategoryRepositoryImpl, LandedCostAllocationRepositoryImpl, LandedCostDocumentRepositoryImpl,
    LandedCostLineRepositoryImpl, LotSerialRepositoryImpl, PgDeliveryOrderItemRepository,
    PgDeliveryOrderRepository, PgInventoryLevelRepository, PgPutawayRepository,
    PgQualityControlPointRepository, PgReorderRuleRepository, PgRmaItemRepository, PgRmaRepository,
    PgStockMoveRepository, PgStockReconciliationItemRepository, PgStockReconciliationRepository,
    PgStockTakeLineRepository, PgStockTakeRepository, PgTransferItemRepository,
    PgTransferRepository, PickingMethodRepositoryImpl, ProductRepositoryImpl,
    ReceiptRepositoryImpl, ValuationRepositoryImpl, WarehouseRepositoryImpl,
};
use inventory_service_infra::services::{
    CategoryServiceImpl, LandedCostServiceImpl, LotSerialServiceImpl, PgCycleCountingService,
    PgPutawayService, PgQualityControlPointService, PgReplenishmentService, PgRmaService,
    PgScrapService, PgStockReconciliationService, PgStockTakeService, PgTransferService,
    PickingMethodServiceImpl, ReceiptServiceImpl, RedisDistributedLockService,
    ValuationServiceImpl,
};
use uuid::Uuid;

use inventory_service_api::handlers::reconciliation::create_reconciliation_routes;
use inventory_service_api::middleware::IdempotencyConfig;
use inventory_service_api::state::AppState;

use shared_auth::AuthUser;
use shared_config::Config;
use shared_db::init_pool;

/// Setup test database with migrations
pub async fn setup_test_database() -> PgPool {
    let config = Config::from_env().unwrap();
    let pool = init_pool(&config.database_url, config.max_connections.unwrap_or(10))
        .await
        .unwrap();

    // Run migrations
    let migrations_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../migrations");
    let migrator = Migrator::new(migrations_path).await.unwrap();
    migrator.run(&pool).await.unwrap();

    pool
}

/// Create test application with minimal services for reconciliation tests
pub async fn create_test_app(pool: PgPool) -> Router {
    // Clone PgPool directly (it's internally Arc-wrapped)
    let pool_ref = pool.clone();

    // -- Repositories --
    // Note: Many of these repos don't implement Clone, so we wrap them in Arc immediately
    // and DON'T try to clone the impl afterward.

    // Product - both trait object and concrete for different services
    let product_repo_impl = Arc::new(ProductRepositoryImpl::new(pool_ref.clone()));
    let product_repo: Arc<dyn inventory_service_core::repositories::product::ProductRepository> =
        product_repo_impl.clone();

    // Warehouse
    let warehouse_repo: Arc<dyn inventory_service_core::repositories::WarehouseRepository> =
        Arc::new(WarehouseRepositoryImpl::new(pool_ref.clone()));

    // Inventory Level - Some repos take Arc<PgPool>, some take PgPool. Check each.
    let inventory_repo: Arc<dyn inventory_service_core::repositories::InventoryLevelRepository> =
        Arc::new(PgInventoryLevelRepository::new(Arc::new(pool_ref.clone())));

    // Stock Move
    let stock_move_repo = Arc::new(PgStockMoveRepository::new(Arc::new(pool_ref.clone())));
    let stock_move_repo_trait: Arc<dyn inventory_service_core::repositories::StockMoveRepository> =
        stock_move_repo.clone();

    // Reconciliation
    let reconciliation_repo =
        Arc::new(PgStockReconciliationRepository::new(Arc::new(pool_ref.clone())));
    let reconciliation_item_repo =
        Arc::new(PgStockReconciliationItemRepository::new(Arc::new(pool_ref.clone())));

    // Valuation - single instance, wrap in Arc for each trait
    let valuation_repo_impl = Arc::new(ValuationRepositoryImpl::new(pool_ref.clone()));
    let valuation_repo_trait: Arc<
        dyn inventory_service_core::repositories::valuation::ValuationRepository,
    > = valuation_repo_impl.clone();
    let valuation_layer_trait: Arc<
        dyn inventory_service_core::repositories::valuation::ValuationLayerRepository,
    > = valuation_repo_impl.clone();
    let valuation_history_trait: Arc<
        dyn inventory_service_core::repositories::valuation::ValuationHistoryRepository,
    > = valuation_repo_impl.clone();

    // Category
    let category_repo = CategoryRepositoryImpl::new(pool_ref.clone());

    // Picking Method
    let picking_repo_trait: Arc<
        dyn inventory_service_core::repositories::picking_method::PickingMethodRepository
            + Send
            + Sync,
    > = Arc::new(PickingMethodRepositoryImpl::new(pool_ref.clone()));

    // Lot Serial
    let lot_serial_repo_impl = LotSerialRepositoryImpl::new(pool_ref.clone());

    // Receipt
    let receipt_repo = Arc::new(ReceiptRepositoryImpl::new(pool_ref.clone()));

    // Transfer
    let transfer_repo = Arc::new(PgTransferRepository::new(Arc::new(pool_ref.clone())));
    let transfer_item_repo = Arc::new(PgTransferItemRepository::new(Arc::new(pool_ref.clone())));

    // Stock Take
    let stock_take_repo = Arc::new(PgStockTakeRepository::new(Arc::new(pool_ref.clone())));
    let stock_take_line_repo = Arc::new(PgStockTakeLineRepository::new(Arc::new(pool_ref.clone())));

    // RMA
    let rma_repo = Arc::new(PgRmaRepository::new(Arc::new(pool_ref.clone())));
    let rma_item_repo = Arc::new(PgRmaItemRepository::new(Arc::new(pool_ref.clone())));

    // Reorder Rule - takes PgPool, not Arc<PgPool>
    let reorder_repo = Arc::new(PgReorderRuleRepository::new(pool_ref.clone()));

    // Quality - takes PgPool, not Arc<PgPool>
    let quality_repo = Arc::new(PgQualityControlPointRepository::new(pool_ref.clone()));

    // Putaway - takes PgPool, not Arc<PgPool>
    let putaway_repo = Arc::new(PgPutawayRepository::new(pool_ref.clone()));

    // -- Services --

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let distributed_lock_service = Arc::new(RedisDistributedLockService::new(&redis_url).unwrap());

    // Create a new stock move repo for putaway service (it takes by value)
    let stock_move_for_putaway = PgStockMoveRepository::new(Arc::new(pool_ref.clone()));

    // Create a new inventory level repo for reconciliation service
    let inventory_for_reconciliation =
        Arc::new(PgInventoryLevelRepository::new(Arc::new(pool_ref.clone())));

    // Reconciliation Service
    let reconciliation_service = Arc::new(PgStockReconciliationService::new(
        Arc::new(pool_ref.clone()),
        reconciliation_repo,
        reconciliation_item_repo,
        stock_move_repo.clone(),
        inventory_for_reconciliation,
        product_repo.clone(),
    ));

    // Create minimal app state
    // Cycle Counting Service
    let cycle_counting_service = Arc::new(PgCycleCountingService::new(
        Arc::new(pool_ref.clone()),
        stock_move_repo.clone(),
        Arc::new(PgInventoryLevelRepository::new(Arc::new(pool_ref.clone()))),
    ));

    let app_state = AppState {
        category_service: Arc::new(CategoryServiceImpl::new(category_repo)),
        cycle_counting_service,
        lot_serial_service: Arc::new(LotSerialServiceImpl::new(
            lot_serial_repo_impl,
            stock_move_repo.clone(),
            warehouse_repo.clone(),
        )),
        picking_method_service: Arc::new(PickingMethodServiceImpl::new(picking_repo_trait)),
        product_service: Arc::new(
            inventory_service_infra::services::product::ProductServiceImpl::new(
                product_repo.clone(),
            ),
        ),
        valuation_service: Arc::new(ValuationServiceImpl::new(
            valuation_repo_trait,
            valuation_layer_trait,
            valuation_history_trait,
        )),
        warehouse_repository: warehouse_repo.clone(),
        receipt_service: Arc::new(ReceiptServiceImpl::new(
            receipt_repo,
            product_repo_impl.clone(), // Needs concrete type, not dyn
            distributed_lock_service.clone(),
        )),
        delivery_service: Arc::new(StubDeliveryService),
        transfer_service: Arc::new(PgTransferService::new(
            transfer_repo,
            transfer_item_repo,
            stock_move_repo.clone(),
            Arc::new(PgInventoryLevelRepository::new(Arc::new(pool_ref.clone()))),
        )),
        stock_take_service: Arc::new(PgStockTakeService::new(
            Arc::new(pool_ref.clone()),
            stock_take_repo,
            stock_take_line_repo,
            stock_move_repo.clone(),
            Arc::new(PgInventoryLevelRepository::new(Arc::new(pool_ref.clone()))),
        )),
        reconciliation_service,
        rma_service: Arc::new(PgRmaService::new(rma_repo, rma_item_repo, stock_move_repo_trait)),
        landed_cost_service: Arc::new(LandedCostServiceImpl::new(
            Arc::new(LandedCostDocumentRepositoryImpl::new(pool_ref.clone())),
            Arc::new(LandedCostLineRepositoryImpl::new(pool_ref.clone())),
            Arc::new(LandedCostAllocationRepositoryImpl::new(pool_ref.clone())),
        )),
        replenishment_service: Arc::new(PgReplenishmentService::new(
            reorder_repo,
            inventory_repo.clone(),
            None,
        )),
        quality_service: Arc::new(PgQualityControlPointService::new(quality_repo)),
        putaway_service: Arc::new(PgPutawayService::new(putaway_repo, stock_move_for_putaway)),
        scrap_service: Arc::new(PgScrapService::new(Arc::new(pool_ref.clone()))),
        distributed_lock_service,
        enforcer: shared_auth::enforcer::create_enforcer(
            &std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://test:test@localhost:5433/test".to_string()),
            None,
        )
        .await
        .unwrap(),
        jwt_secret: "test_jwt_secret".to_string(),
        idempotency_state: Arc::new(
            inventory_service_api::middleware::IdempotencyState::new(IdempotencyConfig {
                redis_url: redis_url.clone(),
                ..Default::default()
            })
            .unwrap(),
        ),
    };

    Router::new()
        .nest("/api/v1/inventory/reconciliations", create_reconciliation_routes())
        .layer(axum::Extension(app_state))
}

/// Create a test user for authentication
pub async fn create_test_user(pool: &PgPool) -> AuthUser {
    let user_id = Uuid::now_v7();
    let tenant_id = Uuid::now_v7();

    // Insert test tenant if not exists
    // Using runtime queries instead of macros for test compatibility without DB connection at compile time
    let slug = format!("test-tenant-{}", tenant_id);
    sqlx::query(
        "INSERT INTO tenants (tenant_id, name, slug, plan, status, settings, created_at, updated_at)
         VALUES ($1, $2, $3, 'free', 'active', '{}'::jsonb, NOW(), NOW())
         ON CONFLICT (tenant_id) DO NOTHING",
    )
    .bind(tenant_id)
    .bind("Test Tenant")
    .bind(&slug)
    .execute(pool)
    .await
    .unwrap();

    // Insert test user if not exists
    sqlx::query(
        "INSERT INTO users (user_id, tenant_id, email, created_at) VALUES ($1, $2, $3, NOW())
         ON CONFLICT (user_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(tenant_id)
    .bind("test@example.com")
    .execute(pool)
    .await
    .unwrap();

    AuthUser {
        user_id,
        tenant_id,
        email: Some("test@example.com".to_string()),
        role: "user".to_string(),
    }
}

/// Create test inventory data for reconciliation
pub async fn create_test_inventory(pool: &PgPool, tenant_id: Uuid, warehouse_id: Uuid) {
    let product_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();

    // Insert test product
    sqlx::query(
        "INSERT INTO products (product_id, tenant_id, sku, name, created_at)
         VALUES ($1, $2, $3, $4, NOW())
         ON CONFLICT (product_id) DO NOTHING",
    )
    .bind(product_id)
    .bind(tenant_id)
    .bind("TEST001")
    .bind("Test Product")
    .execute(pool)
    .await
    .unwrap();

    // Insert test warehouse
    sqlx::query(
        "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, created_at)
         VALUES ($1, $2, $3, $4, NOW())
         ON CONFLICT (warehouse_id) DO NOTHING",
    )
    .bind(warehouse_id)
    .bind(tenant_id)
    .bind("TESTWH")
    .bind("Test Warehouse")
    .execute(pool)
    .await
    .unwrap();
}
