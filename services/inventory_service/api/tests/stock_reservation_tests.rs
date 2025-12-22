//! Stock Reservation Integration Tests
//!
//! Verifies the stock reservation logic (reserve/release) exposed by InventoryService.

mod business_logic_test_helpers;

use business_logic_test_helpers::{
    cleanup_valuation_test_data, create_inventory_level, setup_test_pool,
    setup_test_tenant_product_warehouse,
};
use inventory_service_core::services::InventoryService;
use inventory_service_infra::repositories::PgInventoryRepository;
use inventory_service_infra::services::InventoryServiceImpl;
use std::sync::Arc;

/// Helper to create InventoryService
async fn create_inventory_service(pool: &sqlx::PgPool) -> InventoryServiceImpl {
    // We need ProductRepository and LotSerialRepository to instantiate PgInventoryRepository
    // because it handles both standard and lot-tracked reservations.

    // Using default implementations from existing repositories
    let product_repo = Arc::new(inventory_service_infra::repositories::ProductRepositoryImpl::new(
        pool.clone(),
    ));
    let lot_serial_repo = Arc::new(
        inventory_service_infra::repositories::LotSerialRepositoryImpl::new(pool.clone()),
    );

    let inventory_repo = Arc::new(PgInventoryRepository::new(
        Arc::new(pool.clone()),
        product_repo,
        lot_serial_repo,
    ));

    InventoryServiceImpl::new(inventory_repo)
}

#[tokio::test]
async fn test_reserve_stock_standard_product() {
    let pool = setup_test_pool().await;
    let (tenant_id, product_id, warehouse_id) = setup_test_tenant_product_warehouse(&pool).await;
    let service = create_inventory_service(&pool).await;

    // Set initial inventory: 100 available
    create_inventory_level(&pool, tenant_id, product_id, warehouse_id, 100).await;

    // Reserve 40
    service
        .reserve_stock(tenant_id, warehouse_id, product_id, 40)
        .await
        .expect("Reservation should succeed");

    // Check available stock (should be 60)
    let available = service
        .get_available_stock(tenant_id, warehouse_id, product_id)
        .await
        .expect("Should get available stock");
    assert_eq!(available, 60);

    // Verify database state directly
    let level = sqlx::query!(
        "SELECT available_quantity, reserved_quantity FROM inventory_levels
         WHERE tenant_id = $1 AND product_id = $2 AND warehouse_id = $3",
        tenant_id, product_id, warehouse_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(level.available_quantity, 60);
    assert_eq!(level.reserved_quantity, 40);

    cleanup_valuation_test_data(&pool, tenant_id).await;
}

#[tokio::test]
async fn test_reserve_insufficient_stock_fails() {
    let pool = setup_test_pool().await;
    let (tenant_id, product_id, warehouse_id) = setup_test_tenant_product_warehouse(&pool).await;
    let service = create_inventory_service(&pool).await;

    create_inventory_level(&pool, tenant_id, product_id, warehouse_id, 50).await;

    // Try to reserve 60
    let result = service
        .reserve_stock(tenant_id, warehouse_id, product_id, 60)
        .await;

    assert!(result.is_err(), "Should fail due to insufficient stock");

    // Verify stock unchanged
    let available = service
        .get_available_stock(tenant_id, warehouse_id, product_id)
        .await
        .unwrap();
    assert_eq!(available, 50);

    cleanup_valuation_test_data(&pool, tenant_id).await;
}

#[tokio::test]
async fn test_release_stock() {
    let pool = setup_test_pool().await;
    let (tenant_id, product_id, warehouse_id) = setup_test_tenant_product_warehouse(&pool).await;
    let service = create_inventory_service(&pool).await;

    // Initial: 100
    create_inventory_level(&pool, tenant_id, product_id, warehouse_id, 100).await;

    // Reserve 50
    service.reserve_stock(tenant_id, warehouse_id, product_id, 50).await.unwrap();

    // Release 20
    service
        .release_stock(tenant_id, warehouse_id, product_id, 20)
        .await
        .expect("Release should succeed");

    // Check state: 100 - 50 + 20 = 70 available, 30 reserved
    let level = sqlx::query!(
        "SELECT available_quantity, reserved_quantity FROM inventory_levels
         WHERE tenant_id = $1 AND product_id = $2 AND warehouse_id = $3",
        tenant_id, product_id, warehouse_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(level.available_quantity, 70);
    assert_eq!(level.reserved_quantity, 30);

    cleanup_valuation_test_data(&pool, tenant_id).await;
}
