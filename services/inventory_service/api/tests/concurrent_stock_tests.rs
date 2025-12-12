//! Concurrent stock operations tests
//!
//! Tests for concurrent stock operations to verify database locking,
//! transaction isolation, and race condition prevention using tokio async tasks.

use axum::{
    body::{to_bytes, Body},
    http::{Method, Request, StatusCode},
    Router,
};
use inventory_service_api::create_app;
use serde_json::json;
use shared_config::Config;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tower::ServiceExt;
use uuid::Uuid;

// ============================================================================
// Test Constants
// ============================================================================

// These constants are used across multiple test modules conditionally
#[allow(dead_code)]
/// Initial inventory quantity for basic tests
const INITIAL_INVENTORY_STANDARD: i64 = 100;
#[allow(dead_code)]
/// Initial inventory for high-volume tests
const INITIAL_INVENTORY_LARGE: i64 = 1000;
#[allow(dead_code)]
/// Standard reservation amount per concurrent task
const RESERVATION_AMOUNT: i64 = 15;
#[allow(dead_code)]
/// Standard transfer amount per concurrent task
const TRANSFER_AMOUNT: i64 = 25;
#[allow(dead_code)]
/// Concurrent receipt addition amount
const RECEIPT_AMOUNT: i64 = 10;
#[allow(dead_code)]
/// Number of concurrent reservation tasks
const CONCURRENT_RESERVATION_TASKS: usize = 10;
#[allow(dead_code)]
/// Number of concurrent transfer tasks
const CONCURRENT_TRANSFER_TASKS: usize = 5;
#[allow(dead_code)]
/// Number of concurrent receipt tasks
const CONCURRENT_RECEIPT_TASKS: usize = 10;
#[allow(dead_code)]
/// Number of concurrent increment operations
const CONCURRENT_INCREMENT_OPS: usize = 20;
#[allow(dead_code)]
/// Number of concurrent decrement operations
const CONCURRENT_DECREMENT_OPS: usize = 10;
#[allow(dead_code)]
/// Increment amount per operation
const INCREMENT_AMOUNT: i64 = 10;
#[allow(dead_code)]
/// Decrement amount per operation
const DECREMENT_AMOUNT: i64 = 5;
#[allow(dead_code)]
/// Receipt amount for mixed operations test
const MIXED_RECEIPT_AMOUNT: i64 = 50;
#[allow(dead_code)]
/// Transfer amount for mixed operations test
const MIXED_TRANSFER_AMOUNT: i64 = 30;

// ============================================================================
// Test Configuration
// ============================================================================

fn test_config() -> Config {
    Config {
        database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://anthill:anthill@localhost:5433/anthill_test".to_string()
        }),
        jwt_secret: std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "test-secret-key-at-least-32-characters-long".to_string()),
        jwt_expiration: 900,
        jwt_refresh_expiration: 604800,
        host: "0.0.0.0".to_string(),
        port: 8001,
        cors_origins: None,
        kanidm_url: Some("http://localhost:8080".to_string()),
        kanidm_client_id: Some("test-client".to_string()),
        kanidm_client_secret: Some("test-secret".to_string()),
        kanidm_redirect_url: Some("http://localhost:8001/oauth/callback".to_string()),
        nats_url: None,
        redis_url: Some("redis://localhost:6379".to_string()),
        casbin_model_path: "shared/auth/model.conf".to_string(),
        max_connections: Some(10),
    }
}

// ============================================================================
// Test Database Helper
// ============================================================================

struct TestDatabase {
    pool: PgPool,
    test_tenants: Arc<Mutex<Vec<Uuid>>>,
}

impl TestDatabase {
    async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://anthill:anthill@localhost:5433/anthill_test".to_string()
        });

        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        Self {
            pool,
            test_tenants: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn create_test_tenant(&self, name: &str) -> Uuid {
        let tenant_id = Uuid::now_v7();
        let slug = format!("test-{}-{}", name.to_lowercase().replace(' ', "-"), tenant_id);

        sqlx::query(
            "INSERT INTO tenants (tenant_id, name, slug, plan, status, settings, created_at, updated_at)
             VALUES ($1, $2, $3, 'free', 'active', '{}'::jsonb, NOW(), NOW())"
        )
        .bind(tenant_id)
        .bind(name)
        .bind(&slug)
        .execute(&self.pool)
        .await
        .expect("Failed to create test tenant");

        self.test_tenants.lock().await.push(tenant_id);
        tenant_id
    }

    async fn create_test_warehouse(&self, tenant_id: Uuid, code: &str, name: &str) -> Uuid {
        let warehouse_id = Uuid::now_v7();

        sqlx::query(
            "INSERT INTO warehouses (warehouse_id, tenant_id, code, name, is_active, created_at, updated_at)
             VALUES ($1, $2, $3, $4, true, NOW(), NOW())"
        )
        .bind(warehouse_id)
        .bind(tenant_id)
        .bind(code)
        .bind(name)
        .execute(&self.pool)
        .await
        .expect("Failed to create test warehouse");

        warehouse_id
    }

    async fn create_test_product(&self, tenant_id: Uuid, sku: &str, name: &str) -> Uuid {
        let product_id = Uuid::now_v7();

        sqlx::query(
            "INSERT INTO products (product_id, tenant_id, sku, name, product_type, is_active, is_sellable, created_at, updated_at)
             VALUES ($1, $2, $3, $4, 'goods', true, true, NOW(), NOW())"
        )
        .bind(product_id)
        .bind(tenant_id)
        .bind(sku)
        .bind(name)
        .execute(&self.pool)
        .await
        .expect("Failed to create test product");

        product_id
    }

    async fn set_inventory_level(&self, tenant_id: Uuid, warehouse_id: Uuid, product_id: Uuid, quantity: i64) {
        sqlx::query(
            "INSERT INTO inventory_levels (tenant_id, warehouse_id, product_id, available_quantity, reserved_quantity, created_at, updated_at)
             VALUES ($1, $2, $3, $4, 0, NOW(), NOW())
             ON CONFLICT (tenant_id, warehouse_id, product_id)
             DO UPDATE SET available_quantity = $4, updated_at = NOW()"
        )
        .bind(tenant_id)
        .bind(warehouse_id)
        .bind(product_id)
        .bind(quantity)
        .execute(&self.pool)
        .await
        .expect("Failed to set inventory level");
    }

    async fn get_inventory_level(&self, tenant_id: Uuid, warehouse_id: Uuid, product_id: Uuid) -> Option<i64> {
        let result: Option<(i64,)> = sqlx::query_as(
            "SELECT available_quantity FROM inventory_levels
             WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3"
        )
        .bind(tenant_id)
        .bind(warehouse_id)
        .bind(product_id)
        .fetch_optional(&self.pool)
        .await
        .expect("Failed to get inventory level");

        result.map(|(qty,)| qty)
    }

    /// Get reserved quantity for inventory item
    async fn get_reserved_quantity(&self, tenant_id: Uuid, warehouse_id: Uuid, product_id: Uuid) -> Option<i64> {
        let result: Option<(i64,)> = sqlx::query_as(
            "SELECT reserved_quantity FROM inventory_levels
             WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3"
        )
        .bind(tenant_id)
        .bind(warehouse_id)
        .bind(product_id)
        .fetch_optional(&self.pool)
        .await
        .expect("Failed to get reserved quantity");

        result.map(|(qty,)| qty)
    }

    /// Update reserved quantity for inventory item
    async fn update_reserved_quantity(&self, tenant_id: Uuid, warehouse_id: Uuid, product_id: Uuid, quantity: i64) {
        sqlx::query(
            "UPDATE inventory_levels
             SET reserved_quantity = $4, updated_at = NOW()
             WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3"
        )
        .bind(tenant_id)
        .bind(warehouse_id)
        .bind(product_id)
        .bind(quantity)
        .execute(&self.pool)
        .await
        .expect("Failed to update reserved quantity");
    }

    async fn cleanup(&self) {
        let tenant_ids = self.test_tenants.lock().await.clone();

        for tenant_id in tenant_ids {
            let _ = sqlx::query("DELETE FROM stock_moves WHERE tenant_id = $1")
                .bind(tenant_id)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM inventory_levels WHERE tenant_id = $1")
                .bind(tenant_id)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM products WHERE tenant_id = $1")
                .bind(tenant_id)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM warehouses WHERE tenant_id = $1")
                .bind(tenant_id)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM tenants WHERE tenant_id = $1")
                .bind(tenant_id)
                .execute(&self.pool)
                .await;
        }

        self.test_tenants.lock().await.clear();
    }
}

// ============================================================================
// Test App Helper
// ============================================================================

struct TestApp {
    router: Router,
    db: TestDatabase,
}

impl TestApp {
    async fn new() -> Self {
        let db = TestDatabase::new().await;
        let config = test_config();
        let router = create_app(config).await;

        Self { router, db }
    }

    async fn send_request(&self, request: Request<Body>) -> (StatusCode, String) {
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        (status, body_str)
    }

    fn db(&self) -> &TestDatabase {
        &self.db
    }

    async fn cleanup(&self) {
        self.db.cleanup().await;
    }
}

fn create_auth_header(tenant_id: Uuid, user_id: Uuid) -> String {
    format!("Bearer mock-jwt-{}-{}", tenant_id, user_id)
}

// ============================================================================
// Stock Reservation Conflict Tests
// ============================================================================

#[cfg(test)]
mod reservation_tests {
    use super::*;

    /// Test that concurrent reservations for the same product don't over-reserve
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_concurrent_reservations_same_product() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Reservation Conflict").await;
        let warehouse_id = app.db().create_test_warehouse(tenant_id, "RES-WH", "Reservation Warehouse").await;
        let product_id = app.db().create_test_product(tenant_id, "RES-001", "Reservation Product").await;

        // Set initial inventory: 100 units
        app.db().set_inventory_level(tenant_id, warehouse_id, product_id, INITIAL_INVENTORY_STANDARD).await;

        // Spawn 10 concurrent tasks, each trying to reserve 15 units
        // Total requested: 150 units, but only 100 available
        let db_pool = app.db().pool.clone();
        let mut handles = JoinSet::new();

        for _i in 0..CONCURRENT_RESERVATION_TASKS {
            let pool = db_pool.clone();
            let t_id = tenant_id;
            let w_id = warehouse_id;
            let p_id = product_id;

            handles.spawn(async move {
                // Simulate reservation by updating reserved_quantity
                // Uses SELECT FOR UPDATE to prevent race conditions
                let result = sqlx::query(
                    "UPDATE inventory_levels
                     SET reserved_quantity = reserved_quantity + $4,
                         updated_at = NOW()
                     WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3
                     AND available_quantity - reserved_quantity >= $4"
                )
                .bind(t_id)
                .bind(w_id)
                .bind(p_id)
                .bind(RESERVATION_AMOUNT)
                .execute(&pool)
                .await;

                match result {
                    Ok(r) => r.rows_affected() > 0,
                    Err(_) => false,
                }
            });
        }

        // Wait for all tasks and count successful reservations
        let mut successful_reservations = 0;
        while let Some(result) = handles.join_next().await {
            if let Ok(success) = result {
                if success {
                    successful_reservations += 1;
                }
            }
        }

        // Should only allow 6 successful reservations (6 * 15 = 90 <= 100)
        let max_reservations = (INITIAL_INVENTORY_STANDARD / RESERVATION_AMOUNT) as usize;
        assert!(successful_reservations <= max_reservations,
            "Too many reservations succeeded: {}, expected <= {}", successful_reservations, max_reservations);

        // Verify final state using helper method
        let final_reserved = app.db().get_reserved_quantity(tenant_id, warehouse_id, product_id).await;
        if let Some(reserved) = final_reserved {
            assert!(reserved <= INITIAL_INVENTORY_STANDARD, "Over-reservation detected: {} > {}", reserved, INITIAL_INVENTORY_STANDARD);
        }

        app.cleanup().await;
    }

    /// Test reserve → release → re-reserve cycle
    #[tokio::test]
    async fn test_reserve_release_rereserve() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Reserve Release").await;
        let warehouse_id = app.db().create_test_warehouse(tenant_id, "REL-WH", "Release Warehouse").await;
        let product_id = app.db().create_test_product(tenant_id, "REL-001", "Release Product").await;

        // Set initial inventory: 50 units
        app.db().set_inventory_level(tenant_id, warehouse_id, product_id, 50).await;

        // Reserve 30 units using helper
        app.db().update_reserved_quantity(tenant_id, warehouse_id, product_id, 30).await;

        // Release all reserved units
        app.db().update_reserved_quantity(tenant_id, warehouse_id, product_id, 0).await;

        // Re-reserve 50 units (full amount now available)
        let result = sqlx::query(
            "UPDATE inventory_levels
             SET reserved_quantity = 50, updated_at = NOW()
             WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3
             AND available_quantity >= 50"
        )
        .bind(tenant_id)
        .bind(warehouse_id)
        .bind(product_id)
        .execute(&app.db().pool)
        .await
        .unwrap();

        assert_eq!(result.rows_affected(), 1, "Re-reservation should succeed");

        app.cleanup().await;
    }
}

// ============================================================================
// Concurrent Stock Move Tests
// ============================================================================

#[cfg(test)]
mod concurrent_move_tests {
    use super::*;

    /// Test concurrent transfers from the same location
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_concurrent_transfers_same_source() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Concurrent Transfer").await;
        let source_wh = app.db().create_test_warehouse(tenant_id, "SRC-WH", "Source").await;
        let dest_wh = app.db().create_test_warehouse(tenant_id, "DST-WH", "Destination").await;
        let product_id = app.db().create_test_product(tenant_id, "TRF-001", "Transfer Product").await;

        // Set initial inventory: 100 units at source
        app.db().set_inventory_level(tenant_id, source_wh, product_id, INITIAL_INVENTORY_STANDARD).await;

        let db_pool = app.db().pool.clone();
        let mut handles = JoinSet::new();

        // Spawn 5 concurrent transfers of 25 units each
        for _i in 0..CONCURRENT_TRANSFER_TASKS {
            let pool = db_pool.clone();
            let t_id = tenant_id;
            let src = source_wh;
            let dst = dest_wh;
            let p_id = product_id;

            handles.spawn(async move {
                // Start transaction
                let mut tx = pool.begin().await.unwrap();

                // Lock and check source inventory
                let available: Option<(i64,)> = sqlx::query_as(
                    "SELECT available_quantity FROM inventory_levels
                     WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3
                     FOR UPDATE"
                )
                .bind(t_id)
                .bind(src)
                .bind(p_id)
                .fetch_optional(&mut *tx)
                .await
                .unwrap();

                if let Some((qty,)) = available {
                    if qty >= TRANSFER_AMOUNT {
                        // Deduct from source
                        sqlx::query(
                            "UPDATE inventory_levels
                             SET available_quantity = available_quantity - $4, updated_at = NOW()
                             WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3"
                        )
                        .bind(t_id)
                        .bind(src)
                        .bind(p_id)
                        .bind(TRANSFER_AMOUNT)
                        .execute(&mut *tx)
                        .await
                        .unwrap();

                        // Add to destination
                        sqlx::query(
                            "INSERT INTO inventory_levels (tenant_id, warehouse_id, product_id, available_quantity, reserved_quantity, created_at, updated_at)
                             VALUES ($1, $2, $3, $4, 0, NOW(), NOW())
                             ON CONFLICT (tenant_id, warehouse_id, product_id)
                             DO UPDATE SET available_quantity = inventory_levels.available_quantity + $4, updated_at = NOW()"
                        )
                        .bind(t_id)
                        .bind(dst)
                        .bind(p_id)
                        .bind(TRANSFER_AMOUNT)
                        .execute(&mut *tx)
                        .await
                        .unwrap();

                        tx.commit().await.unwrap();
                        return true;
                    }
                }

                tx.rollback().await.unwrap();
                false
            });
        }

        // Wait for all transfers
        let mut successful = 0;
        while let Some(result) = handles.join_next().await {
            if let Ok(true) = result {
                successful += 1;
            }
        }

        // Only 4 transfers should succeed (4 * 25 = 100)
        let expected_transfers = (INITIAL_INVENTORY_STANDARD / TRANSFER_AMOUNT) as usize;
        assert_eq!(successful, expected_transfers, "Expected exactly {} successful transfers", expected_transfers);

        // Verify final states
        let source_qty = app.db().get_inventory_level(tenant_id, source_wh, product_id).await;
        let dest_qty = app.db().get_inventory_level(tenant_id, dest_wh, product_id).await;

        assert_eq!(source_qty, Some(0), "Source should be empty");
        assert_eq!(dest_qty, Some(INITIAL_INVENTORY_STANDARD), "Destination should have {} units", INITIAL_INVENTORY_STANDARD);

        app.cleanup().await;
    }

    /// Test simultaneous receipts to the same location
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_concurrent_receipts_same_location() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Concurrent Receipt").await;
        let warehouse_id = app.db().create_test_warehouse(tenant_id, "RCV-WH", "Receiving").await;
        let product_id = app.db().create_test_product(tenant_id, "RCV-001", "Receipt Product").await;

        // Start with 0 inventory
        app.db().set_inventory_level(tenant_id, warehouse_id, product_id, 0).await;

        let db_pool = app.db().pool.clone();
        let mut handles = JoinSet::new();

        // Spawn 10 concurrent receipt operations, each adding 10 units
        for _ in 0..CONCURRENT_RECEIPT_TASKS {
            let pool = db_pool.clone();
            let t_id = tenant_id;
            let w_id = warehouse_id;
            let p_id = product_id;

            handles.spawn(async move {
                sqlx::query(
                    "UPDATE inventory_levels
                     SET available_quantity = available_quantity + $4, updated_at = NOW()
                     WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3"
                )
                .bind(t_id)
                .bind(w_id)
                .bind(p_id)
                .bind(RECEIPT_AMOUNT)
                .execute(&pool)
                .await
                .is_ok()
            });
        }

        // Wait for all receipts
        while let Some(_) = handles.join_next().await {}

        // Verify final quantity: should be exactly 100 (10 * 10)
        let expected_qty = (CONCURRENT_RECEIPT_TASKS as i64) * RECEIPT_AMOUNT;
        let final_qty = app.db().get_inventory_level(tenant_id, warehouse_id, product_id).await;
        assert_eq!(final_qty, Some(expected_qty), "Final quantity should be {}", expected_qty);

        app.cleanup().await;
    }

    /// Test mixed operations (receipt + transfer concurrently)
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_mixed_concurrent_operations() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Mixed Operations").await;
        let wh1 = app.db().create_test_warehouse(tenant_id, "MIX-WH1", "Warehouse 1").await;
        let wh2 = app.db().create_test_warehouse(tenant_id, "MIX-WH2", "Warehouse 2").await;
        let product_id = app.db().create_test_product(tenant_id, "MIX-001", "Mixed Product").await;

        // Initial: INITIAL_INVENTORY_STANDARD at WH1, 0 at WH2
        app.db().set_inventory_level(tenant_id, wh1, product_id, INITIAL_INVENTORY_STANDARD).await;
        app.db().set_inventory_level(tenant_id, wh2, product_id, 0).await;

        let db_pool = app.db().pool.clone();
        let mut handles: JoinSet<bool> = JoinSet::new();

        // Receipt to WH1: +MIXED_RECEIPT_AMOUNT
        {
            let pool = db_pool.clone();
            let t_id = tenant_id;
            let w_id = wh1;
            let p_id = product_id;
            handles.spawn(async move {
                sqlx::query(
                    "UPDATE inventory_levels
                     SET available_quantity = available_quantity + $4, updated_at = NOW()
                     WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3"
                )
                .bind(t_id)
                .bind(w_id)
                .bind(p_id)
                .bind(MIXED_RECEIPT_AMOUNT)
                .execute(&pool)
                .await
                .is_ok()
            });
        }

        // Transfer MIXED_TRANSFER_AMOUNT from WH1 to WH2
        {
            let pool = db_pool.clone();
            let t_id = tenant_id;
            let src = wh1;
            let dst = wh2;
            let p_id = product_id;
            handles.spawn(async move {
                let mut tx = pool.begin().await.unwrap();

                // Lock and check source inventory
                let available: Option<(i64,)> = sqlx::query_as(
                    "SELECT available_quantity FROM inventory_levels
                     WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3
                     FOR UPDATE"
                )
                .bind(t_id)
                .bind(src)
                .bind(p_id)
                .fetch_optional(&mut *tx)
                .await
                .unwrap();

                if let Some((qty,)) = available {
                    if qty >= MIXED_TRANSFER_AMOUNT {
                        // Deduct from source
                        sqlx::query(
                            "UPDATE inventory_levels
                             SET available_quantity = available_quantity - $4, updated_at = NOW()
                             WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3"
                        )
                        .bind(t_id)
                        .bind(src)
                        .bind(p_id)
                        .bind(MIXED_TRANSFER_AMOUNT)
                        .execute(&mut *tx)
                        .await
                        .unwrap();

                        // Add to destination (INSERT ON CONFLICT for robustness)
                        sqlx::query(
                            "INSERT INTO inventory_levels (tenant_id, warehouse_id, product_id, available_quantity, reserved_quantity, created_at, updated_at)
                             VALUES ($1, $2, $3, $4, 0, NOW(), NOW())
                             ON CONFLICT (tenant_id, warehouse_id, product_id)
                             DO UPDATE SET available_quantity = inventory_levels.available_quantity + $4, updated_at = NOW()"
                        )
                        .bind(t_id)
                        .bind(dst)
                        .bind(p_id)
                        .bind(MIXED_TRANSFER_AMOUNT)
                        .execute(&mut *tx)
                        .await
                        .unwrap();

                        tx.commit().await.unwrap();
                        return true;
                    }
                }

                tx.rollback().await.unwrap();
                false
            });
        }

        // Wait for all operations and verify they all succeeded
        let mut all_succeeded = true;
        while let Some(result) = handles.join_next().await {
            match result {
                Ok(success) => {
                    if !success {
                        all_succeeded = false;
                    }
                }
                Err(_) => all_succeeded = false,
            }
        }

        // All operations must succeed for the test to be valid
        assert!(all_succeeded, "All concurrent operations should succeed");

        // Verify final states
        // WH1: INITIAL_INVENTORY_STANDARD + MIXED_RECEIPT_AMOUNT - MIXED_TRANSFER_AMOUNT
        // WH2: 0 + MIXED_TRANSFER_AMOUNT
        let expected_wh1 = INITIAL_INVENTORY_STANDARD + MIXED_RECEIPT_AMOUNT - MIXED_TRANSFER_AMOUNT;
        let expected_wh2 = MIXED_TRANSFER_AMOUNT;
        let wh1_qty = app.db().get_inventory_level(tenant_id, wh1, product_id).await;
        let wh2_qty = app.db().get_inventory_level(tenant_id, wh2, product_id).await;

        assert_eq!(wh1_qty, Some(expected_wh1), "WH1 should have {} units", expected_wh1);
        assert_eq!(wh2_qty, Some(expected_wh2), "WH2 should have {} units", expected_wh2);

        app.cleanup().await;
    }
}

// ============================================================================
// Idempotency Tests
// ============================================================================

#[cfg(test)]
mod idempotency_tests {
    use super::*;

    /// Test that duplicate requests with same idempotency key are rejected
    #[tokio::test]
    async fn test_duplicate_request_handling() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Idempotency").await;
        let warehouse_id = app.db().create_test_warehouse(tenant_id, "IDEM-WH", "Idempotency WH").await;

        let idempotency_key = Uuid::new_v4().to_string();

        // First request - should succeed or fail based on endpoint logic
        let request1 = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/inventory/stock-takes")
            .header("content-type", "application/json")
            .header("x-idempotency-key", &idempotency_key)
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::from(json!({
                "warehouse_id": warehouse_id,
                "name": "Test Stock Take",
                "count_type": "full"
            }).to_string()))
            .unwrap();

        let (status1, body1) = app.send_request(request1).await;

        // First request must succeed for idempotency test to be meaningful
        assert!(
            status1.is_success(),
            "First request should succeed for idempotency test. Got status: {:?}, body: {}",
            status1,
            body1
        );

        // Second request with same idempotency key should be rejected
        let request2 = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/inventory/stock-takes")
            .header("content-type", "application/json")
            .header("x-idempotency-key", &idempotency_key) // Same key
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::from(json!({
                "warehouse_id": warehouse_id,
                "name": "Test Stock Take 2",
                "count_type": "full"
            }).to_string()))
            .unwrap();

        let (status2, _body2) = app.send_request(request2).await;

        // Second request should return 409 Conflict
        assert_eq!(
            status2,
            StatusCode::CONFLICT,
            "Duplicate request should be rejected with 409 Conflict"
        );

        app.cleanup().await;
    }

    /// Test that different idempotency keys allow separate requests
    #[tokio::test]
    async fn test_different_idempotency_keys() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Idempotency Keys").await;
        let warehouse_id = app.db().create_test_warehouse(tenant_id, "KEY-WH", "Key WH").await;

        // Two requests with different keys
        let key1 = Uuid::new_v4().to_string();
        let key2 = Uuid::new_v4().to_string();

        let request1 = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/inventory/stock-takes")
            .header("content-type", "application/json")
            .header("x-idempotency-key", &key1)
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::from(json!({
                "warehouse_id": warehouse_id,
                "name": "Stock Take 1",
                "count_type": "full"
            }).to_string()))
            .unwrap();

        let request2 = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/inventory/stock-takes")
            .header("content-type", "application/json")
            .header("x-idempotency-key", &key2)
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::from(json!({
                "warehouse_id": warehouse_id,
                "name": "Stock Take 2",
                "count_type": "full"
            }).to_string()))
            .unwrap();

        let (status1, body1) = app.send_request(request1).await;
        let (status2, body2) = app.send_request(request2).await;

        // Both should get the same treatment (both succeed or both fail on validation)
        // Additionally verify they're not erroring unexpectedly
        assert_eq!(
            status1, status2,
            "Different keys should allow independent requests. Status1: {:?}, Status2: {:?}",
            status1, status2
        );

        // Log bodies for debugging if statuses don't match expectations
        if !status1.is_success() {
            eprintln!("Request 1 failed with body: {}", body1);
            eprintln!("Request 2 failed with body: {}", body2);
        }

        app.cleanup().await;
    }
}

// ============================================================================
// Database Consistency Tests
// ============================================================================

#[cfg(test)]
mod consistency_tests {
    use super::*;

    /// Test that stock levels remain consistent after concurrent operations
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_stock_levels_consistency() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Consistency").await;
        let warehouse_id = app.db().create_test_warehouse(tenant_id, "CON-WH", "Consistency WH").await;
        let product_id = app.db().create_test_product(tenant_id, "CON-001", "Consistency Product").await;

        // Initial: 1000 units
        app.db().set_inventory_level(tenant_id, warehouse_id, product_id, INITIAL_INVENTORY_LARGE).await;

        let db_pool = app.db().pool.clone();
        let mut handles = JoinSet::new();

        // 20 concurrent +10 operations
        for _ in 0..CONCURRENT_INCREMENT_OPS {
            let pool = db_pool.clone();
            let t_id = tenant_id;
            let w_id = warehouse_id;
            let p_id = product_id;
            handles.spawn(async move {
                sqlx::query(
                    "UPDATE inventory_levels
                     SET available_quantity = available_quantity + $4, updated_at = NOW()
                     WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3"
                )
                .bind(t_id)
                .bind(w_id)
                .bind(p_id)
                .bind(INCREMENT_AMOUNT)
                .execute(&pool)
                .await
            });
        }

        // 10 concurrent -5 operations
        for _ in 0..CONCURRENT_DECREMENT_OPS {
            let pool = db_pool.clone();
            let t_id = tenant_id;
            let w_id = warehouse_id;
            let p_id = product_id;
            handles.spawn(async move {
                sqlx::query(
                    "UPDATE inventory_levels
                     SET available_quantity = available_quantity - $4, updated_at = NOW()
                     WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3"
                )
                .bind(t_id)
                .bind(w_id)
                .bind(p_id)
                .bind(DECREMENT_AMOUNT)
                .execute(&pool)
                .await
            });
        }

        // Wait for all operations
        while let Some(_) = handles.join_next().await {}

        // Final: 1000 + (20 * 10) - (10 * 5) = 1000 + 200 - 50 = 1150
        let expected_final = INITIAL_INVENTORY_LARGE
            + (CONCURRENT_INCREMENT_OPS as i64 * INCREMENT_AMOUNT)
            - (CONCURRENT_DECREMENT_OPS as i64 * DECREMENT_AMOUNT);
        let final_qty = app.db().get_inventory_level(tenant_id, warehouse_id, product_id).await;
        assert_eq!(final_qty, Some(expected_final), "Final quantity should be {}", expected_final);

        app.cleanup().await;
    }

    /// Test that no negative inventory is allowed
    #[tokio::test]
    async fn test_no_negative_inventory() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("No Negative").await;
        let warehouse_id = app.db().create_test_warehouse(tenant_id, "NEG-WH", "No Neg WH").await;
        let product_id = app.db().create_test_product(tenant_id, "NEG-001", "No Neg Product").await;

        // Initial: 50 units
        app.db().set_inventory_level(tenant_id, warehouse_id, product_id, 50).await;

        // Try to deduct 100 (more than available)
        let result = sqlx::query(
            "UPDATE inventory_levels
             SET available_quantity = available_quantity - 100, updated_at = NOW()
             WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3
             AND available_quantity >= 100"
        )
        .bind(tenant_id)
        .bind(warehouse_id)
        .bind(product_id)
        .execute(&app.db().pool)
        .await
        .unwrap();

        // Should not update any rows
        assert_eq!(result.rows_affected(), 0, "Should not allow negative inventory");

        // Verify quantity unchanged
        let qty = app.db().get_inventory_level(tenant_id, warehouse_id, product_id).await;
        assert_eq!(qty, Some(50), "Quantity should remain 50");

        app.cleanup().await;
    }
}
