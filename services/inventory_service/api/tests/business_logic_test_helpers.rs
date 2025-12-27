//! Shared Test Helpers for Business Logic Tests
//!
//! Common utilities for integration tests covering valuation and reorder rules.
//! This module consolidates duplicate setup/cleanup logic to reduce code duplication.

#![allow(dead_code)]

use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, sync::Arc, time::Duration};
use tokio::sync::OnceCell;
use uuid::Uuid;

// ============================================================================
// Constants
// ============================================================================

/// Default test database URL (used when DATABASE_URL env var is not set)
const DEFAULT_TEST_DATABASE_URL: &str = "postgres://anthill:anthill@localhost:5432/anthill_test";

// ============================================================================
// Pool Setup
// ============================================================================

/// Initialize a test database connection pool.
///
/// Reuses a shared pool (OnceCell) with bounded connections and configurable
/// acquire timeout to reduce PoolTimedOut flakiness while avoiding connection
/// explosion when tests are run in parallel.
pub async fn setup_test_pool() -> PgPool {
    static TEST_POOL: OnceCell<PgPool> = OnceCell::const_new();

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        if env::var("CI").is_ok() {
            eprintln!("WARNING: DATABASE_URL not set in CI, using default test URL");
        }
        DEFAULT_TEST_DATABASE_URL.to_string()
    });

    let max_connections = env::var("TEST_DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);

    let acquire_timeout_secs = env::var("TEST_DB_ACQUIRE_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);

    let pool_mode = env::var("TEST_DB_POOL_MODE").unwrap_or_else(|_| "shared".to_string());

    if pool_mode.eq_ignore_ascii_case("fresh") || pool_mode.eq_ignore_ascii_case("per_test") {
        return PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(Duration::from_secs(acquire_timeout_secs))
            .connect(&database_url)
            .await
            .expect("Failed to initialize test pool");
    }

    let pool = TEST_POOL
        .get_or_init(|| async {
            PgPoolOptions::new()
                .max_connections(max_connections)
                .acquire_timeout(Duration::from_secs(acquire_timeout_secs))
                .connect(&database_url)
                .await
                .expect("Failed to initialize test pool")
        })
        .await;

    pool.clone()
}

// ============================================================================
// Service Factories
// ============================================================================

use inventory_service_infra::repositories::ValuationRepositoryImpl;
use inventory_service_infra::services::ValuationServiceImpl;

/// Create a ValuationService instance for testing.
#[allow(dead_code)]
pub fn create_valuation_service(pool: &PgPool) -> ValuationServiceImpl {
    // ValuationRepositoryImpl implements all three repository traits
    let repo = Arc::new(ValuationRepositoryImpl::new(pool.clone()));
    ValuationServiceImpl::new(repo.clone(), repo.clone(), repo)
}

use inventory_service_infra::repositories::{PgInventoryLevelRepository, PgReorderRuleRepository};
use inventory_service_infra::services::PgReplenishmentService;

/// Create a ReplenishmentService instance for testing.
pub fn create_replenishment_service(pool: &PgPool) -> PgReplenishmentService {
    let reorder_rule_repo = Arc::new(PgReorderRuleRepository::new(pool.clone()));
    // Use shared Arc for pool to avoid unnecessary allocations
    let pool_arc = Arc::new(pool.clone());
    let inventory_level_repo = Arc::new(PgInventoryLevelRepository::new(pool_arc));

    PgReplenishmentService::new(reorder_rule_repo, inventory_level_repo, None)
}

// ============================================================================
// Test Data Setup
// ============================================================================

/// Create a test tenant and product, returning their IDs.
pub async fn setup_test_tenant_and_product(pool: &PgPool) -> (Uuid, Uuid) {
    let tenant_id = Uuid::now_v7();
    let product_id = Uuid::now_v7();

    // Insert test tenant
    sqlx::query(
        "INSERT INTO tenants (tenant_id, name, slug, created_at) VALUES ($1, $2, $3, NOW())
         ON CONFLICT (tenant_id) DO NOTHING",
    )
    .bind(tenant_id)
    .bind("Test Tenant")
    .bind(format!("test-{}", tenant_id))
    .execute(pool)
    .await
    .expect("Failed to insert tenant");

    // Insert test product
    sqlx::query(
        "INSERT INTO products (product_id, tenant_id, sku, name, created_at)
         VALUES ($1, $2, $3, $4, NOW())
         ON CONFLICT (product_id) DO NOTHING",
    )
    .bind(product_id)
    .bind(tenant_id)
    .bind(format!("TEST-{}", Uuid::now_v7()))
    .bind("Test Product")
    .execute(pool)
    .await
    .expect("Failed to insert product");

    (tenant_id, product_id)
}

/// Create a test tenant, product, and warehouse, returning their IDs.
pub async fn setup_test_tenant_product_warehouse(pool: &PgPool) -> (Uuid, Uuid, Uuid) {
    let (tenant_id, product_id) = setup_test_tenant_and_product(pool).await;
    let warehouse_id = Uuid::now_v7();

    // Insert test warehouse
    sqlx::query(
        "INSERT INTO warehouses (tenant_id, warehouse_id, warehouse_name, warehouse_code, created_at, updated_at)
         VALUES ($1, $2, $3, $4, NOW(), NOW())
         ON CONFLICT (warehouse_id) DO NOTHING",
    )
    .bind(tenant_id)
    .bind(warehouse_id)
    .bind("Test Warehouse")
    .bind(format!("WH-{}", &Uuid::now_v7().to_string()[..8].to_uppercase()))
    .execute(pool)
    .await
    .expect("Failed to insert warehouse");

    (tenant_id, product_id, warehouse_id)
}

/// Create an inventory level record for testing.
pub async fn create_inventory_level(
    pool: &PgPool,
    tenant_id: Uuid,
    product_id: Uuid,
    warehouse_id: Uuid,
    quantity: i64,
) {
    // Delete any existing record first (location_id constraint requires this approach)
    let _ = sqlx::query(
        "DELETE FROM inventory_levels WHERE tenant_id = $1 AND warehouse_id = $2 AND product_id = $3",
    )
    .bind(tenant_id)
    .bind(warehouse_id)
    .bind(product_id)
    .execute(pool)
    .await;

    sqlx::query(
        "INSERT INTO inventory_levels (inventory_id, tenant_id, warehouse_id, product_id, available_quantity, created_at)
         VALUES (gen_random_uuid(), $1, $2, $3, $4, NOW())",
    )
    .bind(tenant_id)
    .bind(warehouse_id)
    .bind(product_id)
    .bind(quantity)
    .execute(pool)
    .await
    .expect("Failed to create inventory level");
}

// ============================================================================
// Test Data Cleanup
// ============================================================================

/// Clean up valuation-related test data for a tenant.
#[allow(dead_code)]
pub async fn cleanup_valuation_test_data(pool: &PgPool, tenant_id: Uuid) {
    // Clean up in reverse dependency order
    let _ = sqlx::query("DELETE FROM valuation_history WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM valuation_layers WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM valuations WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(pool)
        .await;
    cleanup_base_test_data(pool, tenant_id).await;
}

/// Clean up reorder-related test data for a tenant.
pub async fn cleanup_reorder_test_data(pool: &PgPool, tenant_id: Uuid) {
    let _ = sqlx::query("DELETE FROM reorder_rules WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM inventory_levels WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM warehouses WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(pool)
        .await;
    cleanup_base_test_data(pool, tenant_id).await;
}

/// Clean up base test data (products and tenants).
async fn cleanup_base_test_data(pool: &PgPool, tenant_id: Uuid) {
    let _ = sqlx::query("DELETE FROM products WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM tenants WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(pool)
        .await;
}
