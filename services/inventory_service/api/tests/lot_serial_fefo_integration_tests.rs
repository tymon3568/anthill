use std::sync::Arc;

use inventory_service_core::domains::inventory::product::ProductTrackingMethod;
use inventory_service_core::models::{LotSerialStatus, LotSerialTrackingType};
use inventory_service_core::repositories::delivery_order::InventoryRepository;
use inventory_service_core::repositories::lot_serial::LotSerialRepository;
use inventory_service_infra::repositories::delivery_order::PgInventoryRepository;
use inventory_service_infra::repositories::lot_serial::LotSerialRepositoryImpl;
use inventory_service_infra::repositories::product::ProductRepositoryImpl;
use std::str::FromStr;
use uuid::Uuid;

mod helpers;

use helpers::setup_test_database;

#[sqlx::test]
async fn test_fefo_reservation_picks_earliest_expiry_first() {
    let pool = setup_test_database().await;
    let tenant_id = Uuid::now_v7();
    let warehouse_id = Uuid::now_v7();
    let product_id = Uuid::now_v7();

    // Create test tenant
    sqlx::query(
        "INSERT INTO tenants (tenant_id, name, created_at) VALUES ($1, $2, NOW())",
    )
    .bind(tenant_id)
    .bind("Test Tenant")
    .execute(&pool)
    .await
    .unwrap();

    // Create test warehouse
    sqlx::query(
        "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, created_at) VALUES ($1, $2, $3, $4, NOW())",
    )
    .bind(warehouse_id)
    .bind(tenant_id)
    .bind("TESTWH")
    .bind("Test Warehouse")
    .execute(&pool)
    .await
    .unwrap();

    // Create lot-tracked product
    sqlx::query(
        "INSERT INTO products (product_id, tenant_id, sku, name, tracking_method, created_at) VALUES ($1, $2, $3, $4, $5, NOW())",
    )
    .bind(product_id)
    .bind(tenant_id)
    .bind("LOTTEST")
    .bind("Lot Test Product")
    .bind(ProductTrackingMethod::Lot.to_string())
    .execute(&pool)
    .await
    .unwrap();

    // Create inventory level for the product
    sqlx::query(
        "INSERT INTO inventory_levels (inventory_id, tenant_id, warehouse_id, product_id, available_quantity, reserved_quantity, created_at)
         VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, NOW())",
    )
    .bind(tenant_id)
    .bind(warehouse_id)
    .bind(product_id)
    .bind(100) // available
    .bind(0)    // reserved
    .execute(&pool)
    .await
    .unwrap();

    // Create lots with different expiry dates (FEFO: earliest first)
    let lot1_id = Uuid::now_v7();
    let lot2_id = Uuid::now_v7();
    let lot3_id = Uuid::now_v7();

    // Lot 1: expires soon (should be picked first)
    sqlx::query(
        r#"INSERT INTO lots_serial_numbers (lot_serial_id, tenant_id, product_id, warehouse_id, tracking_type, lot_number, initial_quantity, remaining_quantity, expiry_date, status, created_at)
           VALUES ($1, $2, $3, $4, $5::text::lot_serial_tracking_type, $6, $7, $8, $9, $10::text::lot_serial_status, NOW())"#,
    )
    .bind(lot1_id)
    .bind(tenant_id)
    .bind(product_id)
    .bind(warehouse_id)
    .bind(LotSerialTrackingType::Lot.to_string())
    .bind("LOT001")
    .bind(50)
    .bind(50)
    .bind(chrono::Utc::now() + chrono::Duration::days(7)) // expires in...
    .bind(LotSerialStatus::Active.to_string())
    .execute(&pool)
    .await
    .unwrap();

    // Lot 2: expires later
    sqlx::query(
        r#"INSERT INTO lots_serial_numbers (lot_serial_id, tenant_id, product_id, warehouse_id, tracking_type, lot_number, initial_quantity, remaining_quantity, expiry_date, status, created_at)
           VALUES ($1, $2, $3, $4, $5::text::lot_serial_tracking_type, $6, $7, $8, $9, $10::text::lot_serial_status, NOW())"#,
    )
    .bind(lot2_id)
    .bind(tenant_id)
    .bind(product_id)
    .bind(warehouse_id)
    .bind(LotSerialTrackingType::Lot.to_string())
    .bind("LOT002")
    .bind(30)
    .bind(30)
    .bind(chrono::Utc::now() + chrono::Duration::days(30)) // expires ...
    .bind(LotSerialStatus::Active.to_string())
    .execute(&pool)
    .await
    .unwrap();

    // Lot 3: expires latest
    sqlx::query(
        r#"INSERT INTO lots_serial_numbers (lot_serial_id, tenant_id, product_id, warehouse_id, tracking_type, lot_number, initial_quantity, remaining_quantity, expiry_date, status, created_at)
           VALUES ($1, $2, $3, $4, $5::text::lot_serial_tracking_type, $6, $7, $8, $9, $10::text::lot_serial_status, NOW())"#,
    )
    .bind(lot3_id)
    .bind(tenant_id)
    .bind(product_id)
    .bind(warehouse_id)
    .bind(LotSerialTrackingType::Lot.to_string())
    .bind("LOT003")
    .bind(20)
    .bind(20)
    .bind(chrono::Utc::now() + chrono::Duration::days(60)) // expires ...
    .bind(LotSerialStatus::Active.to_string())
    .execute(&pool)
    .await
    .unwrap();

    // Setup repositories
    let product_repo = Arc::new(ProductRepositoryImpl::new(pool.clone()));
    let lot_serial_repo = Arc::new(LotSerialRepositoryImpl::new(pool.clone()));
    let inventory_repo =
        PgInventoryRepository::new(Arc::new(pool.clone()), product_repo, lot_serial_repo);

    // Reserve 40 units (should take 40 from lot1, leaving 10)
    let result = inventory_repo
        .reserve_stock(tenant_id, warehouse_id, product_id, 40)
        .await;
    assert!(result.is_ok(), "Reservation should succeed");

    // Check lot quantities after reservation
    let lot1_after = sqlx::query(
        "SELECT remaining_quantity FROM lots_serial_numbers WHERE lot_serial_id = $1",
    )
    .bind(lot1_id)
    .map(|row: sqlx::postgres::PgRow| {
        use sqlx::Row;
        let q: Option<i32> = row.get("remaining_quantity");
        q
    })
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(lot1_after, Some(10), "Lot1 should have 10 remaining");

    let lot2_after = sqlx::query(
        "SELECT remaining_quantity FROM lots_serial_numbers WHERE lot_serial_id = $1",
    )
    .bind(lot2_id)
    .map(|row: sqlx::postgres::PgRow| {
        use sqlx::Row;
        let q: Option<i32> = row.get("remaining_quantity");
        q
    })
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(lot2_after, Some(30), "Lot2 should remain unchanged");

    let lot3_after = sqlx::query(
        "SELECT remaining_quantity FROM lots_serial_numbers WHERE lot_serial_id = $1",
    )
    .bind(lot3_id)
    .map(|row: sqlx::postgres::PgRow| {
        use sqlx::Row;
        let q: Option<i32> = row.get("remaining_quantity");
        q
    })
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(lot3_after, Some(20), "Lot3 should remain unchanged");

    // Check inventory_levels updated
    let inv_level = sqlx::query(
        "SELECT available_quantity, reserved_quantity FROM inventory_levels WHERE warehouse_id = $1 AND product_id = $2",
    )
    .bind(warehouse_id)
    .bind(product_id)
    .map(|row: sqlx::postgres::PgRow| {
        use sqlx::Row;
        let a: i32 = row.get("available_quantity");
        let r: i32 = row.get("reserved_quantity");
        (a, r)
    })
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(inv_level.0, 60, "Available should decrease by 40");
    assert_eq!(inv_level.1, 40, "Reserved should increase by 40");
}

#[sqlx::test]
async fn test_fefo_prevents_picking_expired_lots() {
    let pool = setup_test_database().await;
    let tenant_id = Uuid::now_v7();
    let warehouse_id = Uuid::now_v7();
    let product_id = Uuid::now_v7();

    // Create test tenant
    sqlx::query(
        "INSERT INTO tenants (tenant_id, name, created_at) VALUES ($1, $2, NOW())",
    )
    .bind(tenant_id)
    .bind("Test Tenant")
    .execute(&pool)
    .await
    .unwrap();

    // Create test warehouse
    sqlx::query(
        "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, created_at) VALUES ($1, $2, $3, $4, NOW())",
    )
    .bind(warehouse_id)
    .bind(tenant_id)
    .bind("TESTWH")
    .bind("Test Warehouse")
    .execute(&pool)
    .await
    .unwrap();

    // Create lot-tracked product
    sqlx::query(
        "INSERT INTO products (product_id, tenant_id, sku, name, tracking_method, created_at) VALUES ($1, $2, $3, $4, $5, NOW())",
    )
    .bind(product_id)
    .bind(tenant_id)
    .bind("LOTTEST")
    .bind("Lot Test Product")
    .bind(ProductTrackingMethod::Lot.to_string())
    .execute(&pool)
    .await
    .unwrap();

    // Create inventory level
    sqlx::query(
        "INSERT INTO inventory_levels (inventory_id, tenant_id, warehouse_id, product_id, available_quantity, reserved_quantity, created_at)
         VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, NOW())",
    )
    .bind(tenant_id)
    .bind(warehouse_id)
    .bind(product_id)
    .bind(80) // 50 expired + 30 valid
    .bind(0)
    .execute(&pool)
    .await
    .unwrap();

    // Create expired lot
    let expired_lot_id = Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO lots_serial_numbers (lot_serial_id, tenant_id, product_id, warehouse_id, tracking_type, lot_number, initial_quantity, remaining_quantity, expiry_date, status, created_at)
           VALUES ($1, $2, $3, $4, $5::text::lot_serial_tracking_type, $6, $7, $8, $9, $10::text::lot_serial_status, NOW())"#,
    )
    .bind(expired_lot_id)
    .bind(tenant_id)
    .bind(product_id)
    .bind(warehouse_id)
    .bind(LotSerialTrackingType::Lot.to_string())
    .bind("EXPIRED001")
    .bind(50)
    .bind(50)
    .bind(chrono::Utc::now() - chrono::Duration::days(2)) // safely ex...
    .bind(LotSerialStatus::Active.to_string())
    .execute(&pool)
    .await
    .unwrap();

    // Create valid lot
    let valid_lot_id = Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO lots_serial_numbers (lot_serial_id, tenant_id, product_id, warehouse_id, tracking_type, lot_number, initial_quantity, remaining_quantity, expiry_date, status, created_at)
           VALUES ($1, $2, $3, $4, $5::text::lot_serial_tracking_type, $6, $7, $8, $9, $10::text::lot_serial_status, NOW())"#,
    )
    .bind(valid_lot_id)
    .bind(tenant_id)
    .bind(product_id)
    .bind(warehouse_id)
    .bind(LotSerialTrackingType::Lot.to_string())
    .bind("VALID001")
    .bind(30)
    .bind(30)
    .bind(chrono::Utc::now() + chrono::Duration::days(30)) // expires in 30 days
    .bind(LotSerialStatus::Active.to_string())
    .execute(&pool)
    .await
    .unwrap();

    // Setup repositories
    let product_repo = Arc::new(ProductRepositoryImpl::new(pool.clone()));
    let lot_serial_repo = Arc::new(LotSerialRepositoryImpl::new(pool.clone()));
    let inventory_repo =
        PgInventoryRepository::new(Arc::new(pool.clone()), product_repo, lot_serial_repo);

    // Try to reserve 40 units - should fail because only 30 non-expired units are available (expired lot is skipped)
    let result = inventory_repo
        .reserve_stock(tenant_id, warehouse_id, product_id, 40)
        .await;
    assert!(result.is_err(), "Reservation should fail due to insufficient non-expired stock");

    // Check that expired lot was not touched
    let expired_lot_after = sqlx::query(
        "SELECT remaining_quantity FROM lots_serial_numbers WHERE lot_serial_id = $1",
    )
    .bind(expired_lot_id)
    .map(|row: sqlx::postgres::PgRow| {
        use sqlx::Row;
        let q: Option<i32> = row.get("remaining_quantity");
        q
    })
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        expired_lot_after,
        Some(50),
        "Expired lot should remain unchanged"
    );

    // Valid lot should remain unchanged since reservation failed
    let valid_lot_after = sqlx::query(
        "SELECT remaining_quantity FROM lots_serial_numbers WHERE lot_serial_id = $1",
    )
    .bind(valid_lot_id)
    .map(|row: sqlx::postgres::PgRow| {
        use sqlx::Row;
        let q: Option<i32> = row.get("remaining_quantity");
        q
    })
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        valid_lot_after,
        Some(30),
        "Valid lot should remain unchanged"
    );
}

#[sqlx::test]
async fn test_quarantine_expired_lots() {
    let pool = setup_test_database().await;
    let tenant_id = Uuid::now_v7();
    let warehouse_id = Uuid::now_v7();
    let product_id = Uuid::now_v7();

    // Create test tenant
    sqlx::query(
        "INSERT INTO tenants (tenant_id, name, created_at) VALUES ($1, $2, NOW())",
    )
    .bind(tenant_id)
    .bind("Test Tenant")
    .execute(&pool)
    .await
    .unwrap();

    // Create test warehouse
    sqlx::query(
        "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, created_at) VALUES ($1, $2, $3, $4, NOW())",
    )
    .bind(warehouse_id)
    .bind(tenant_id)
    .bind("TESTWH")
    .bind("Test Warehouse")
    .execute(&pool)
    .await
    .unwrap();

    // Create lot-tracked product
    sqlx::query(
        "INSERT INTO products (product_id, tenant_id, sku, name, tracking_method, created_at) VALUES ($1, $2, $3, $4, $5, NOW())",
    )
    .bind(product_id)
    .bind(tenant_id)
    .bind("LOTTEST")
    .bind("Lot Test Product")
    .bind(ProductTrackingMethod::Lot.to_string())
    .execute(&pool)
    .await
    .unwrap();

    // Create inventory level for consistency
    sqlx::query(
        "INSERT INTO inventory_levels (inventory_id, tenant_id, warehouse_id, product_id, available_quantity, reserved_quantity, created_at)
         VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, NOW())",
    )
    .bind(tenant_id)
    .bind(warehouse_id)
    .bind(product_id)
    .bind(80) // 50 expired + 30 valid
    .bind(0)
    .execute(&pool)
    .await
    .unwrap();

    // Create expired active lot
    let expired_lot_id = Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO lots_serial_numbers (lot_serial_id, tenant_id, product_id, warehouse_id, tracking_type, lot_number, initial_quantity, remaining_quantity, expiry_date, status, created_at)
           VALUES ($1, $2, $3, $4, $5::text::lot_serial_tracking_type, $6, $7, $8, $9, $10::text::lot_serial_status, NOW())"#,
    )
    .bind(expired_lot_id)
    .bind(tenant_id)
    .bind(product_id)
    .bind(warehouse_id)
    .bind(LotSerialTrackingType::Lot.to_string())
    .bind("EXPIRED001")
    .bind(50)
    .bind(25) // partially consumed
    .bind(chrono::Utc::now() - chrono::Duration::days(2)) // safely ex...
    .bind(LotSerialStatus::Active.to_string())
    .execute(&pool)
    .await
    .unwrap();

    // Create valid lot (should not be quarantined)
    let valid_lot_id = Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO lots_serial_numbers (lot_serial_id, tenant_id, product_id, warehouse_id, tracking_type, lot_number, initial_quantity, remaining_quantity, expiry_date, status, created_at)
           VALUES ($1, $2, $3, $4, $5::text::lot_serial_tracking_type, $6, $7, $8, $9, $10::text::lot_serial_status, NOW())"#,
    )
    .bind(valid_lot_id)
    .bind(tenant_id)
    .bind(product_id)
    .bind(warehouse_id)
    .bind(LotSerialTrackingType::Lot.to_string())
    .bind("VALID001")
    .bind(30)
    .bind(30)
    .bind(chrono::Utc::now() + chrono::Duration::days(30))
    .bind(LotSerialStatus::Active.to_string())
    .execute(&pool)
    .await
    .unwrap();

    // Setup repository
    let lot_serial_repo = LotSerialRepositoryImpl::new(pool.clone());

    // Quarantine expired lots
    let quarantined_count = lot_serial_repo
        .quarantine_expired_lots(tenant_id)
        .await
        .unwrap();

    assert_eq!(quarantined_count, 1, "Should quarantine 1 expired lot");

    // Check expired lot status changed
    let expired_lot_after: Option<String> = sqlx::query_scalar(
        "SELECT status::text FROM lots_serial_numbers WHERE lot_serial_id = $1",
    )
    .bind(expired_lot_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let expired_lot_status = LotSerialStatus::from_str(&expired_lot_after.unwrap()).unwrap();
    assert_eq!(
        expired_lot_status,
        LotSerialStatus::Quarantined,
        "Expired lot should be quarantined"
    );

    // Check valid lot remains active
    let valid_lot_after: Option<String> = sqlx::query_scalar(
        "SELECT status::text FROM lots_serial_numbers WHERE lot_serial_id = $1",
    )
    .bind(valid_lot_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let valid_lot_status = LotSerialStatus::from_str(&valid_lot_after.unwrap()).unwrap();
    assert_eq!(valid_lot_status, LotSerialStatus::Active, "Valid lot should remain active");
}
