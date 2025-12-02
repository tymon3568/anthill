use axum::http::StatusCode;
use axum_test::TestServer;
use inventory_service_api::AppState;
use inventory_service_core::models::{
    CreateStockMoveRequest, LotSerial, LotSerialLifecycle, LotSerialStatus, LotSerialTrackingType,
};
use inventory_service_core::repositories::StockMoveRepository;
use inventory_service_core::services::LotSerialService;

use shared::db::init_pool;
use std::sync::Arc;
use uuid::Uuid;

use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Serialize;

#[derive(Serialize)]
struct Claims {
    sub: String,
    groups: Vec<String>,
    preferred_username: String,
    email: String,
    exp: usize,
}

#[sqlx::test]
async fn test_lot_serial_lifecycle_endpoint() {
    // Setup database pool
    let pool = init_pool(&std::env::var("DATABASE_URL").unwrap(), 5)
        .await
        .expect("Failed to init pool");

    // Create repositories
    let lot_serial_repo =
        inventory_service_infra::repositories::LotSerialRepositoryImpl::new(pool.clone());
    let stock_move_repo =
        Arc::new(inventory_service_infra::repositories::stock::PgStockMoveRepository::new(
            Arc::new(pool.clone()),
        ));
    let warehouse_repo = Arc::new(
        inventory_service_infra::repositories::WarehouseRepositoryImpl::new(pool.clone()),
    );

    // Create service
    let lot_serial_service =
        Arc::new(inventory_service_infra::services::LotSerialServiceImpl::new(
            lot_serial_repo,
            stock_move_repo.clone(),
            warehouse_repo.clone(),
        ));

    // Create minimal AppState for test
    let app_state = AppState {
        lot_serial_service: lot_serial_service.clone(),
        warehouse_repository: warehouse_repo.clone(),
        // Dummy values for required fields
        category_service: Arc::new(inventory_service_infra::services::category::CategoryServiceImpl::new(
            inventory_service_infra::repositories::category::CategoryRepositoryImpl::new(pool.clone()),
        )),
        product_service: Arc::new(inventory_service_infra::services::product::ProductServiceImpl::new(
            inventory_service_infra::repositories::product::ProductRepositoryImpl::new(pool.clone()),
            inventory_service_infra::repositories::category::CategoryRepositoryImpl::new(pool.clone()),
        )),
        valuation_service: Arc::new(inventory_service_infra::services::valuation::ValuationServiceImpl::new(
            inventory_service_infra::repositories::valuation::ValuationRepositoryImpl::new(pool.clone()),
            inventory_service_infra::repositories::valuation::ValuationLayerRepositoryImpl::new(pool.clone()),
            inventory_service_infra::repositories::valuation::ValuationHistoryRepositoryImpl::new(pool.clone()),
        )),
        receipt_service: Arc::new(inventory_service_infra::services::receipt::ReceiptServiceImpl::new(
            Arc::new(inventory_service_infra::repositories::receipt::ReceiptRepositoryImpl::new(pool.clone())),
            inventory_service_infra::repositories::product::ProductRepositoryImpl::new(pool.clone()),
        )),
        delivery_service: Arc::new(inventory_service_api::DummyDeliveryService {}),
        transfer_service: Arc::new(inventory_service_infra::services::transfer::PgTransferService::new(
            Arc::new(inventory_service_infra::repositories::transfer::TransferRepositoryImpl::new(pool.clone())),
            Arc::new(inventory_service_infra::repositories::transfer::TransferItemRepositoryImpl::new(pool.clone())),
            stock_move_repo.clone(),
            Arc::new(inventory_service_infra::repositories::inventory_level::InventoryLevelRepositoryImpl::new(pool.clone())),
        )),
        stock_take_service: Arc::new(inventory_service_infra::services::stock_take::PgStockTakeService::new(
            pool.clone(),
            Arc::new(inventory_service_infra::repositories::stock_take::StockTakeRepositoryImpl::new(pool.clone())),
            Arc::new(inventory_service_infra::repositories::stock_take::StockTakeLineRepositoryImpl::new(pool.clone())),
            stock_move_repo.clone(),
            Arc::new(inventory_service_infra::repositories::inventory_level::InventoryLevelRepositoryImpl::new(pool.clone())),
        )),
        reconciliation_service: Arc::new(inventory_service_infra::services::reconciliation::PgStockReconciliationService::new(
            pool.clone(),
            Arc::new(inventory_service_infra::repositories::reconciliation::StockReconciliationRepositoryImpl::new(pool.clone())),
            Arc::new(inventory_service_infra::repositories::reconciliation::StockReconciliationItemRepositoryImpl::new(pool.clone())),
            stock_move_repo.clone(),
            Arc::new(inventory_service_infra::repositories::inventory_level::InventoryLevelRepositoryImpl::new(pool.clone())),
            inventory_service_infra::repositories::product::ProductRepositoryImpl::new(pool.clone()),
        )),
        rma_service: Arc::new(inventory_service_infra::services::rma::PgRmaService::new(
            Arc::new(inventory_service_infra::repositories::rma::RmaRepositoryImpl::new(pool.clone())),
            Arc::new(inventory_service_infra::repositories::rma::RmaItemRepositoryImpl::new(pool.clone())),
            stock_move_repo.clone(),
        )),
        enforcer: Arc::new(casbin::Enforcer::new(
            "shared/auth/model.conf",
            casbin::MemoryAdapter::default(),
        ).await.expect("Failed to create test enforcer")),
        jwt_secret: "test_secret".to_string(),
        kanidm_client: shared::kanidm_client::KanidmClient::new(shared::kanidm_client::KanidmConfig {
            kanidm_url: "http://localhost:8080".to_string(),
            client_id: "test_client".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_uri: "http://localhost:3000/oauth/callback".to_string(),
            scopes: vec!["openid".to_string(), "profile".to_string()],
            skip_jwt_verification: true,
            allowed_issuers: vec!["http://localhost:8080".to_string()],
            expected_audience: Some("test_client".to_string()),
        }).expect("Failed to create test Kanidm client"),
    };

    // Create test server
    let app = inventory_service_api::create_router(app_state.clone());
    let server = TestServer::new(app).unwrap();

    // Create test data
    let tenant_id = Uuid::new_v4();
    let product_id = Uuid::new_v4();
    let warehouse_id = Uuid::new_v4();
    let zone_id = Uuid::new_v4();
    let location_id = Uuid::new_v4();

    // Insert kanidm tenant group mapping
    let kanidm_group_uuid = Uuid::new_v4();
    let kanidm_group_name = format!("tenant_{}_users", tenant_id);
    sqlx::query!(
        "INSERT INTO kanidm_tenant_groups (tenant_id, kanidm_group_uuid, kanidm_group_name) VALUES ($1, $2, $3)",
        tenant_id, kanidm_group_uuid, kanidm_group_name
    )
    .execute(&pool)
    .await
    .expect("Failed to insert kanidm group");

    // Add Casbin policy for test user
    app_state
        .enforcer
        .add_policy(vec![
            "test_user".to_string(),
            "/api/v1/inventory/lot-serials/tracking/*".to_string(),
            "GET".to_string(),
        ])
        .await
        .expect("Failed to add policy");

    // Generate JWT token
    let claims = Claims {
        sub: "test_user".to_string(),
        groups: vec![kanidm_group_name],
        preferred_username: "test".to_string(),
        email: "test@example.com".to_string(),
        exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(app_state.jwt_secret.as_ref()),
    )
    .expect("Failed to encode JWT");

    // Insert test warehouse
    sqlx::query!(
        "INSERT INTO warehouses (tenant_id, warehouse_id, warehouse_name, warehouse_code, created_at, updated_at) VALUES ($1, $2, $3, $4, NOW(), NOW())",
        tenant_id, warehouse_id, "Test Warehouse", "WH001"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert warehouse");

    // Insert test warehouse zone
    sqlx::query!(
        "INSERT INTO warehouse_zones (tenant_id, zone_id, zone_name, warehouse_id, created_at, updated_at) VALUES ($1, $2, $3, $4, NOW(), NOW())",
        tenant_id, zone_id, "Test Zone", warehouse_id
    )
    .execute(&pool)
    .await
    .expect("Failed to insert zone");

    // Insert test warehouse location
    sqlx::query!(
        "INSERT INTO warehouse_locations (tenant_id, location_id, location_code, zone_id, created_at, updated_at) VALUES ($1, $2, $3, $4, NOW(), NOW())",
        tenant_id, location_id, "LOC-001", zone_id
    )
    .execute(&pool)
    .await
    .expect("Failed to insert location");

    // Create lot serial
    let lot_serial = LotSerial {
        lot_serial_id: Uuid::new_v4(),
        tenant_id,
        product_id,
        tracking_type: LotSerialTrackingType::Lot,
        lot_number: Some("LOT001".to_string()),
        serial_number: None,
        initial_quantity: Some(100),
        remaining_quantity: Some(100),
        expiry_date: None,
        status: LotSerialStatus::Active,
        warehouse_id: Some(warehouse_id),
        location_id: Some(location_id),
        created_by: Uuid::new_v4(),
        updated_by: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        deleted_at: None,
    };

    lot_serial_service
        .create_lot_serial(&lot_serial)
        .await
        .expect("Failed to create lot serial");

    // Create stock move
    let stock_move = CreateStockMoveRequest {
        product_id,
        source_location_id: None,
        destination_location_id: Some(location_id),
        move_type: "receipt".to_string(),
        quantity: 100,
        unit_cost: Some(10),
        reference_type: "receipt".to_string(),
        reference_id: Uuid::new_v4(),
        lot_serial_id: Some(lot_serial.lot_serial_id),
        idempotency_key: Uuid::new_v4().to_string(),
        move_reason: Some("Initial receipt".to_string()),
        batch_info: None,
        metadata: None,
    };

    // Persist stock move
    stock_move_repo
        .create(&stock_move, tenant_id)
        .await
        .expect("Failed to create stock move");

    // Call endpoint
    let response = server
        .get(&format!("/api/v1/inventory/lot-serials/tracking/{}", lot_serial.lot_serial_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let lifecycle: LotSerialLifecycle = response.json();

    assert_eq!(lifecycle.lot_serial.lot_serial_id, lot_serial.lot_serial_id);
    assert_eq!(lifecycle.current_warehouse_name, Some("Test Warehouse".to_string()));
    assert_eq!(lifecycle.current_location_code, Some("LOC-001".to_string()));
    assert!(!lifecycle.stock_moves.is_empty());
}
