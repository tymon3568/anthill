//! Concurrent stock operations tests
//!
//! Tests for concurrent stock operations to verify database locking,
//! transaction isolation, and race condition prevention using tokio async tasks.

#![allow(dead_code, clippy::unnecessary_unwrap)]

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
use std::time::Duration;
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
#[allow(dead_code)]
/// Timeout for JoinSet wait loops to prevent CI hanging on deadlock/stall
const JOINSET_TIMEOUT_SECS: u64 = 30;

// ============================================================================
// Test Configuration
// ============================================================================

/// Check if integration tests should run.
/// Returns Some(database_url) if tests should run, None if they should skip.
///
/// Integration tests require either:
/// - RUN_INVENTORY_INTEGRATION_TESTS=1 to be set, OR
/// - DATABASE_URL to be explicitly set
///
/// When skipping, a clear message is printed so CI shows tests were intentionally skipped.
fn should_run_integration_tests() -> Option<String> {
    // Check for explicit opt-in
    let opt_in = std::env::var("RUN_INVENTORY_INTEGRATION_TESTS")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false);

    // Check if DATABASE_URL is explicitly set (not using fallback)
    let db_url = std::env::var("DATABASE_URL").ok();

    if opt_in || db_url.is_some() {
        // Tests should run - return the database URL
        Some(db_url.unwrap_or_else(|| {
            panic!(
                "RUN_INVENTORY_INTEGRATION_TESTS=1 is set but DATABASE_URL is not. \
                 Please set DATABASE_URL to run integration tests."
            )
        }))
    } else {
        // Skip tests with clear message
        eprintln!(
            "SKIPPING integration tests: Set RUN_INVENTORY_INTEGRATION_TESTS=1 and DATABASE_URL to run. \
             This is expected in CI when database services are not available."
        );
        None
    }
}

fn test_config() -> Config {
    // This should only be called after should_run_integration_tests() returns Some
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set to run integration tests");

    let redis_url = std::env::var("REDIS_URL").ok();
    let kanidm_url = std::env::var("KANIDM_URL").ok();
    let kanidm_client_id = std::env::var("KANIDM_CLIENT_ID").ok();
    let kanidm_client_secret = std::env::var("KANIDM_CLIENT_SECRET").ok();
    let kanidm_redirect_url = std::env::var("KANIDM_REDIRECT_URL").ok();

    Config {
        database_url,
        jwt_secret: "test-secret-key-at-least-32-characters-long".to_string(),
        jwt_expiration: 900,
        jwt_refresh_expiration: 604800,
        host: "0.0.0.0".to_string(),
        port: 8001,
        cors_origins: None,
        kanidm_url,
        kanidm_client_id,
        kanidm_client_secret,
        kanidm_redirect_url,
        nats_url: None,
        redis_url,
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
    /// Create a new test database connection.
    /// Returns None if integration tests should be skipped.
    async fn try_new() -> Option<Self> {
        let database_url = should_run_integration_tests()?;

        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        Some(Self {
            pool,
            test_tenants: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Create a new test database connection.
    /// Panics if DATABASE_URL is not set. Use try_new() for graceful skip.
    async fn new() -> Self {
        Self::try_new()
            .await
            .expect("Integration tests require DATABASE_URL to be set")
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
            "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, warehouse_type, is_active, created_at, updated_at)
             VALUES ($1, $2, $3, $4, 'main', true, NOW(), NOW())"
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

    async fn create_test_user(&self, tenant_id: Uuid, _name: &str) -> Uuid {
        let user_id = Uuid::now_v7();
        let email = format!("test-{}@example.com", user_id);

        sqlx::query(
            "INSERT INTO users (user_id, tenant_id, email, email_verified, role, status, failed_login_attempts, auth_method, created_at, updated_at)
             VALUES ($1, $2, $3, true, 'admin', 'active', 0, 'password', NOW(), NOW())"
        )
        .bind(user_id)
        .bind(tenant_id)
        .bind(email)
        .execute(&self.pool)
        .await
        .expect("Failed to create test user");

        // Add Casbin Role (g policy)
        sqlx::query(
            "INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, created_at) VALUES ('g', $1, 'admin', $2, '', NOW())"
        )
        .bind(user_id.to_string())
        .bind(tenant_id.to_string())
        .execute(&self.pool)
        .await
        .expect("Failed to add casbin role");

        // Add Casbin Permission (p policy) - Admin can POST to inventory
        sqlx::query(
            "INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, created_at) VALUES ('p', 'admin', $1, '/api/v1/inventory/categories', 'POST', NOW())"
        )
        .bind(tenant_id.to_string())
        .execute(&self.pool)
        .await
        .expect("Failed to add casbin permission");

        user_id
    }

    async fn create_test_location(&self, tenant_id: Uuid, warehouse_id: Uuid, zone_name: &str, code: &str, user_id: Uuid) -> Uuid {
        let location_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO storage_locations (location_id, tenant_id, warehouse_id, zone, location_code, location_type, is_active, created_by, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, 'standard', true, $6, NOW(), NOW())"
        )
        .bind(location_id)
        .bind(tenant_id)
        .bind(warehouse_id)
        .bind(zone_name)
        .bind(code)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .expect("Failed to create test location");
        location_id
    }

    async fn set_inventory_level(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        location_id: Uuid,
        product_id: Uuid,
        quantity: i64,
    ) {
        sqlx::query(
            "INSERT INTO inventory_levels (tenant_id, warehouse_id, location_id, product_id, available_quantity, reserved_quantity, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, 0, NOW(), NOW())
             ON CONFLICT (tenant_id, warehouse_id, location_id, product_id)
             DO UPDATE SET available_quantity = $5, updated_at = NOW()"
        )
        .bind(tenant_id)
        .bind(warehouse_id)
        .bind(location_id)
        .bind(product_id)
        .bind(quantity)
        .execute(&self.pool)
        .await
        .expect("Failed to set inventory level");
    }

    async fn get_inventory_level(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
    ) -> Option<i64> {
        let result: Option<(i64,)> = sqlx::query_as(
            "SELECT available_quantity FROM inventory_levels
             WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3",
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
    async fn get_reserved_quantity(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
    ) -> Option<i64> {
        let result: Option<(i64,)> = sqlx::query_as(
            "SELECT reserved_quantity FROM inventory_levels
             WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3",
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
    async fn update_reserved_quantity(
        &self,
        tenant_id: Uuid,
        warehouse_id: Uuid,
        product_id: Uuid,
        quantity: i64,
    ) {
        sqlx::query(
            "UPDATE inventory_levels
             SET reserved_quantity = $4, updated_at = NOW()
             WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3",
        )
        .bind(tenant_id)
        .bind(warehouse_id)
        .bind(product_id)
        .bind(quantity)
        .execute(&self.pool)
        .await
        .expect("Failed to update reserved quantity");
    }

    fn upsert_inventory_sql() -> &'static str {
        "INSERT INTO inventory_levels (tenant_id, warehouse_id, location_id, product_id, available_quantity, reserved_quantity, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, 0, NOW(), NOW())
         ON CONFLICT (tenant_id, warehouse_id, location_id, product_id)
         DO UPDATE SET available_quantity = inventory_levels.available_quantity + $5, updated_at = NOW()"
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
    /// Create a new test app. Returns None if integration tests should be skipped.
    async fn try_new() -> Option<Self> {
        let db = Self::init_db().await?;
        Some(Self::build_app(db).await)
    }

    async fn init_db() -> Option<TestDatabase> {
        TestDatabase::try_new().await
    }

    async fn build_app(db: TestDatabase) -> Self {
        let config = test_config();
        let router = create_app(config).await;
        Self { router, db }
    }

    async fn new() -> Self {
        Self::try_new()
            .await
            .expect("Integration tests require DATABASE_URL to be set")
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
    use shared_jwt::{encode_jwt, Claims};

    let jwt_secret = "test-secret-key-at-least-32-characters-long";

    let claims = Claims::new_access(user_id, tenant_id, "admin".to_string(), 3600);
    let token = encode_jwt(&claims, &jwt_secret).expect("Failed to encode JWT token for test");

    format!("Bearer {}", token)
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
        let warehouse_id = app
            .db()
            .create_test_warehouse(tenant_id, "RES-WH", "Reservation Warehouse")
            .await;
        let product_id = app
            .db()
            .create_test_product(tenant_id, "RES-001", "Reservation Product")
            .await;

        let user_id = app.db().create_test_user(tenant_id, "res_user").await;
        let location_id = app.db().create_test_location(tenant_id, warehouse_id, "Z1", "L1", user_id).await;

        // Set initial inventory: 100 units
        app.db()
            .set_inventory_level(tenant_id, warehouse_id, location_id, product_id, INITIAL_INVENTORY_STANDARD)
            .await;

        // Spawn 10 concurrent tasks, each trying to reserve 15 units
        // Total requested: 150 units, but only 100 available
        let db_pool = app.db().pool.clone();
        let mut handles = JoinSet::new();

        for _i in 0..CONCURRENT_RESERVATION_TASKS {
            let pool = db_pool.clone();
            let t_id = tenant_id;
            let w_id = warehouse_id;
            let l_id = location_id;
            let p_id = product_id;

            handles.spawn(async move {
                // Simulate reservation by updating reserved_quantity
                // Uses conditional UPDATE to prevent race conditions
                let result = sqlx::query(
                    "UPDATE inventory_levels
                     SET reserved_quantity = reserved_quantity + $5,
                         updated_at = NOW()
                     WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4
                     AND available_quantity - reserved_quantity >= $5"
                )
                .bind(t_id)
                .bind(w_id)
                .bind(l_id)
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

        // Wait for all tasks with timeout to prevent CI hanging
        let mut successful_reservations = 0;
        let wait_result = tokio::time::timeout(Duration::from_secs(JOINSET_TIMEOUT_SECS), async {
            while let Some(result) = handles.join_next().await {
                let success = result.expect("reservation task panicked/cancelled");
                if success {
                    successful_reservations += 1;
                }
            }
        })
        .await;

        if wait_result.is_err() {
            handles.shutdown().await;
            panic!("Test timed out after {}s - possible deadlock or stall", JOINSET_TIMEOUT_SECS);
        }

        // Should only allow 6 successful reservations (6 * 15 = 90 <= 100)
        let max_reservations = (INITIAL_INVENTORY_STANDARD / RESERVATION_AMOUNT) as usize;
        assert!(
            successful_reservations <= max_reservations,
            "Too many reservations succeeded: {}, expected <= {}",
            successful_reservations,
            max_reservations
        );

        // Verify final state using helper method
        // Note: get_reserved_quantity needs update too, skipping implies manual check or update...
        // For now, let's update the manual check if helper is not updated.
        // Wait, I didn't update get_reserved_quantity helper!
        // I should stick to updating SQL here or update helper separately.
        // Let's assume helper needs update. But I only edited set_inventory_level.
        // I will do raw SQL check here to be safe and avoid spiral of helper edits.

        let final_reserved: Option<i64> = sqlx::query_scalar(
            "SELECT reserved_quantity FROM inventory_levels
             WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4",
        )
        .bind(tenant_id)
        .bind(warehouse_id)
        .bind(location_id)
        .bind(product_id)
        .fetch_optional(&app.db().pool)
        .await
        .unwrap();

        if let Some(reserved) = final_reserved {
            assert!(
                reserved <= INITIAL_INVENTORY_STANDARD,
                "Over-reservation detected: {} > {}",
                reserved,
                INITIAL_INVENTORY_STANDARD
            );
        }

        app.cleanup().await;
    }

    /// Test reserve → release → re-reserve cycle
    #[tokio::test]
    async fn test_reserve_release_rereserve() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Reserve Release").await;
        let warehouse_id = app
            .db()
            .create_test_warehouse(tenant_id, "REL-WH", "Release Warehouse")
            .await;
        let product_id = app
            .db()
            .create_test_product(tenant_id, "REL-001", "Release Product")
            .await;

        let user_id = app.db().create_test_user(tenant_id, "rel_user").await;
        let location_id = app.db().create_test_location(tenant_id, warehouse_id, "Z2", "L2", user_id).await;

        // Set initial inventory: 50 units
        app.db()
            .set_inventory_level(tenant_id, warehouse_id, location_id, product_id, 50)
            .await;

        // Reserve 30 units using raw SQL since helper is outdated
        sqlx::query(
            "UPDATE inventory_levels
             SET reserved_quantity = $5, updated_at = NOW()
             WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4",
        )
        .bind(tenant_id)
        .bind(warehouse_id)
        .bind(location_id)
        .bind(product_id)
        .bind(30)
        .execute(&app.db().pool)
        .await
        .unwrap();

        // Release all reserved units
        sqlx::query(
            "UPDATE inventory_levels
             SET reserved_quantity = $5, updated_at = NOW()
             WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4",
        )
        .bind(tenant_id)
        .bind(warehouse_id)
        .bind(location_id)
        .bind(product_id)
        .bind(0)
        .execute(&app.db().pool)
        .await
        .unwrap();

        // Re-reserve 50 units (full amount now available)
        let result = sqlx::query(
            "UPDATE inventory_levels
             SET reserved_quantity = 50, updated_at = NOW()
             WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4
             AND available_quantity >= 50",
        )
        .bind(tenant_id)
        .bind(warehouse_id)
        .bind(location_id)
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
        let source_wh = app
            .db()
            .create_test_warehouse(tenant_id, "SRC-WH", "Source")
            .await;
        let dest_wh = app
            .db()
            .create_test_warehouse(tenant_id, "DST-WH", "Destination")
            .await;
        let product_id = app
            .db()
            .create_test_product(tenant_id, "TRF-001", "Transfer Product")
            .await;

        let user_id = app.db().create_test_user(tenant_id, "trf_user").await;
        let loc_src = app.db().create_test_location(tenant_id, source_wh, "Z_SRC", "L_SRC", user_id).await;
        let loc_dst = app.db().create_test_location(tenant_id, dest_wh, "Z_DST", "L_DST", user_id).await;

        // Set initial inventory: 100 units at source
        app.db()
            .set_inventory_level(tenant_id, source_wh, loc_src, product_id, INITIAL_INVENTORY_STANDARD)
            .await;

        let db_pool = app.db().pool.clone();
        let mut handles = JoinSet::new();

        // Spawn 5 concurrent transfers of 25 units each
        for _i in 0..CONCURRENT_TRANSFER_TASKS {
            let pool = db_pool.clone();
            let t_id = tenant_id;
            let src = source_wh;
            let dst = dest_wh;
            let l_src = loc_src;
            let l_dst = loc_dst;
            let p_id = product_id;

            handles.spawn(async move {
                // Start transaction
                let mut tx = pool.begin().await.unwrap();

                // Lock and check source inventory
                let available: Option<(i64,)> = sqlx::query_as(
                    "SELECT available_quantity FROM inventory_levels
                     WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4
                     FOR UPDATE",
                )
                .bind(t_id)
                .bind(src)
                .bind(l_src)
                .bind(p_id)
                .fetch_optional(&mut *tx)
                .await
                .unwrap();

                if let Some((qty,)) = available {
                    if qty >= TRANSFER_AMOUNT {
                        // Deduct from source
                        sqlx::query(
                            "UPDATE inventory_levels
                             SET available_quantity = available_quantity - $5, updated_at = NOW()
                             WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4",
                        )
                        .bind(t_id)
                        .bind(src)
                        .bind(l_src)
                        .bind(p_id)
                        .bind(TRANSFER_AMOUNT)
                        .execute(&mut *tx)
                        .await
                        .unwrap();

                        // Add to destination
                        sqlx::query(TestDatabase::upsert_inventory_sql())
                            .bind(t_id)
                            .bind(dst)
                            .bind(l_dst)
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

        // Wait for all transfers with timeout to prevent CI hanging
        let mut successful = 0;
        let wait_result = tokio::time::timeout(Duration::from_secs(JOINSET_TIMEOUT_SECS), async {
            while let Some(result) = handles.join_next().await {
                let ok = result.expect("transfer task panicked/cancelled");
                if ok {
                    successful += 1;
                }
            }
        })
        .await;

        if wait_result.is_err() {
            handles.shutdown().await;
            panic!("Test timed out after {}s - possible deadlock or stall", JOINSET_TIMEOUT_SECS);
        }

        // Only 4 transfers should succeed (4 * 25 = 100)
        let expected_transfers = (INITIAL_INVENTORY_STANDARD / TRANSFER_AMOUNT) as usize;
        assert_eq!(
            successful, expected_transfers,
            "Expected exactly {} successful transfers",
            expected_transfers
        );

        // Verify final states
        let source_qty: Option<i64> = sqlx::query_scalar(
            "SELECT available_quantity FROM inventory_levels
             WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4"
        )
        .bind(tenant_id)
        .bind(source_wh)
        .bind(loc_src)
        .bind(product_id)
        .fetch_optional(&app.db().pool)
        .await
        .unwrap();

        let dest_qty: Option<i64> = sqlx::query_scalar(
            "SELECT available_quantity FROM inventory_levels
             WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4"
        )
        .bind(tenant_id)
        .bind(dest_wh)
        .bind(loc_dst)
        .bind(product_id)
        .fetch_optional(&app.db().pool)
        .await
        .unwrap();

        assert_eq!(source_qty, Some(0), "Source should be empty");
        assert_eq!(
            dest_qty,
            Some(INITIAL_INVENTORY_STANDARD),
            "Destination should have {} units",
            INITIAL_INVENTORY_STANDARD
        );

        app.cleanup().await;
    }

    /// Test simultaneous receipts to the same location
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_concurrent_receipts_same_location() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Concurrent Receipt").await;
        let warehouse_id = app
            .db()
            .create_test_warehouse(tenant_id, "RCV-WH", "Receiving")
            .await;
        let product_id = app
            .db()
            .create_test_product(tenant_id, "RCV-001", "Receipt Product")
            .await;

        let user_id = app.db().create_test_user(tenant_id, "rcv_user").await;
        let location_id = app.db().create_test_location(tenant_id, warehouse_id, "Z_RCV", "L_RCV", user_id).await;

        // Start with 0 inventory
        app.db()
            .set_inventory_level(tenant_id, warehouse_id, location_id, product_id, 0)
            .await;

        let db_pool = app.db().pool.clone();
        let mut handles = JoinSet::new();

        // Spawn 10 concurrent receipt operations, each adding 10 units
        for _ in 0..CONCURRENT_RECEIPT_TASKS {
            let pool = db_pool.clone();
            let t_id = tenant_id;
            let w_id = warehouse_id;
            let l_id = location_id;
            let p_id = product_id;

            handles.spawn(async move {
                sqlx::query(
                    "UPDATE inventory_levels
                     SET available_quantity = available_quantity + $5, updated_at = NOW()
                     WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4",
                )
                .bind(t_id)
                .bind(w_id)
                .bind(l_id)
                .bind(p_id)
                .bind(RECEIPT_AMOUNT)
                .execute(&pool)
                .await
                .is_ok()
            });
        }

        // Wait for all receipts with timeout to prevent CI hanging
        let mut failure_count = 0;
        let wait_result = tokio::time::timeout(Duration::from_secs(JOINSET_TIMEOUT_SECS), async {
            while let Some(join_result) = handles.join_next().await {
                match join_result {
                    Ok(task_ok) => {
                        if !task_ok {
                            failure_count += 1;
                        }
                    },
                    Err(join_error) => {
                        eprintln!("join error in concurrent receipt task: {join_error}");
                        failure_count += 1;
                    },
                }
            }
        })
        .await;

        if wait_result.is_err() {
            handles.shutdown().await;
            panic!("Test timed out after {}s - possible deadlock or stall", JOINSET_TIMEOUT_SECS);
        }
        assert_eq!(failure_count, 0, "All concurrent receipt tasks must succeed");

        // Verify final quantity: should be exactly 100 (10 * 10)
        let expected_qty = (CONCURRENT_RECEIPT_TASKS as i64) * RECEIPT_AMOUNT;
        let final_qty: Option<i64> = sqlx::query_scalar(
            "SELECT available_quantity FROM inventory_levels
             WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4"
        )
        .bind(tenant_id)
        .bind(warehouse_id)
        .bind(location_id)
        .bind(product_id)
        .fetch_optional(&app.db().pool)
        .await
        .unwrap();
        assert_eq!(final_qty, Some(expected_qty), "Final quantity should be {}", expected_qty);

        app.cleanup().await;
    }

    /// Test mixed operations (receipt + transfer concurrently)
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_mixed_concurrent_operations() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Mixed Operations").await;
        let wh1 = app
            .db()
            .create_test_warehouse(tenant_id, "MIX-WH1", "Warehouse 1")
            .await;
        let wh2 = app
            .db()
            .create_test_warehouse(tenant_id, "MIX-WH2", "Warehouse 2")
            .await;
        let product_id = app
            .db()
            .create_test_product(tenant_id, "MIX-001", "Mixed Product")
            .await;

        let user_id = app.db().create_test_user(tenant_id, "mix_user").await;
        let loc1 = app.db().create_test_location(tenant_id, wh1, "Z_MIX1", "L_MIX1", user_id).await;
        let loc2 = app.db().create_test_location(tenant_id, wh2, "Z_MIX2", "L_MIX2", user_id).await;

        // Initial: INITIAL_INVENTORY_STANDARD at WH1, 0 at WH2
        app.db()
            .set_inventory_level(tenant_id, wh1, loc1, product_id, INITIAL_INVENTORY_STANDARD)
            .await;
        app.db()
            .set_inventory_level(tenant_id, wh2, loc2, product_id, 0)
            .await;

        let db_pool = app.db().pool.clone();
        let mut handles: JoinSet<bool> = JoinSet::new();

        // Receipt to WH1: +MIXED_RECEIPT_AMOUNT
        {
            let pool = db_pool.clone();
            let t_id = tenant_id;
            let w_id = wh1;
            let l_id = loc1;
            let p_id = product_id;
            handles.spawn(async move {
                sqlx::query(
                    "UPDATE inventory_levels
                     SET available_quantity = available_quantity + $5, updated_at = NOW()
                     WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4",
                )
                .bind(t_id)
                .bind(w_id)
                .bind(l_id)
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
            let l_src = loc1;
            let l_dst = loc2;
            let p_id = product_id;
            handles.spawn(async move {
                let mut tx = pool.begin().await.unwrap();

                // Lock and check source inventory
                let available: Option<(i64,)> = sqlx::query_as(
                    "SELECT available_quantity FROM inventory_levels
                     WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4
                     FOR UPDATE",
                )
                .bind(t_id)
                .bind(src)
                .bind(l_src)
                .bind(p_id)
                .fetch_optional(&mut *tx)
                .await
                .unwrap();

                if let Some((qty,)) = available {
                    if qty >= MIXED_TRANSFER_AMOUNT {
                        // Deduct from source
                        sqlx::query(
                            "UPDATE inventory_levels
                             SET available_quantity = available_quantity - $5, updated_at = NOW()
                             WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4",
                        )
                        .bind(t_id)
                        .bind(src)
                        .bind(l_src)
                        .bind(p_id)
                        .bind(MIXED_TRANSFER_AMOUNT)
                        .execute(&mut *tx)
                        .await
                        .unwrap();

                        // Add to destination (INSERT ON CONFLICT for robustness)
                        sqlx::query(TestDatabase::upsert_inventory_sql())
                            .bind(t_id)
                            .bind(dst)
                            .bind(l_dst)
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

        // Wait for all operations with timeout to prevent CI hanging
        let mut all_succeeded = true;
        let wait_result = tokio::time::timeout(Duration::from_secs(JOINSET_TIMEOUT_SECS), async {
            while let Some(result) = handles.join_next().await {
                match result {
                    Ok(success) => {
                        if !success {
                            all_succeeded = false;
                        }
                    },
                    Err(_) => all_succeeded = false,
                }
            }
        })
        .await;

        if wait_result.is_err() {
            handles.shutdown().await;
            panic!("Test timed out after {}s - possible deadlock or stall", JOINSET_TIMEOUT_SECS);
        }

        // All operations must succeed for the test to be valid
        assert!(all_succeeded, "All concurrent operations should succeed");

        // Verify final states
        // WH1: INITIAL_INVENTORY_STANDARD + MIXED_RECEIPT_AMOUNT - MIXED_TRANSFER_AMOUNT
        // WH2: 0 + MIXED_TRANSFER_AMOUNT
        let expected_wh1 =
            INITIAL_INVENTORY_STANDARD + MIXED_RECEIPT_AMOUNT - MIXED_TRANSFER_AMOUNT;
        let expected_wh2 = MIXED_TRANSFER_AMOUNT;
        let wh1_qty = app
            .db()
            .get_inventory_level(tenant_id, wh1, product_id)
            .await;
        let wh2_qty = app
            .db()
            .get_inventory_level(tenant_id, wh2, product_id)
            .await;

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

    async fn make_idempotency_request(
        app: &TestApp,
        tenant_id: Uuid,
        user_id: Uuid,
        idempotency_key: &str,
        name_suffix: &str,
    ) -> (StatusCode, String) {
        let payload = json!({
            "name": format!("Start Cat {}", name_suffix),
            "description": "Idempotency test category",
            "code": format!("IDEMP-{}", name_suffix),
            "display_order": 1,
            "is_active": true
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/inventory/categories")
            .header("content-type", "application/json")
            .header("x-idempotency-key", idempotency_key)
            .header("authorization", create_auth_header(tenant_id, user_id))
            .body(Body::from(payload.to_string()))
            .expect("Failed to build request");

        app.send_request(request).await
    }

    /// Test that duplicate requests with same idempotency key are rejected
    #[tokio::test]
    async fn test_duplicate_request_handling() {
        let db = TestApp::init_db().await.expect("Failed to init DB");
        let tenant_id = db.create_test_tenant("Idempotency").await;
        let user_id = db.create_test_user(tenant_id, "idemp1").await;

        let app = TestApp::build_app(db).await;

        // First request - should succeed
        let (status1, body1) = make_idempotency_request(&app, tenant_id, user_id, "key-1", "001").await;
        if !status1.is_success() {
             panic!("First request should succeed for idempotency test. Got status: {}, body: {}", status1, body1);
        }

        // Second request with SAME key - should match first response
        let (status2, body2) = make_idempotency_request(&app, tenant_id, user_id, "key-1", "001").await;

        assert_eq!(status1, status2, "Status codes should match");
        assert_eq!(body1, body2, "Response bodies should match");

        app.cleanup().await;
    }

    /// Test that different idempotency keys allow separate requests
    #[tokio::test]
    async fn test_different_idempotency_keys() {
        let db = TestApp::init_db().await.expect("Failed to init DB");
        let tenant_id = db.create_test_tenant("Different Keys").await;
        let user_id = db.create_test_user(tenant_id, "idemp2").await;

        let app = TestApp::build_app(db).await;

        // Request 1
        let (status1, body1) = make_idempotency_request(&app, tenant_id, user_id, "key-A", "A").await;

        // Request 2 (different key)
        let (status2, body2) = make_idempotency_request(&app, tenant_id, user_id, "key-B", "B").await;

        // Both should get the same treatment (both succeed)
        assert_eq!(
            status1, status2,
            "Different keys should allow independent requests. Status1: {:?}, Status2: {:?}",
            status1, status2
        );

        // Both requests MUST succeed
        assert!(
            status1.is_success(),
            "Both requests should succeed with different idempotency keys. \
             Got status1: {:?}, body1: {}, status2: {:?}, body2: {}",
            status1,
            body1,
            status2,
            body2
        );

        app.cleanup().await;
    }



    /// Test idempotency under concurrent duplicate requests
    ///
    /// This validates that when two requests with the same idempotency key
    /// are fired in parallel, only one operation is applied (or both return
    /// the same response), preventing double inserts under contention.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_concurrent_duplicate_idempotency() {
        let db = TestApp::init_db().await.expect("Failed to init DB");
        let tenant_id = db.create_test_tenant("Concurrent Idem").await;
        let user_id = db.create_test_user(tenant_id, "idemp3").await;

        let app = TestApp::build_app(db).await;

        let _warehouse_id = app
            .db()
            .create_test_warehouse(tenant_id, "CONC-IDEM-WH", "Concurrent Idempotency WH")
            .await;

        let idempotency_key = "concurrent-key-1".to_string();

        let (result1, result2) = tokio::join!(
            make_idempotency_request(&app, tenant_id, user_id, &idempotency_key, "CON1"),
            make_idempotency_request(&app, tenant_id, user_id, &idempotency_key, "CON2")
        );

        let (status1, body1) = result1;
        let (status2, body2) = result2;

        // Under proper idempotency, exactly one request should succeed and the other
        // should be rejected with a conflict. Using XOR to enforce this strictly.
        let s1_success = status1.is_success();
        let s2_success = status2.is_success();

        assert!(
            s1_success ^ s2_success,
            "Exactly one request should succeed, but got status1: {:?} and status2: {:?}. \
             Body1: {}, Body2: {}",
            status1,
            status2,
            body1,
            body2
        );

        // The conflicting request must return 409 Conflict
        if s1_success {
            assert_eq!(
                status2,
                StatusCode::CONFLICT,
                "The conflicting request should return 409 Conflict. Got status2: {:?}, body: {}",
                status2,
                body2
            );
        } else {
            assert_eq!(
                status1,
                StatusCode::CONFLICT,
                "The conflicting request should return 409 Conflict. Got status1: {:?}, body: {}",
                status1,
                body1
            );
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
        let warehouse_id = app
            .db()
            .create_test_warehouse(tenant_id, "CON-WH", "Consistency WH")
            .await;
        let product_id = app
            .db()
            .create_test_product(tenant_id, "CON-001", "Consistency Product")
            .await;

        let user_id = app.db().create_test_user(tenant_id, "con_user").await;
        let location_id = app.db().create_test_location(tenant_id, warehouse_id, "Z_CON", "L_CON", user_id).await;

        // Initial: 1000 units
        app.db()
            .set_inventory_level(tenant_id, warehouse_id, location_id, product_id, INITIAL_INVENTORY_LARGE)
            .await;

        let db_pool = app.db().pool.clone();
        let mut handles = JoinSet::new();

        // 20 concurrent +10 operations
        for _ in 0..CONCURRENT_INCREMENT_OPS {
            let pool = db_pool.clone();
            let t_id = tenant_id;
            let w_id = warehouse_id;
            let l_id = location_id;
            let p_id = product_id;
            handles.spawn(async move {
                sqlx::query(
                    "UPDATE inventory_levels
                     SET available_quantity = available_quantity + $5, updated_at = NOW()
                     WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4",
                )
                .bind(t_id)
                .bind(w_id)
                .bind(l_id)
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
            let l_id = location_id;
            let p_id = product_id;
            handles.spawn(async move {
                sqlx::query(
                    "UPDATE inventory_levels
                     SET available_quantity = available_quantity - $5, updated_at = NOW()
                     WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4",
                )
                .bind(t_id)
                .bind(w_id)
                .bind(l_id)
                .bind(p_id)
                .bind(DECREMENT_AMOUNT)
                .execute(&pool)
                .await
            });
        }

        // Wait for all concurrent operations with timeout to prevent CI hanging
        let mut failure_count = 0;
        let wait_result = tokio::time::timeout(Duration::from_secs(JOINSET_TIMEOUT_SECS), async {
            while let Some(join_result) = handles.join_next().await {
                match join_result {
                    Ok(task_result) => {
                        if task_result.is_err() {
                            // Check if the SQL operation itself failed
                            eprintln!("SQL operation failed: {}", task_result.unwrap_err());
                            failure_count += 1;
                        }
                    },
                    Err(join_error) => {
                        eprintln!("join error in consistency task: {join_error}");
                        failure_count += 1;
                    },
                }
            }
        })
        .await;

        if wait_result.is_err() {
            handles.shutdown().await;
            panic!("Test timed out after {}s - possible deadlock or stall", JOINSET_TIMEOUT_SECS);
        }
        assert_eq!(failure_count, 0, "All concurrent operations must succeed");

        // Final: 1000 + (20 * 10) - (10 * 5) = 1000 + 200 - 50 = 1150
        let expected_final = INITIAL_INVENTORY_LARGE
            + (CONCURRENT_INCREMENT_OPS as i64 * INCREMENT_AMOUNT)
            - (CONCURRENT_DECREMENT_OPS as i64 * DECREMENT_AMOUNT);
        let final_qty: Option<i64> = sqlx::query_scalar(
            "SELECT available_quantity FROM inventory_levels
             WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4"
        )
        .bind(tenant_id)
        .bind(warehouse_id)
        .bind(location_id)
        .bind(product_id)
        .fetch_optional(&app.db().pool)
        .await
        .unwrap();
        assert_eq!(final_qty, Some(expected_final), "Final quantity should be {}", expected_final);

        app.cleanup().await;
    }

    /// Test that no negative inventory is allowed
    #[tokio::test]
    async fn test_no_negative_inventory() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("No Negative").await;
        let warehouse_id = app
            .db()
            .create_test_warehouse(tenant_id, "NEG-WH", "No Neg WH")
            .await;
        let product_id = app
            .db()
            .create_test_product(tenant_id, "NEG-001", "No Neg Product")
            .await;

        let user_id = app.db().create_test_user(tenant_id, "neg_user").await;
        let location_id = app.db().create_test_location(tenant_id, warehouse_id, "Z_NEG", "L_NEG", user_id).await;

        // Initial: 50 units
        app.db()
            .set_inventory_level(tenant_id, warehouse_id, location_id, product_id, 50)
            .await;

        // Try to deduct 100 (more than available)
        let result = sqlx::query(
            "UPDATE inventory_levels
             SET available_quantity = available_quantity - 100, updated_at = NOW()
             WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4
             AND available_quantity >= 100",
        )
        .bind(tenant_id)
        .bind(warehouse_id)
        .bind(location_id)
        .bind(product_id)
        .execute(&app.db().pool)
        .await
        .unwrap();

        // Should not update any rows
        assert_eq!(result.rows_affected(), 0, "Should not allow negative inventory");

        // Verify quantity unchanged
        let qty: Option<i64> = sqlx::query_scalar(
            "SELECT available_quantity FROM inventory_levels
             WHERE tenant_id = $1 AND warehouse_id = $2 AND location_id = $3 AND product_id = $4"
        )
        .bind(tenant_id)
        .bind(warehouse_id)
        .bind(location_id)
        .bind(product_id)
        .fetch_optional(&app.db().pool)
        .await
        .unwrap();
        assert_eq!(qty, Some(50), "Quantity should remain 50");

        app.cleanup().await;
    }
}
