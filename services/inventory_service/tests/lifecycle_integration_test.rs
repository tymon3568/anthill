use axum::http::StatusCode;
use axum_test::TestServer;
use inventory_service_api::AppState;
use inventory_service_core::models::{
    CreateStockMoveRequest, LotSerial, LotSerialLifecycle, LotSerialStatus, LotSerialTrackingType,
};
use inventory_service_core::services::LotSerialService;
use serde_json::json;
use shared::db::init_pool;
use std::sync::Arc;
use uuid::Uuid;

#[sqlx::test]
async fn test_lot_serial_lifecycle_endpoint() {
    // Setup database pool
    let pool = init_pool().await.expect("Failed to init pool");

    // Create repositories
    let lot_serial_repo =
        inventory_service_infra::repositories::LotSerialRepositoryImpl::new(pool.clone());
    let stock_move_repo =
        inventory_service_infra::repositories::StockMoveRepositoryImpl::new(pool.clone());

    // Create service
    let lot_serial_service =
        Arc::new(inventory_service_infra::services::LotSerialServiceImpl::new(
            lot_serial_repo,
            stock_move_repo,
        ));

    // Create AppState
    let app_state = AppState {
        lot_serial_service: lot_serial_service.clone(),
        // Add other services as needed
    };

    // Create test server
    let app = inventory_service_api::create_router(app_state);
    let server = TestServer::new(app).unwrap();

    // Create test data
    let tenant_id = Uuid::new_v4();
    let product_id = Uuid::new_v4();
    let warehouse_id = Uuid::new_v4(); // Assume zone_id
    let location_id = Uuid::new_v4();

    // Insert test warehouse zone
    sqlx::query!(
        "INSERT INTO warehouse_zones (tenant_id, zone_id, zone_name, warehouse_id) VALUES ($1, $2, $3, $4)",
        tenant_id, warehouse_id, "Test Zone", warehouse_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Insert test warehouse location
    sqlx::query!(
        "INSERT INTO warehouse_locations (tenant_id, location_id, location_code, zone_id) VALUES ($1, $2, $3, $4)",
        tenant_id, location_id, "LOC-001", warehouse_id
    )
    .execute(&pool)
    .await
    .unwrap();

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
        .unwrap();

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

    // Assume stock_move_repo.create is implemented
    // stock_move_repo.create(&stock_move, tenant_id).await.unwrap();

    // Call endpoint
    let response = server
        .get(&format!("/api/v1/inventory/lot-serials/tracking/{}", lot_serial.lot_serial_id))
        .add_header("Authorization", "Bearer test_token") // Mock auth
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let lifecycle: LotSerialLifecycle = response.json();

    assert_eq!(lifecycle.lot_serial.lot_serial_id, lot_serial.lot_serial_id);
    assert_eq!(lifecycle.current_warehouse_name, Some("Test Zone".to_string()));
    assert_eq!(lifecycle.current_location_code, Some("LOC-001".to_string()));
    assert!(!lifecycle.stock_moves.is_empty());
}
