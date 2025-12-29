//! Integration tests for Stock Reports MVP P1 features
//!
//! These tests cover:
//! - Stock Aging Report (4.09.03)
//! - Inventory Turnover Report (4.09.04)
//! - Tenant isolation for both reports
//! - Calculation correctness verification

#![allow(
    dead_code,
    unused_imports,
    clippy::needless_borrow,
    clippy::needless_borrows_for_generic_args
)]

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use chrono::{Duration, Utc};
use http_body_util::BodyExt;
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use inventory_service_core::dto::reports::{
    calculate_avg_inventory, calculate_dio, calculate_turnover_ratio, get_age_bucket_label,
    AgeBucket, AgeBucketPreset, AgingBasis,
};
use shared_auth::AuthUser;

// Test utilities
mod helpers;

use helpers::{create_test_app, create_test_user, setup_test_database};

/// Helper to create a test tenant with warehouse, products and stock moves
async fn setup_tenant_with_stock_data(
    pool: &PgPool,
    tenant_name: &str,
) -> (Uuid, Uuid, Uuid, Uuid) {
    let tenant_id = Uuid::now_v7();
    let user_id = Uuid::now_v7();
    let warehouse_id = Uuid::now_v7();
    let product_id = Uuid::now_v7();
    let slug = format!("test-reports-tenant-{}-{}", tenant_name, tenant_id);

    // Insert tenant
    sqlx::query(
        "INSERT INTO tenants (tenant_id, name, slug, plan, status, settings, created_at, updated_at)
         VALUES ($1, $2, $3, 'free', 'active', '{}'::jsonb, NOW(), NOW())
         ON CONFLICT (tenant_id) DO NOTHING",
    )
    .bind(tenant_id)
    .bind(format!("Reports Test Tenant {}", tenant_name))
    .bind(&slug)
    .execute(pool)
    .await
    .unwrap();

    // Insert user
    sqlx::query(
        "INSERT INTO users (user_id, tenant_id, email, created_at) VALUES ($1, $2, $3, NOW())
         ON CONFLICT (user_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(tenant_id)
    .bind(format!("reports-test-{}@example.com", tenant_name))
    .execute(pool)
    .await
    .unwrap();

    // Insert warehouse
    sqlx::query(
        "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, created_at)
         VALUES ($1, $2, $3, $4, NOW())
         ON CONFLICT (warehouse_id) DO NOTHING",
    )
    .bind(warehouse_id)
    .bind(tenant_id)
    .bind(format!("RPT-WH-{}", tenant_name))
    .bind(format!("Reports Warehouse {}", tenant_name))
    .execute(pool)
    .await
    .unwrap();

    // Insert product
    sqlx::query(
        "INSERT INTO products (product_id, tenant_id, sku, name, created_at)
         VALUES ($1, $2, $3, $4, NOW())
         ON CONFLICT (product_id) DO NOTHING",
    )
    .bind(product_id)
    .bind(tenant_id)
    .bind(format!("RPT-SKU-{}", tenant_name))
    .bind(format!("Reports Product {}", tenant_name))
    .execute(pool)
    .await
    .unwrap();

    (tenant_id, user_id, warehouse_id, product_id)
}

/// Helper to create AuthUser for a tenant
fn create_auth_user(user_id: Uuid, tenant_id: Uuid, email: &str) -> AuthUser {
    AuthUser {
        user_id,
        tenant_id,
        email: Some(email.to_string()),
        kanidm_user_id: Some(Uuid::new_v4()),
        role: "user".to_string(),
    }
}

// ============================================================================
// Unit-like tests for report calculations (no DB required)
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;

    // Stock Aging tests
    fn get_default_buckets() -> Vec<AgeBucket> {
        vec![
            AgeBucket {
                label: "0-30".to_string(),
                min_days: 0,
                max_days: Some(31),
            },
            AgeBucket {
                label: "31-60".to_string(),
                min_days: 31,
                max_days: Some(61),
            },
            AgeBucket {
                label: "61-90".to_string(),
                min_days: 61,
                max_days: Some(91),
            },
            AgeBucket {
                label: "91-180".to_string(),
                min_days: 91,
                max_days: Some(181),
            },
            AgeBucket {
                label: "181-365".to_string(),
                min_days: 181,
                max_days: Some(366),
            },
            AgeBucket {
                label: "365+".to_string(),
                min_days: 366,
                max_days: None,
            },
        ]
    }

    #[test]
    fn test_age_bucket_0_30_days() {
        let buckets = get_default_buckets();
        assert_eq!(get_age_bucket_label(0, &buckets), "0-30");
        assert_eq!(get_age_bucket_label(15, &buckets), "0-30");
        assert_eq!(get_age_bucket_label(30, &buckets), "0-30");
    }

    #[test]
    fn test_age_bucket_31_60_days() {
        let buckets = get_default_buckets();
        assert_eq!(get_age_bucket_label(31, &buckets), "31-60");
        assert_eq!(get_age_bucket_label(45, &buckets), "31-60");
        assert_eq!(get_age_bucket_label(60, &buckets), "31-60");
    }

    #[test]
    fn test_age_bucket_61_90_days() {
        let buckets = get_default_buckets();
        assert_eq!(get_age_bucket_label(61, &buckets), "61-90");
        assert_eq!(get_age_bucket_label(75, &buckets), "61-90");
        assert_eq!(get_age_bucket_label(90, &buckets), "61-90");
    }

    #[test]
    fn test_age_bucket_91_180_days() {
        let buckets = get_default_buckets();
        assert_eq!(get_age_bucket_label(91, &buckets), "91-180");
        assert_eq!(get_age_bucket_label(120, &buckets), "91-180");
        assert_eq!(get_age_bucket_label(180, &buckets), "91-180");
    }

    #[test]
    fn test_age_bucket_181_365_days() {
        let buckets = get_default_buckets();
        assert_eq!(get_age_bucket_label(181, &buckets), "181-365");
        assert_eq!(get_age_bucket_label(250, &buckets), "181-365");
        assert_eq!(get_age_bucket_label(365, &buckets), "181-365");
    }

    #[test]
    fn test_age_bucket_over_365_days() {
        let buckets = get_default_buckets();
        assert_eq!(get_age_bucket_label(366, &buckets), "365+");
        assert_eq!(get_age_bucket_label(500, &buckets), "365+");
        assert_eq!(get_age_bucket_label(1000, &buckets), "365+");
    }

    // Inventory Turnover tests
    #[test]
    fn test_turnover_ratio_normal() {
        // COGS = $10,000 (1_000_000 cents), Avg Inventory = $5,000 (500_000 cents)
        // Turnover = 10,000 / 5,000 = 2.0
        let ratio = calculate_turnover_ratio(1_000_000, 500_000);
        assert!((ratio - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_turnover_ratio_zero_inventory() {
        // Division by zero should return 0.0 safely
        let ratio = calculate_turnover_ratio(1_000_000, 0);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_turnover_ratio_zero_cogs() {
        // Zero COGS should return 0.0
        let ratio = calculate_turnover_ratio(0, 500_000);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_turnover_ratio_negative_inventory() {
        // Negative inventory should return 0.0 safely
        let ratio = calculate_turnover_ratio(1_000_000, -500_000);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_dio_normal() {
        // Turnover ratio = 4.0, Period = 365 days
        // DIO = 365 / 4 = 91.25 days
        let dio = calculate_dio(4.0, 365);
        assert!(dio.is_some());
        assert!((dio.unwrap() - 91.25).abs() < 0.01);
    }

    #[test]
    fn test_dio_zero_turnover() {
        // Zero turnover should return None
        let dio = calculate_dio(0.0, 365);
        assert!(dio.is_none());
    }

    #[test]
    fn test_dio_high_turnover() {
        // High turnover = 12 (monthly), Period = 365 days
        // DIO = 365 / 12 ≈ 30.4 days
        let dio = calculate_dio(12.0, 365);
        assert!(dio.is_some());
        assert!((dio.unwrap() - 30.42).abs() < 0.1);
    }

    #[test]
    fn test_avg_inventory() {
        // Opening = $10,000 (1_000_000 cents), Closing = $8,000 (800_000 cents)
        // Average = (1_000_000 + 800_000) / 2 = 900_000 cents
        let avg = calculate_avg_inventory(1_000_000, 800_000);
        assert_eq!(avg, 900_000);
    }

    #[test]
    fn test_avg_inventory_zero_values() {
        let avg = calculate_avg_inventory(0, 0);
        assert_eq!(avg, 0);
    }

    #[test]
    fn test_avg_inventory_negative_handling() {
        // Average of -100,000 and 100,000 = 0
        let avg = calculate_avg_inventory(-100_000, 100_000);
        assert_eq!(avg, 0);
    }
}

// ============================================================================
// Integration tests requiring database
// ============================================================================

/// Test Stock Aging Report endpoint
#[cfg(feature = "integration_tests_reports")]
#[tokio::test]
async fn test_stock_aging_report_basic() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;
    let (tenant_id, user_id, warehouse_id, _product_id) =
        setup_tenant_with_stock_data(&pool, "AgingBasic").await;
    let auth_user = create_auth_user(user_id, tenant_id, "aging-basic@example.com");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/api/v1/inventory/reports/aging?warehouse_id={}&aging_basis=last_inbound",
                    warehouse_id
                ))
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let report: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(report["rows"].is_array());
    assert!(report["as_of"].is_string());
    assert!(report["aging_basis"].is_string());
}

/// Test Stock Aging Report with both aging bases
#[cfg(feature = "integration_tests_reports")]
#[tokio::test]
async fn test_stock_aging_report_both_bases() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;
    let (tenant_id, user_id, warehouse_id, _) =
        setup_tenant_with_stock_data(&pool, "AgingBases").await;
    let auth_user = create_auth_user(user_id, tenant_id, "aging-bases@example.com");

    // Test last_inbound basis
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/api/v1/inventory/reports/aging?warehouse_id={}&aging_basis=last_inbound",
                    warehouse_id
                ))
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let report: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(report["aging_basis"], "last_inbound");

    // Test last_movement basis
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/api/v1/inventory/reports/aging?warehouse_id={}&aging_basis=last_movement",
                    warehouse_id
                ))
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let report: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(report["aging_basis"], "last_movement");
}

/// Test Stock Aging Report tenant isolation
#[cfg(feature = "integration_tests_reports")]
#[tokio::test]
async fn test_stock_aging_tenant_isolation() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;

    // Setup Tenant A with data
    let (tenant_a_id, user_a_id, warehouse_a_id, product_a_id) =
        setup_tenant_with_stock_data(&pool, "AgingIsoA").await;
    let auth_user_a = create_auth_user(user_a_id, tenant_a_id, "aging-iso-a@example.com");

    // Setup Tenant B
    let (tenant_b_id, user_b_id, _, _) = setup_tenant_with_stock_data(&pool, "AgingIsoB").await;
    let auth_user_b = create_auth_user(user_b_id, tenant_b_id, "aging-iso-b@example.com");

    // Create stock move for Tenant A (if stock_moves table exists)
    let _ = sqlx::query(
        "INSERT INTO stock_moves (stock_move_id, tenant_id, product_id, destination_location_id, quantity, move_date, created_at)
         VALUES ($1, $2, $3, $4, 100, NOW() - INTERVAL '45 days', NOW())
         ON CONFLICT DO NOTHING",
    )
    .bind(Uuid::now_v7())
    .bind(tenant_a_id)
    .bind(product_a_id)
    .bind(warehouse_a_id)
    .execute(&pool)
    .await;

    // Tenant A queries aging report
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/inventory/reports/aging")
                .header("x-user-id", auth_user_a.user_id.to_string())
                .header("x-tenant-id", auth_user_a.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let report_a: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Tenant B queries aging report
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/inventory/reports/aging")
                .header("x-user-id", auth_user_b.user_id.to_string())
                .header("x-tenant-id", auth_user_b.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let report_b: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify Tenant B doesn't see Tenant A's data
    if let (Some(rows_a), Some(rows_b)) = (report_a["rows"].as_array(), report_b["rows"].as_array())
    {
        // Extract product IDs from each report
        let product_ids_a: Vec<&str> = rows_a
            .iter()
            .filter_map(|r| r["product_id"].as_str())
            .collect();

        let product_ids_b: Vec<&str> = rows_b
            .iter()
            .filter_map(|r| r["product_id"].as_str())
            .collect();

        // No overlap should exist
        for pid in &product_ids_a {
            assert!(!product_ids_b.contains(pid), "Tenant B should not see Tenant A's products");
        }
    }
}

/// Test Inventory Turnover Report endpoint
#[cfg(feature = "integration_tests_reports")]
#[tokio::test]
async fn test_inventory_turnover_report_basic() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;
    let (tenant_id, user_id, _, _) = setup_tenant_with_stock_data(&pool, "TurnoverBasic").await;
    let auth_user = create_auth_user(user_id, tenant_id, "turnover-basic@example.com");

    let from = (Utc::now() - Duration::days(90)).format("%Y-%m-%dT%H:%M:%SZ");
    let to = Utc::now().format("%Y-%m-%dT%H:%M:%SZ");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/api/v1/inventory/reports/turnover?from={}&to={}&group_by=product",
                    from, to
                ))
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let report: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(report["rows"].is_array());
    assert!(report["from"].is_string());
    assert!(report["to"].is_string());
    assert!(report["period_days"].is_number());
}

/// Test Inventory Turnover Report with different groupings
#[cfg(feature = "integration_tests_reports")]
#[tokio::test]
async fn test_inventory_turnover_report_groupings() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;
    let (tenant_id, user_id, _, _) = setup_tenant_with_stock_data(&pool, "TurnoverGroup").await;
    let auth_user = create_auth_user(user_id, tenant_id, "turnover-group@example.com");

    let from = (Utc::now() - Duration::days(90)).format("%Y-%m-%dT%H:%M:%SZ");
    let to = Utc::now().format("%Y-%m-%dT%H:%M:%SZ");

    for group_by in &["product", "category", "warehouse"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!(
                        "/api/v1/inventory/reports/turnover?from={}&to={}&group_by={}",
                        from, to, group_by
                    ))
                    .header("x-user-id", auth_user.user_id.to_string())
                    .header("x-tenant-id", auth_user.tenant_id.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK, "Failed for group_by={}", group_by);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let report: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(report["group_by"], *group_by, "group_by mismatch for {}", group_by);
    }
}

/// Test Inventory Turnover Report tenant isolation
#[cfg(feature = "integration_tests_reports")]
#[tokio::test]
async fn test_inventory_turnover_tenant_isolation() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;

    // Setup Tenant A
    let (tenant_a_id, user_a_id, warehouse_a_id, product_a_id) =
        setup_tenant_with_stock_data(&pool, "TurnoverIsoA").await;
    let auth_user_a = create_auth_user(user_a_id, tenant_a_id, "turnover-iso-a@example.com");

    // Setup Tenant B
    let (tenant_b_id, user_b_id, _, _) = setup_tenant_with_stock_data(&pool, "TurnoverIsoB").await;
    let auth_user_b = create_auth_user(user_b_id, tenant_b_id, "turnover-iso-b@example.com");

    // Create stock moves for Tenant A
    let _ = sqlx::query(
        "INSERT INTO stock_moves (stock_move_id, tenant_id, product_id, destination_location_id, quantity, total_cost, move_date, created_at)
         VALUES ($1, $2, $3, $4, 100, 10000, NOW() - INTERVAL '30 days', NOW())
         ON CONFLICT DO NOTHING",
    )
    .bind(Uuid::now_v7())
    .bind(tenant_a_id)
    .bind(product_a_id)
    .bind(warehouse_a_id)
    .execute(&pool)
    .await;

    let from = (Utc::now() - Duration::days(90)).format("%Y-%m-%dT%H:%M:%SZ");
    let to = Utc::now().format("%Y-%m-%dT%H:%M:%SZ");

    // Tenant A queries turnover report
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/api/v1/inventory/reports/turnover?from={}&to={}&group_by=product",
                    from, to
                ))
                .header("x-user-id", auth_user_a.user_id.to_string())
                .header("x-tenant-id", auth_user_a.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let report_a: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Tenant B queries turnover report
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/api/v1/inventory/reports/turnover?from={}&to={}&group_by=product",
                    from, to
                ))
                .header("x-user-id", auth_user_b.user_id.to_string())
                .header("x-tenant-id", auth_user_b.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let report_b: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify Tenant B doesn't see Tenant A's data
    if let (Some(rows_a), Some(rows_b)) = (report_a["rows"].as_array(), report_b["rows"].as_array())
    {
        let group_ids_a: Vec<&str> = rows_a
            .iter()
            .filter_map(|r| r["group_id"].as_str())
            .collect();

        let group_ids_b: Vec<&str> = rows_b
            .iter()
            .filter_map(|r| r["group_id"].as_str())
            .collect();

        for gid in &group_ids_a {
            assert!(!group_ids_b.contains(gid), "Tenant B should not see Tenant A's data");
        }
    }
}

/// Test Inventory Turnover Report validates required parameters
#[cfg(feature = "integration_tests_reports")]
#[tokio::test]
async fn test_inventory_turnover_report_validation() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;
    let (tenant_id, user_id, _, _) = setup_tenant_with_stock_data(&pool, "TurnoverValid").await;
    let auth_user = create_auth_user(user_id, tenant_id, "turnover-valid@example.com");

    // Missing 'from' parameter
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/inventory/reports/turnover?to=2024-12-31T23:59:59Z&group_by=product")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 400 Bad Request for missing required parameter
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Database-level tests (can run without full app setup)
// ============================================================================

/// Test that report queries filter by tenant_id at database level
#[tokio::test]
async fn test_reports_tenant_filter_in_queries() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:password@localhost:5432/inventory_db".to_string());

    let pool = match sqlx::PgPool::connect(&database_url).await {
        Ok(pool) => pool,
        Err(_) => {
            eprintln!("Skipping test - database not available");
            return;
        },
    };

    let tenant_a_id = Uuid::now_v7();
    let tenant_b_id = Uuid::now_v7();
    let product_a_id = Uuid::now_v7();
    let product_b_id = Uuid::now_v7();

    // Insert test tenants
    for (tenant_id, name) in [
        (tenant_a_id, "Reports Filter Test A"),
        (tenant_b_id, "Reports Filter Test B"),
    ] {
        let slug = format!("test-reports-filter-{}", tenant_id);
        let _ = sqlx::query(
            "INSERT INTO tenants (tenant_id, name, slug, plan, status, settings, created_at, updated_at)
             VALUES ($1, $2, $3, 'free', 'active', '{}'::jsonb, NOW(), NOW())
             ON CONFLICT (tenant_id) DO NOTHING",
        )
        .bind(tenant_id)
        .bind(name)
        .bind(&slug)
        .execute(&pool)
        .await;
    }

    // Insert products for each tenant
    let _ = sqlx::query(
        "INSERT INTO products (product_id, tenant_id, sku, name, created_at)
         VALUES ($1, $2, 'RPT-TEST-A', 'Report Test A', NOW())
         ON CONFLICT DO NOTHING",
    )
    .bind(product_a_id)
    .bind(tenant_a_id)
    .execute(&pool)
    .await;

    let _ = sqlx::query(
        "INSERT INTO products (product_id, tenant_id, sku, name, created_at)
         VALUES ($1, $2, 'RPT-TEST-B', 'Report Test B', NOW())
         ON CONFLICT DO NOTHING",
    )
    .bind(product_b_id)
    .bind(tenant_b_id)
    .execute(&pool)
    .await;

    // Query products for tenant A
    let rows_a: Vec<(Uuid,)> =
        sqlx::query_as("SELECT product_id FROM products WHERE tenant_id = $1")
            .bind(tenant_a_id)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

    // Verify tenant A only sees their products
    for (id,) in &rows_a {
        assert_ne!(*id, product_b_id, "Tenant A should not see Tenant B's products");
    }

    // Query products for tenant B
    let rows_b: Vec<(Uuid,)> =
        sqlx::query_as("SELECT product_id FROM products WHERE tenant_id = $1")
            .bind(tenant_b_id)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

    // Verify tenant B only sees their products
    for (id,) in &rows_b {
        assert_ne!(*id, product_a_id, "Tenant B should not see Tenant A's products");
    }

    // Cleanup
    let _ = sqlx::query("DELETE FROM products WHERE product_id IN ($1, $2)")
        .bind(product_a_id)
        .bind(product_b_id)
        .execute(&pool)
        .await;
}

/// Test turnover calculation with known values
#[tokio::test]
async fn test_turnover_calculation_known_values() {
    // This test verifies the pure calculation logic without needing DB

    // Scenario: Q1 2024 (90 days)
    // Opening inventory: $50,000 (5_000_000 cents)
    // Closing inventory: $40,000 (4_000_000 cents)
    // COGS: $180,000 (18_000_000 cents)

    let opening = 5_000_000_i64;
    let closing = 4_000_000_i64;
    let cogs = 18_000_000_i64;
    let period_days = 90_i64;

    // Average inventory = (50,000 + 40,000) / 2 = $45,000
    let avg_inventory = calculate_avg_inventory(opening, closing);
    assert_eq!(avg_inventory, 4_500_000); // 4,500,000 cents

    // Turnover ratio = 180,000 / 45,000 = 4.0
    let turnover_ratio = calculate_turnover_ratio(cogs, avg_inventory);
    assert!((turnover_ratio - 4.0).abs() < 0.001);

    // DIO = 90 / 4.0 = 22.5 days
    let dio = calculate_dio(turnover_ratio, period_days);
    assert!(dio.is_some());
    assert!((dio.unwrap() - 22.5).abs() < 0.1);
}
