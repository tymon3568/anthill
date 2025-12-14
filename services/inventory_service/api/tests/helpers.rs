#![allow(dead_code, unused_imports, clippy::single_component_path_imports)]

use std::sync::Arc;

use axum::Router;
use sqlx::{migrate::Migrator, PgPool};

use inventory_service_api::routes::DummyDeliveryService;

use inventory_service_infra::repositories::{
    CategoryRepositoryImpl, PgDeliveryOrderItemRepository, PgDeliveryOrderRepository,
    PgInventoryLevelRepository, PgStockMoveRepository, PgStockReconciliationItemRepository,
    PgStockReconciliationRepository, PgStockTakeLineRepository, PgStockTakeRepository,
    PgTransferItemRepository, PgTransferRepository, ReceiptRepositoryImpl, ValuationRepositoryImpl,
    WarehouseRepositoryImpl,
};
use inventory_service_infra::services::{
    CategoryServiceImpl, PgStockReconciliationService, PgStockTakeService, PgTransferService,
    ReceiptServiceImpl, ValuationServiceImpl,
};
use uuid::Uuid;

use inventory_service_api::handlers::reconciliation::create_reconciliation_routes;
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
    let shared_pool = Arc::new(pool);

    // Create repositories needed for reconciliation
    let reconciliation_repo = Arc::new(PgStockReconciliationRepository::new(shared_pool.clone()));
    let reconciliation_item_repo =
        Arc::new(PgStockReconciliationItemRepository::new(shared_pool.clone()));
    let stock_move_repo = Arc::new(PgStockMoveRepository::new(shared_pool.clone()));
    let inventory_repo = Arc::new(PgInventoryLevelRepository::new(shared_pool.clone()));
    let product_repo =
        Arc::new(inventory_service_infra::repositories::product::ProductRepositoryImpl::new(
            (*shared_pool).clone(),
        ));

    // Create reconciliation service
    let reconciliation_service = Arc::new(PgStockReconciliationService::new(
        shared_pool.clone(),
        reconciliation_repo,
        reconciliation_item_repo,
        stock_move_repo,
        inventory_repo,
        product_repo,
    ));

    // Create minimal app state with only required fields for reconciliation
    let app_state = AppState {
        category_service: Arc::new(CategoryServiceImpl::new(CategoryRepositoryImpl::new((*shared_pool).clone()))),
        lot_serial_service: Arc::new(inventory_service_infra::services::lot_serial::LotSerialServiceImpl::new(shared_pool.clone())),
        picking_method_service: Arc::new(inventory_service_infra::services::picking_method::PickingMethodServiceImpl::new(shared_pool.clone())),
        product_service: Arc::new(inventory_service_infra::services::product::ProductServiceImpl::new(product_repo)),
        valuation_service: Arc::new(inventory_service_infra::services::valuation::ValuationServiceImpl::new(shared_pool.clone(), Arc::new(inventory_service_infra::repositories::valuation::ValuationRepositoryImpl::new(shared_pool.clone())))),
        warehouse_repository: Arc::new(WarehouseRepositoryImpl::new((*shared_pool).clone())),
        receipt_service: Arc::new(ReceiptServiceImpl::new(
            Arc::new(ReceiptRepositoryImpl::new((*shared_pool).clone())),
            Arc::new(inventory_service_infra::repositories::product::ProductRepositoryImpl::new((*shared_pool).clone())),
            Arc::new(inventory_service_infra::services::distributed_lock::DistributedLockServiceImpl::new(shared_pool.clone())),
        )),
        delivery_service: Arc::new(DummyDeliveryService {}),
        transfer_service: Arc::new(PgTransferService::new(
            Arc::new(PgTransferRepository::new(shared_pool.clone())),
            Arc::new(PgTransferItemRepository::new(shared_pool.clone())),
            Arc::new(PgStockMoveRepository::new(shared_pool.clone())),
            Arc::new(PgInventoryLevelRepository::new(shared_pool.clone())),
        )),
        stock_take_service: Arc::new(PgStockTakeService::new(
            shared_pool.clone(),
            Arc::new(PgStockTakeRepository::new(shared_pool.clone())),
            Arc::new(PgStockTakeLineRepository::new(shared_pool.clone())),
            Arc::new(PgStockMoveRepository::new(shared_pool.clone())),
            Arc::new(PgInventoryLevelRepository::new(shared_pool.clone())),
        )),
        reconciliation_service,
        rma_service: Arc::new(inventory_service_infra::services::rma::RmaServiceImpl::new(shared_pool.clone())),
        replenishment_service: Arc::new(inventory_service_infra::services::replenishment::ReplenishmentServiceImpl::new(shared_pool.clone())),
        quality_service: Arc::new(inventory_service_infra::services::quality::QualityControlPointServiceImpl::new(shared_pool.clone())),
        putaway_service: Arc::new(inventory_service_infra::services::putaway::PutawayServiceImpl::new(shared_pool.clone())),
        distributed_lock_service: Arc::new(inventory_service_infra::services::distributed_lock::DistributedLockServiceImpl::new(shared_pool.clone())),
        enforcer: shared_auth::enforcer::create_enforcer(
            &std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://test:test@localhost:5433/test".to_string()),
            None,
        )
        .await
        .unwrap(),
        jwt_secret: "test_jwt_secret".to_string(),
        kanidm_client: shared_kanidm_client::KanidmClient::new(
            shared_kanidm_client::KanidmConfig {
                kanidm_url: "http://localhost:8080".to_string(),
                client_id: "test_client".to_string(),
                client_secret: "test_secret".to_string(),
                redirect_uri: "http://localhost:8000/callback".to_string(),
                scopes: vec!["openid".to_string()],
                skip_jwt_verification: true,
                allowed_issuers: vec!["http://localhost:8080".to_string()],
                expected_audience: Some("test_client".to_string()),
            },
        )
        .unwrap(),
        idempotency_state: Arc::new(crate::middleware::IdempotencyState::new()),
    };

    create_reconciliation_routes().layer(axum::Extension(app_state))
}

/// Create a test user for authentication
pub async fn create_test_user(pool: &PgPool) -> AuthUser {
    let user_id = Uuid::now_v7();
    let tenant_id = Uuid::now_v7();

    // Insert test tenant if not exists
    sqlx::query!(
        "INSERT INTO tenants (tenant_id, name, created_at) VALUES ($1, $2, NOW())
         ON CONFLICT (tenant_id) DO NOTHING",
        tenant_id,
        "Test Tenant"
    )
    .execute(pool)
    .await
    .unwrap();

    // Insert test user if not exists
    sqlx::query!(
        "INSERT INTO users (user_id, tenant_id, email, created_at) VALUES ($1, $2, $3, NOW())
         ON CONFLICT (user_id) DO NOTHING",
        user_id,
        tenant_id,
        "test@example.com"
    )
    .execute(pool)
    .await
    .unwrap();

    AuthUser {
        user_id,
        tenant_id,
        email: Some("test@example.com".to_string()),
        kanidm_user_id: Some(Uuid::new_v4()),
        role: "user".to_string(),
    }
}

/// Create test inventory data for reconciliation
pub async fn create_test_inventory(pool: &PgPool, tenant_id: Uuid, warehouse_id: Uuid) {
    let product_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();

    // Insert test product
    sqlx::query!(
        "INSERT INTO products (product_id, tenant_id, sku, name, created_at)
         VALUES ($1, $2, $3, $4, NOW())
         ON CONFLICT (product_id) DO NOTHING",
        product_id,
        tenant_id,
        "TEST001",
        "Test Product"
    )
    .execute(pool)
    .await
    .unwrap();

    // Insert test warehouse
    sqlx::query!(
        "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, created_at)
         VALUES ($1, $2, $3, $4, NOW())
         ON CONFLICT (warehouse_id) DO NOTHING",
        warehouse_id,
        tenant_id,
        "TESTWH",
        "Test Warehouse"
    )
    .execute(pool)
    .await
    .unwrap();

    // Insert test inventory level
    sqlx::query!(
        "INSERT INTO inventory_levels (inventory_id, tenant_id, warehouse_id, product_id, available_quantity, created_at)
         VALUES (gen_random_uuid(), $1, $2, $3, $4, NOW())
         ON CONFLICT (tenant_id, warehouse_id, product_id) DO UPDATE SET available_quantity = $4",
        tenant_id,
        warehouse_id,
        product_id,
        100
    )
    .execute(pool)
    .await
    .unwrap();
}
