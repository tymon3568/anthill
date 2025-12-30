//! Integration tests for Cycle Counting MVP P1 feature
//!
//! These tests cover:
//! - Cycle count session creation and workflow
//! - Tenant isolation (tenant A cannot access tenant B's cycle counts)
//! - Status transitions validation
//! - Count submission and reconciliation

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
use http_body_util::BodyExt;
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use inventory_service_core::dto::cycle_count::CycleCountStatus;
use shared_auth::AuthUser;

// Test utilities
mod helpers;

use helpers::{create_test_app, create_test_user, setup_test_database};

/// Helper to create a test tenant with warehouse and product
async fn setup_tenant_with_inventory(pool: &PgPool, tenant_name: &str) -> (Uuid, Uuid, Uuid, Uuid) {
    let tenant_id = Uuid::now_v7();
    let user_id = Uuid::now_v7();
    let warehouse_id = Uuid::now_v7();
    let product_id = Uuid::now_v7();
    let slug = format!("test-tenant-{}-{}", tenant_name, tenant_id);

    // Insert tenant
    sqlx::query(
        "INSERT INTO tenants (tenant_id, name, slug, plan, status, settings, created_at, updated_at)
         VALUES ($1, $2, $3, 'free', 'active', '{}'::jsonb, NOW(), NOW())
         ON CONFLICT (tenant_id) DO NOTHING",
    )
    .bind(tenant_id)
    .bind(format!("Test Tenant {}", tenant_name))
    .bind(&slug)
    .execute(pool)
    .await
    .expect("Failed to insert tenant for cycle count test");

    // Insert user
    sqlx::query(
        "INSERT INTO users (user_id, tenant_id, email, created_at) VALUES ($1, $2, $3, NOW())
         ON CONFLICT (user_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(tenant_id)
    .bind(format!("test-{}@example.com", tenant_name))
    .execute(pool)
    .await
    .expect("Failed to insert user for cycle count test");

    // Insert warehouse
    sqlx::query(
        "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, created_at)
         VALUES ($1, $2, $3, $4, NOW())
         ON CONFLICT (warehouse_id) DO NOTHING",
    )
    .bind(warehouse_id)
    .bind(tenant_id)
    .bind(format!("WH-{}", tenant_name))
    .bind(format!("Warehouse {}", tenant_name))
    .execute(pool)
    .await
    .expect("Failed to insert warehouse for cycle count test");

    // Insert product
    sqlx::query(
        "INSERT INTO products (product_id, tenant_id, sku, name, created_at)
         VALUES ($1, $2, $3, $4, NOW())
         ON CONFLICT (product_id) DO NOTHING",
    )
    .bind(product_id)
    .bind(tenant_id)
    .bind(format!("SKU-{}", tenant_name))
    .bind(format!("Product {}", tenant_name))
    .execute(pool)
    .await
    .expect("Failed to insert product for cycle count test");

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
// Unit-like tests that don't require full app setup
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;
    use inventory_service_core::dto::cycle_count::{
        validate_count_submission, validate_status_transition, CountSubmission, CycleCountStatus,
    };

    #[test]
    fn test_status_transition_draft_to_in_progress() {
        assert!(
            validate_status_transition(CycleCountStatus::Draft, CycleCountStatus::InProgress)
                .is_ok()
        );
    }

    #[test]
    fn test_status_transition_in_progress_to_ready() {
        assert!(validate_status_transition(
            CycleCountStatus::InProgress,
            CycleCountStatus::ReadyToReconcile
        )
        .is_ok());
    }

    #[test]
    fn test_status_transition_ready_to_reconciled() {
        assert!(validate_status_transition(
            CycleCountStatus::ReadyToReconcile,
            CycleCountStatus::Reconciled
        )
        .is_ok());
    }

    #[test]
    fn test_invalid_status_transition_draft_to_reconciled() {
        assert!(
            validate_status_transition(CycleCountStatus::Draft, CycleCountStatus::Reconciled)
                .is_err()
        );
    }

    #[test]
    fn test_invalid_status_transition_reconciled_to_draft() {
        assert!(
            validate_status_transition(CycleCountStatus::Reconciled, CycleCountStatus::Draft)
                .is_err()
        );
    }

    #[test]
    fn test_count_submission_valid() {
        let submission = CountSubmission {
            line_id: Uuid::new_v4(),
            counted_qty: 100,
            notes: None,
        };
        assert!(validate_count_submission(&submission).is_ok());

        let submission_zero = CountSubmission {
            line_id: Uuid::new_v4(),
            counted_qty: 0,
            notes: None,
        };
        assert!(validate_count_submission(&submission_zero).is_ok());
    }

    #[test]
    fn test_count_submission_negative() {
        let submission = CountSubmission {
            line_id: Uuid::new_v4(),
            counted_qty: -1,
            notes: None,
        };
        assert!(validate_count_submission(&submission).is_err());
    }
}

// ============================================================================
// Integration tests requiring database
// ============================================================================

/// Test creating a cycle count session
#[cfg(feature = "integration_tests_cycle_count")]
#[tokio::test]
async fn test_cycle_count_create_session() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;
    let (tenant_id, user_id, warehouse_id, _product_id) =
        setup_tenant_with_inventory(&pool, "A").await;
    let auth_user = create_auth_user(user_id, tenant_id, "test-a@example.com");

    let create_request = json!({
        "warehouse_id": warehouse_id,
        "name": "Test Cycle Count",
        "description": "Integration test cycle count session"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/cycle-counts")
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let create_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // API returns CycleCountResponse with nested cycle_count field
    let cycle_count = &create_response["cycle_count"];
    assert!(cycle_count["cycle_count_id"].is_string());
    assert_eq!(cycle_count["status"], "draft");
}

/// Test tenant isolation - Tenant B cannot access Tenant A's cycle count
#[cfg(feature = "integration_tests_cycle_count")]
#[tokio::test]
async fn test_cycle_count_tenant_isolation() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;

    // Setup Tenant A
    let (tenant_a_id, user_a_id, warehouse_a_id, _) = setup_tenant_with_inventory(&pool, "A").await;
    let auth_user_a = create_auth_user(user_a_id, tenant_a_id, "test-a@example.com");

    // Setup Tenant B
    let (tenant_b_id, user_b_id, _, _) = setup_tenant_with_inventory(&pool, "B").await;
    let auth_user_b = create_auth_user(user_b_id, tenant_b_id, "test-b@example.com");

    // Tenant A creates a cycle count
    let create_request = json!({
        "warehouse_id": warehouse_a_id,
        "name": "Tenant A Cycle Count",
        "description": "Should not be visible to Tenant B"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/cycle-counts")
                .header("content-type", "application/json")
                .header("x-user-id", auth_user_a.user_id.to_string())
                .header("x-tenant-id", auth_user_a.tenant_id.to_string())
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let create_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // API returns CycleCountResponse with nested cycle_count field
    let cycle_count_id = create_response["cycle_count"]["cycle_count_id"]
        .as_str()
        .unwrap();

    // Tenant B tries to access Tenant A's cycle count - should get 404 (not 403 to avoid leaking existence)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/inventory/cycle-counts/{}", cycle_count_id))
                .header("x-user-id", auth_user_b.user_id.to_string())
                .header("x-tenant-id", auth_user_b.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 404 Not Found (not 403 Forbidden to avoid leaking existence)
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Tenant B lists cycle counts - should not see Tenant A's cycle count
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/inventory/cycle-counts")
                .header("x-user-id", auth_user_b.user_id.to_string())
                .header("x-tenant-id", auth_user_b.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let list_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Tenant B should have empty list or not contain Tenant A's cycle count
    // API returns CycleCountListResponse with cycle_counts array
    if let Some(sessions) = list_response["cycle_counts"].as_array() {
        for session in sessions {
            assert_ne!(session["cycle_count_id"].as_str(), Some(cycle_count_id));
        }
    }
}

/// Test complete cycle count workflow
#[cfg(feature = "integration_tests_cycle_count")]
#[tokio::test]
async fn test_cycle_count_e2e_workflow() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;
    let (tenant_id, user_id, warehouse_id, product_id) =
        setup_tenant_with_inventory(&pool, "E2E").await;
    let auth_user = create_auth_user(user_id, tenant_id, "test-e2e@example.com");

    // 1. Create cycle count session
    let create_request = json!({
        "warehouse_id": warehouse_id,
        "name": "E2E Test Cycle Count",
        "description": "End-to-end workflow test"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/cycle-counts")
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let create_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // API returns CycleCountResponse with nested cycle_count field
    let cycle_count_id = create_response["cycle_count"]["cycle_count_id"]
        .as_str()
        .unwrap();

    // 2. Generate lines
    let generate_request = json!({
        "product_ids": [product_id]
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/cycle-counts/{}/generate-lines", cycle_count_id))
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(generate_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // May return 200 OK or 201 Created depending on implementation
    assert!(response.status().is_success());

    // 3. Get generated lines to obtain line_id for count submission
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/inventory/cycle-counts/{}", cycle_count_id))
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let session_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Get line_id from the generated lines (API returns CycleCountWithLinesResponse)
    let line_id = session_response["lines"][0]["line_id"]
        .as_str()
        .expect("expected at least one generated line");

    // Submit counts using line_id (not product_id)
    let counts_request = json!({
        "counts": [
            {
                "line_id": line_id,
                "counted_qty": 100
            }
        ]
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/cycle-counts/{}/counts", cycle_count_id))
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(counts_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());

    // 4. Close session
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/cycle-counts/{}/close", cycle_count_id))
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());

    // 5. Reconcile (handler expects JSON body with ReconcileRequest)
    let reconcile_request = json!({
        "force": false
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/cycle-counts/{}/reconcile", cycle_count_id))
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(reconcile_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());

    // 6. Verify final status
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/inventory/cycle-counts/{}", cycle_count_id))
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let get_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // API returns CycleCountWithLinesResponse with nested cycle_count field
    assert_eq!(get_response["cycle_count"]["status"], "reconciled");
}

/// Test invalid status transitions are rejected
#[cfg(feature = "integration_tests_cycle_count")]
#[tokio::test]
async fn test_cycle_count_invalid_status_transition() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;
    let (tenant_id, user_id, warehouse_id, _) = setup_tenant_with_inventory(&pool, "Invalid").await;
    let auth_user = create_auth_user(user_id, tenant_id, "test-invalid@example.com");

    // Create cycle count session
    let create_request = json!({
        "warehouse_id": warehouse_id,
        "name": "Invalid Transition Test"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/cycle-counts")
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let create_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let cycle_count_id = create_response["cycle_count"]["cycle_count_id"]
        .as_str()
        .expect("cycle_count_id should be present in response");

    // Try to reconcile directly from draft (should fail)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/cycle-counts/{}/reconcile", cycle_count_id))
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 400 Bad Request or 409 Conflict for invalid state transition
    assert!(
        response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::CONFLICT
    );
}

// ============================================================================
// Database-level tests (can run without full app setup)
// ============================================================================

/// Test that cycle count queries filter by tenant_id
#[tokio::test]
async fn test_cycle_count_tenant_filter_in_queries() {
    // This test verifies at the database level that tenant filtering works
    // It doesn't require the full app setup

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:password@localhost:5432/inventory_db".to_string());

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database - ensure DATABASE_URL is set for DB-level tests");

    let tenant_a_id = Uuid::now_v7();
    let tenant_b_id = Uuid::now_v7();
    let user_a_id = Uuid::now_v7();
    let user_b_id = Uuid::now_v7();
    let warehouse_a_id = Uuid::now_v7();
    let warehouse_b_id = Uuid::now_v7();

    // Insert test tenants
    for (tenant_id, name) in [
        (tenant_a_id, "Filter Test A"),
        (tenant_b_id, "Filter Test B"),
    ] {
        let slug = format!("test-filter-{}", tenant_id);
        sqlx::query(
            "INSERT INTO tenants (tenant_id, name, slug, plan, status, settings, created_at, updated_at)
             VALUES ($1, $2, $3, 'free', 'active', '{}'::jsonb, NOW(), NOW())
             ON CONFLICT (tenant_id) DO NOTHING",
        )
        .bind(tenant_id)
        .bind(name)
        .bind(&slug)
        .execute(&pool)
        .await
        .expect("Failed to insert tenant for cycle count test");
    }

    // Insert test users for each tenant (required for stock_takes.created_by FK)
    for (user_id, tenant_id, email) in [
        (user_a_id, tenant_a_id, "filter-test-a@example.com"),
        (user_b_id, tenant_b_id, "filter-test-b@example.com"),
    ] {
        sqlx::query(
            "INSERT INTO users (user_id, tenant_id, email, created_at)
             VALUES ($1, $2, $3, NOW())",
        )
        .bind(user_id)
        .bind(tenant_id)
        .bind(email)
        .execute(&pool)
        .await
        .expect("Failed to insert user for cycle count test");
    }

    // Insert test warehouses for each tenant (required for stock_takes.warehouse_id FK)
    for (warehouse_id, tenant_id, code, name) in [
        (warehouse_a_id, tenant_a_id, "WH-FILTER-A", "Filter Test Warehouse A"),
        (warehouse_b_id, tenant_b_id, "WH-FILTER-B", "Filter Test Warehouse B"),
    ] {
        sqlx::query(
            "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, created_at)
             VALUES ($1, $2, $3, $4, NOW())",
        )
        .bind(warehouse_id)
        .bind(tenant_id)
        .bind(code)
        .bind(name)
        .execute(&pool)
        .await
        .expect("Failed to insert warehouse for cycle count test");
    }

    // Create stock_takes (cycle counts) for each tenant
    let stock_take_a_id = Uuid::now_v7();
    let stock_take_b_id = Uuid::now_v7();

    // Insert stock take for tenant A (using actual schema: warehouse_id, created_by, status, notes)
    sqlx::query(
        "INSERT INTO stock_takes (stock_take_id, tenant_id, warehouse_id, created_by, status, notes, created_at, updated_at)
         VALUES ($1, $2, $3, $4, 'Draft', 'Tenant A Stock Take', NOW(), NOW())",
    )
    .bind(stock_take_a_id)
    .bind(tenant_a_id)
    .bind(warehouse_a_id)
    .bind(user_a_id)
    .execute(&pool)
    .await
    .expect("Failed to insert stock take for tenant A");

    // Insert stock take for tenant B
    sqlx::query(
        "INSERT INTO stock_takes (stock_take_id, tenant_id, warehouse_id, created_by, status, notes, created_at, updated_at)
         VALUES ($1, $2, $3, $4, 'Draft', 'Tenant B Stock Take', NOW(), NOW())",
    )
    .bind(stock_take_b_id)
    .bind(tenant_b_id)
    .bind(warehouse_b_id)
    .bind(user_b_id)
    .execute(&pool)
    .await
    .expect("Failed to insert stock take for tenant B");

    // Query for tenant A - should only see tenant A's stock take
    let rows_a: Vec<(Uuid,)> =
        sqlx::query_as("SELECT stock_take_id FROM stock_takes WHERE tenant_id = $1")
            .bind(tenant_a_id)
            .fetch_all(&pool)
            .await
            .expect("Failed to query stock_takes for tenant A");

    // Explicit check that tenant A sees their own stock take
    assert!(
        rows_a.iter().any(|(id,)| *id == stock_take_a_id),
        "Tenant A should see their own stock take"
    );

    // Verify tenant A only sees their stock takes (not tenant B's)
    for (id,) in &rows_a {
        assert_ne!(*id, stock_take_b_id, "Tenant A should not see Tenant B's stock take");
    }

    // Query for tenant B - should only see tenant B's stock take
    let rows_b: Vec<(Uuid,)> =
        sqlx::query_as("SELECT stock_take_id FROM stock_takes WHERE tenant_id = $1")
            .bind(tenant_b_id)
            .fetch_all(&pool)
            .await
            .expect("Failed to query stock_takes for tenant B");

    // Explicit check that tenant B sees their own stock take
    assert!(
        rows_b.iter().any(|(id,)| *id == stock_take_b_id),
        "Tenant B should see their own stock take"
    );

    // Verify tenant B only sees their stock takes (not tenant A's)
    for (id,) in &rows_b {
        assert_ne!(*id, stock_take_a_id, "Tenant B should not see Tenant A's stock take");
    }

    // Cleanup - delete in reverse order due to FK constraints
    sqlx::query("DELETE FROM stock_takes WHERE stock_take_id = ANY($1)")
        .bind(&[stock_take_a_id, stock_take_b_id][..])
        .execute(&pool)
        .await
        .expect("Failed to clean up stock_takes test data");

    sqlx::query("DELETE FROM warehouses WHERE warehouse_id = ANY($1)")
        .bind(&[warehouse_a_id, warehouse_b_id][..])
        .execute(&pool)
        .await
        .expect("Failed to clean up warehouses test data");

    sqlx::query("DELETE FROM users WHERE user_id = ANY($1)")
        .bind(&[user_a_id, user_b_id][..])
        .execute(&pool)
        .await
        .expect("Failed to clean up users test data");

    sqlx::query("DELETE FROM tenants WHERE tenant_id = ANY($1)")
        .bind(&[tenant_a_id, tenant_b_id][..])
        .execute(&pool)
        .await
        .expect("Failed to clean up tenants test data");
}
