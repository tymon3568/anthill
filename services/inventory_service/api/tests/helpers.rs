use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;
use uuid::Uuid;

use inventory_service_api::handlers::reconciliation::create_reconciliation_routes;
use inventory_service_api::state::AppState;
use inventory_service_infra::repositories::reconciliation::{
    PgStockReconciliationItemRepository, PgStockReconciliationRepository,
};
use inventory_service_infra::repositories::stock::{
    PgInventoryLevelRepository, PgStockMoveRepository,
};
use inventory_service_infra::services::reconciliation::PgStockReconciliationService;
use shared_auth::AuthUser;
use shared_config::Config;
use shared_db::init_pool;

/// Setup test database with migrations
pub async fn setup_test_database() -> PgPool {
    let config = Config::from_env().unwrap();
    let pool = init_pool(&config.database_url).await.unwrap();

    // Run migrations
    sqlx::migrate!("../migrations").run(&pool).await.unwrap();

    pool
}

/// Create test application with all services wired
pub async fn create_test_app(pool: PgPool) -> Router {
    let shared_pool = Arc::new(pool);

    // Create repositories
    let reconciliation_repo = Arc::new(PgStockReconciliationRepository::new(shared_pool.clone()));
    let reconciliation_item_repo =
        Arc::new(PgStockReconciliationItemRepository::new(shared_pool.clone()));
    let stock_move_repo = Arc::new(PgStockMoveRepository::new(shared_pool.clone()));
    let inventory_repo = Arc::new(PgInventoryLevelRepository::new(shared_pool.clone()));

    // Create service
    let reconciliation_service = Arc::new(PgStockReconciliationService::new(
        shared_pool,
        reconciliation_repo,
        reconciliation_item_repo,
        stock_move_repo,
        inventory_repo,
    ));

    // Create app state
    let app_state = AppState {
        reconciliation_service,
        // Add other services as needed for tests
        category_service: Arc::new(
            inventory_service_infra::services::category::CategoryServiceImpl::new(
            // Mock or test implementations
        ),
        ),
        // delivery_service: Arc::new(
        //     inventory_service_infra::services::delivery::DeliveryServiceImpl::new(
        //     // Mock or test implementations
        // ),
        // ),
        delivery_service: Arc::new(DummyDeliveryService {}),
        transfer_service: Arc::new(
            inventory_service_infra::services::transfer::TransferServiceImpl::new(
            // Mock or test implementations
        ),
        ),
        stock_take_service: Arc::new(
            inventory_service_infra::services::stock_take::StockTakeServiceImpl::new(
            // Mock or test implementations
        ),
        ),
        valuation_service: Arc::new(
            inventory_service_infra::services::valuation::ValuationServiceImpl::new(
            // Mock or test implementations
        ),
        ),
        // Add other required fields
        enforcer: Arc::new(
            casbin::Enforcer::new(
                casbin::Model::from_str("").unwrap(),
                casbin::adapter::MemoryAdapter::new(vec![]),
            )
            .await
            .unwrap(),
        ),
        jwt_secret: "test_jwt_secret".to_string(),
        kanidm_client: Arc::new(shared_kanidm::KanidmClient::new(
            "http://localhost:8080".to_string(),
            "test_client".to_string(),
            "test_secret".to_string(),
        )),
    };

    create_reconciliation_routes(app_state)
}

/// Create a test user for authentication
pub async fn create_test_user(pool: &PgPool) -> AuthUser {
    let user_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();

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
        email: "test@example.com".to_string(),
        kanidm_user_id: Uuid::new_v4(),
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
        "INSERT INTO warehouses (warehouse_id, tenant_id, code, name, created_at)
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
        "INSERT INTO inventory_levels (inventory_level_id, tenant_id, warehouse_id, product_id, available_quantity, created_at)
         VALUES (gen_random_uuid(), $1, $2, $3, $4, NOW())
         ON CONFLICT (warehouse_id, product_id) DO UPDATE SET available_quantity = $4",
        tenant_id,
        warehouse_id,
        product_id,
        100
    )
    .execute(pool)
    .await
    .unwrap();
}
