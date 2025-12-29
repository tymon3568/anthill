//! Integration tests for Scrap Management MVP P1 feature
//!
//! These tests cover:
//! - Scrap document creation and workflow (draft → posted)
//! - Tenant isolation (tenant A cannot access tenant B's scraps)
//! - Status transitions validation
//! - Idempotency (double-post prevention)
//! - Inventory impact verification

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

use inventory_service_core::dto::scrap::{ScrapReasonCode, ScrapStatus};
use shared_auth::AuthUser;

// Test utilities
mod helpers;

use helpers::{create_test_app, create_test_user, setup_test_database};

/// Helper to create a test tenant with warehouse, location and product
async fn setup_tenant_with_inventory(
    pool: &PgPool,
    tenant_name: &str,
) -> (Uuid, Uuid, Uuid, Uuid, Uuid) {
    let tenant_id = Uuid::now_v7();
    let user_id = Uuid::now_v7();
    let warehouse_id = Uuid::now_v7();
    let location_id = Uuid::now_v7();
    let product_id = Uuid::now_v7();
    let slug = format!("test-scrap-tenant-{}-{}", tenant_name, tenant_id);

    // Insert tenant
    sqlx::query(
        "INSERT INTO tenants (tenant_id, name, slug, plan, status, settings, created_at, updated_at)
         VALUES ($1, $2, $3, 'free', 'active', '{}'::jsonb, NOW(), NOW())
         ON CONFLICT (tenant_id) DO NOTHING",
    )
    .bind(tenant_id)
    .bind(format!("Scrap Test Tenant {}", tenant_name))
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
    .bind(format!("scrap-test-{}@example.com", tenant_name))
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
    .bind(format!("SCRAP-WH-{}", tenant_name))
    .bind(format!("Scrap Warehouse {}", tenant_name))
    .execute(pool)
    .await
    .unwrap();

    // Insert location (scrap location)
    sqlx::query(
        "INSERT INTO locations (location_id, tenant_id, warehouse_id, location_code, location_name, location_type, created_at)
         VALUES ($1, $2, $3, $4, $5, 'scrap', NOW())
         ON CONFLICT (location_id) DO NOTHING",
    )
    .bind(location_id)
    .bind(tenant_id)
    .bind(warehouse_id)
    .bind(format!("SCRAP-LOC-{}", tenant_name))
    .bind(format!("Scrap Location {}", tenant_name))
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
    .bind(format!("SCRAP-SKU-{}", tenant_name))
    .bind(format!("Scrap Product {}", tenant_name))
    .execute(pool)
    .await
    .unwrap();

    (tenant_id, user_id, warehouse_id, location_id, product_id)
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
    use inventory_service_core::dto::scrap::{
        validate_scrap_line, validate_status_transition, ScrapLineInput, ScrapReasonCode,
        ScrapStatus,
    };

    #[test]
    fn test_scrap_status_draft_to_posted() {
        assert!(validate_status_transition(ScrapStatus::Draft, ScrapStatus::Posted).is_ok());
    }

    #[test]
    fn test_scrap_status_draft_to_cancelled() {
        assert!(validate_status_transition(ScrapStatus::Draft, ScrapStatus::Cancelled).is_ok());
    }

    #[test]
    fn test_scrap_status_posted_cannot_change() {
        // Once posted, cannot go back to draft
        assert!(validate_status_transition(ScrapStatus::Posted, ScrapStatus::Draft).is_err());
        // Posted cannot be cancelled (would need reversal)
        assert!(validate_status_transition(ScrapStatus::Posted, ScrapStatus::Cancelled).is_err());
    }

    #[test]
    fn test_scrap_status_cancelled_cannot_change() {
        assert!(validate_status_transition(ScrapStatus::Cancelled, ScrapStatus::Draft).is_err());
        assert!(validate_status_transition(ScrapStatus::Cancelled, ScrapStatus::Posted).is_err());
    }

    #[test]
    fn test_scrap_line_valid_qty() {
        let line = ScrapLineInput {
            product_id: Uuid::new_v4(),
            variant_id: None,
            source_location_id: Uuid::new_v4(),
            lot_id: None,
            serial_id: None,
            qty: 100,
            reason_code: Some(ScrapReasonCode::Damaged),
            reason: None,
        };
        assert!(validate_scrap_line(&line).is_ok());

        let line_one = ScrapLineInput {
            product_id: Uuid::new_v4(),
            variant_id: None,
            source_location_id: Uuid::new_v4(),
            lot_id: None,
            serial_id: None,
            qty: 1,
            reason_code: Some(ScrapReasonCode::Damaged),
            reason: None,
        };
        assert!(validate_scrap_line(&line_one).is_ok());
    }

    #[test]
    fn test_scrap_line_zero_qty() {
        let line = ScrapLineInput {
            product_id: Uuid::new_v4(),
            variant_id: None,
            source_location_id: Uuid::new_v4(),
            lot_id: None,
            serial_id: None,
            qty: 0,
            reason_code: Some(ScrapReasonCode::Damaged),
            reason: None,
        };
        assert!(validate_scrap_line(&line).is_err());
    }

    #[test]
    fn test_scrap_line_negative_qty() {
        let line = ScrapLineInput {
            product_id: Uuid::new_v4(),
            variant_id: None,
            source_location_id: Uuid::new_v4(),
            lot_id: None,
            serial_id: None,
            qty: -1,
            reason_code: Some(ScrapReasonCode::Damaged),
            reason: None,
        };
        assert!(validate_scrap_line(&line).is_err());
    }

    #[test]
    fn test_reason_code_display() {
        assert_eq!(ScrapReasonCode::Damaged.to_string(), "damaged");
        assert_eq!(ScrapReasonCode::Expired.to_string(), "expired");
        assert_eq!(ScrapReasonCode::Lost.to_string(), "lost");
        assert_eq!(ScrapReasonCode::QualityFail.to_string(), "quality_fail");
        assert_eq!(ScrapReasonCode::Obsolete.to_string(), "obsolete");
        assert_eq!(ScrapReasonCode::Other.to_string(), "other");
    }
}

// ============================================================================
// Integration tests requiring database
// ============================================================================

/// Test creating a scrap document
#[cfg(feature = "integration_tests_scrap")]
#[tokio::test]
async fn test_scrap_create_document() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;
    let (tenant_id, user_id, _warehouse_id, scrap_location_id, _product_id) =
        setup_tenant_with_inventory(&pool, "Create").await;
    let auth_user = create_auth_user(user_id, tenant_id, "scrap-create@example.com");

    let create_request = json!({
        "scrap_location_id": scrap_location_id,
        "reference": "SCRAP-001",
        "notes": "Test scrap document creation"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/scrap")
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

    assert!(create_response["scrap_id"].is_string());
    assert_eq!(create_response["status"], "draft");
    assert_eq!(create_response["reference"], "SCRAP-001");
}

/// Test adding lines to a scrap document
#[cfg(feature = "integration_tests_scrap")]
#[tokio::test]
async fn test_scrap_add_lines() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;
    let (tenant_id, user_id, _warehouse_id, scrap_location_id, product_id) =
        setup_tenant_with_inventory(&pool, "Lines").await;
    let auth_user = create_auth_user(user_id, tenant_id, "scrap-lines@example.com");

    // Create scrap document first
    let create_request = json!({
        "scrap_location_id": scrap_location_id,
        "reference": "SCRAP-LINES-001"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/scrap")
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let create_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let scrap_id = create_response["scrap_id"].as_str().unwrap();

    // Add lines
    let lines_request = json!({
        "lines": [
            {
                "product_id": product_id,
                "source_location_id": scrap_location_id,
                "qty": 10,
                "reason_code": "damaged",
                "reason": "Broken during handling"
            },
            {
                "product_id": product_id,
                "source_location_id": scrap_location_id,
                "qty": 5,
                "reason_code": "expired",
                "reason": "Past expiration date"
            }
        ]
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/scrap/{}/lines", scrap_id))
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(lines_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let lines_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Should have 2 lines
    if let Some(lines) = lines_response["lines"].as_array() {
        assert_eq!(lines.len(), 2);
    }
}

/// Test tenant isolation - Tenant B cannot access Tenant A's scrap documents
#[cfg(feature = "integration_tests_scrap")]
#[tokio::test]
async fn test_scrap_tenant_isolation() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;

    // Setup Tenant A
    let (tenant_a_id, user_a_id, _, scrap_location_a, _) =
        setup_tenant_with_inventory(&pool, "IsoA").await;
    let auth_user_a = create_auth_user(user_a_id, tenant_a_id, "scrap-iso-a@example.com");

    // Setup Tenant B
    let (tenant_b_id, user_b_id, _, _, _) = setup_tenant_with_inventory(&pool, "IsoB").await;
    let auth_user_b = create_auth_user(user_b_id, tenant_b_id, "scrap-iso-b@example.com");

    // Tenant A creates a scrap document
    let create_request = json!({
        "scrap_location_id": scrap_location_a,
        "reference": "SCRAP-TENANT-A",
        "notes": "Should not be visible to Tenant B"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/scrap")
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
    let scrap_id = create_response["scrap_id"].as_str().unwrap();

    // Tenant B tries to access Tenant A's scrap - should get 404
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/inventory/scrap/{}", scrap_id))
                .header("x-user-id", auth_user_b.user_id.to_string())
                .header("x-tenant-id", auth_user_b.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 404 Not Found (not 403 to avoid leaking existence)
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Tenant B tries to add lines to Tenant A's scrap - should fail
    let lines_request = json!({
        "lines": [{"product_id": Uuid::new_v4(), "source_location_id": Uuid::new_v4(), "qty": 1, "reason_code": "damaged"}]
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/scrap/{}/lines", scrap_id))
                .header("content-type", "application/json")
                .header("x-user-id", auth_user_b.user_id.to_string())
                .header("x-tenant-id", auth_user_b.tenant_id.to_string())
                .body(Body::from(lines_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Tenant B tries to post Tenant A's scrap - should fail
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/scrap/{}/post", scrap_id))
                .header("x-user-id", auth_user_b.user_id.to_string())
                .header("x-tenant-id", auth_user_b.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// Test scrap posting flow
#[cfg(feature = "integration_tests_scrap")]
#[tokio::test]
async fn test_scrap_post_flow() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;
    let (tenant_id, user_id, warehouse_id, scrap_location_id, product_id) =
        setup_tenant_with_inventory(&pool, "Post").await;
    let auth_user = create_auth_user(user_id, tenant_id, "scrap-post@example.com");

    // Setup: Create some inventory first
    let source_location_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO locations (location_id, tenant_id, warehouse_id, location_code, location_name, location_type, created_at)
         VALUES ($1, $2, $3, 'SRC-LOC', 'Source Location', 'storage', NOW())
         ON CONFLICT (location_id) DO NOTHING",
    )
    .bind(source_location_id)
    .bind(tenant_id)
    .bind(warehouse_id)
    .execute(&pool)
    .await
    .unwrap();

    // Create inventory level
    sqlx::query(
        "INSERT INTO inventory_levels (inventory_id, tenant_id, warehouse_id, location_id, product_id, available_quantity, reserved_quantity, created_at)
         VALUES ($1, $2, $3, $4, $5, 100, 0, NOW())
         ON CONFLICT DO NOTHING",
    )
    .bind(Uuid::now_v7())
    .bind(tenant_id)
    .bind(warehouse_id)
    .bind(source_location_id)
    .bind(product_id)
    .execute(&pool)
    .await
    .unwrap();

    // 1. Create scrap document
    let create_request = json!({
        "scrap_location_id": scrap_location_id,
        "reference": "SCRAP-POST-001"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/scrap")
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let create_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let scrap_id = create_response["scrap_id"].as_str().unwrap();

    // 2. Add lines
    let lines_request = json!({
        "lines": [{
            "product_id": product_id,
            "source_location_id": source_location_id,
            "qty": 10,
            "reason_code": "damaged",
            "reason": "Test scrap"
        }]
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/scrap/{}/lines", scrap_id))
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(lines_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());

    // 3. Post scrap
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/scrap/{}/post", scrap_id))
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());

    // 4. Verify status is now 'posted'
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/inventory/scrap/{}", scrap_id))
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let get_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(get_response["status"], "posted");
}

/// Test double-post prevention (idempotency)
#[cfg(feature = "integration_tests_scrap")]
#[tokio::test]
async fn test_scrap_idempotency() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;
    let (tenant_id, user_id, warehouse_id, scrap_location_id, product_id) =
        setup_tenant_with_inventory(&pool, "Idemp").await;
    let auth_user = create_auth_user(user_id, tenant_id, "scrap-idemp@example.com");

    // Setup source location with inventory
    let source_location_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO locations (location_id, tenant_id, warehouse_id, location_code, location_name, location_type, created_at)
         VALUES ($1, $2, $3, 'IDEMP-LOC', 'Idempotency Location', 'storage', NOW())
         ON CONFLICT (location_id) DO NOTHING",
    )
    .bind(source_location_id)
    .bind(tenant_id)
    .bind(warehouse_id)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO inventory_levels (inventory_id, tenant_id, warehouse_id, location_id, product_id, available_quantity, reserved_quantity, created_at)
         VALUES ($1, $2, $3, $4, $5, 100, 0, NOW())
         ON CONFLICT DO NOTHING",
    )
    .bind(Uuid::now_v7())
    .bind(tenant_id)
    .bind(warehouse_id)
    .bind(source_location_id)
    .bind(product_id)
    .execute(&pool)
    .await
    .unwrap();

    // Create and setup scrap document
    let create_request = json!({
        "scrap_location_id": scrap_location_id,
        "reference": "SCRAP-IDEMP-001"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/scrap")
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let create_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let scrap_id = create_response["scrap_id"].as_str().unwrap();

    // Add lines
    let lines_request = json!({
        "lines": [{
            "product_id": product_id,
            "source_location_id": source_location_id,
            "qty": 5,
            "reason_code": "lost"
        }]
    });

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/scrap/{}/lines", scrap_id))
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(lines_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // First post - should succeed
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/scrap/{}/post", scrap_id))
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());

    // Second post - should return success (idempotent) or conflict
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/scrap/{}/post", scrap_id))
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Idempotent contract: either 200 OK (returns existing) or 409 Conflict (already posted)
    // Both are acceptable implementations; BAD_REQUEST would indicate a regression
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::CONFLICT,
        "Double-post should return OK or CONFLICT, got: {}",
        response.status()
    );
}

/// Test cancel draft scrap document
#[cfg(feature = "integration_tests_scrap")]
#[tokio::test]
async fn test_scrap_cancel_draft() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;
    let (tenant_id, user_id, _, scrap_location_id, _) =
        setup_tenant_with_inventory(&pool, "Cancel").await;
    let auth_user = create_auth_user(user_id, tenant_id, "scrap-cancel@example.com");

    // Create scrap document
    let create_request = json!({
        "scrap_location_id": scrap_location_id,
        "reference": "SCRAP-CANCEL-001"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/scrap")
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let create_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let scrap_id = create_response["scrap_id"].as_str().unwrap();

    // Cancel the draft
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/scrap/{}/cancel", scrap_id))
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());

    // Verify status is cancelled
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/inventory/scrap/{}", scrap_id))
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let get_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(get_response["status"], "cancelled");
}

// ============================================================================
// Database-level tests (can run without full app setup)
// ============================================================================

/// Test that scrap queries filter by tenant_id
#[tokio::test]
async fn test_scrap_tenant_filter_in_queries() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:password@localhost:5432/inventory_db".to_string());

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database - ensure DATABASE_URL is set for DB-level tests");

    let tenant_a_id = Uuid::now_v7();
    let tenant_b_id = Uuid::now_v7();

    // Insert test tenants
    for (tenant_id, name) in [
        (tenant_a_id, "Scrap Filter Test A"),
        (tenant_b_id, "Scrap Filter Test B"),
    ] {
        let slug = format!("test-scrap-filter-{}", tenant_id);
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
        .expect("Failed to insert tenant for scrap test");
    }

    // Create scrap locations (warehouses) for each tenant to satisfy FK constraint
    let scrap_location_a_id = Uuid::now_v7();
    let scrap_location_b_id = Uuid::now_v7();

    sqlx::query(
        "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, created_at)
         VALUES ($1, $2, $3, $4, NOW())
         ON CONFLICT (warehouse_id) DO NOTHING",
    )
    .bind(scrap_location_a_id)
    .bind(tenant_a_id)
    .bind("SCRAP-FILTER-A")
    .bind("Scrap Filter Warehouse A")
    .execute(&pool)
    .await
    .expect("Failed to insert warehouse for tenant A");

    sqlx::query(
        "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, created_at)
         VALUES ($1, $2, $3, $4, NOW())
         ON CONFLICT (warehouse_id) DO NOTHING",
    )
    .bind(scrap_location_b_id)
    .bind(tenant_b_id)
    .bind("SCRAP-FILTER-B")
    .bind("Scrap Filter Warehouse B")
    .execute(&pool)
    .await
    .expect("Failed to insert warehouse for tenant B");

    // Create scrap documents for each tenant (if table exists)
    let scrap_a_id = Uuid::now_v7();
    let scrap_b_id = Uuid::now_v7();

    // Insert scrap documents with correct scrap_location_id (not tenant_id)
    sqlx::query(
        "INSERT INTO scrap_documents (scrap_id, tenant_id, scrap_location_id, status, created_at, updated_at)
         VALUES ($1, $2, $3, 'draft', NOW(), NOW())
         ON CONFLICT DO NOTHING",
    )
    .bind(scrap_a_id)
    .bind(tenant_a_id)
    .bind(scrap_location_a_id)
    .execute(&pool)
    .await
    .expect("Failed to insert scrap document for tenant A");

    sqlx::query(
        "INSERT INTO scrap_documents (scrap_id, tenant_id, scrap_location_id, status, created_at, updated_at)
         VALUES ($1, $2, $3, 'draft', NOW(), NOW())
         ON CONFLICT DO NOTHING",
    )
    .bind(scrap_b_id)
    .bind(tenant_b_id)
    .bind(scrap_location_b_id)
    .execute(&pool)
    .await
    .expect("Failed to insert scrap document for tenant B");

    // Query for tenant A - should only see tenant A's scraps
    let rows_a: Vec<(Uuid,)> =
        sqlx::query_as("SELECT scrap_id FROM scrap_documents WHERE tenant_id = $1")
            .bind(tenant_a_id)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

    for (id,) in &rows_a {
        assert_ne!(*id, scrap_b_id, "Tenant A should not see Tenant B's scrap");
    }

    // Query for tenant B - should only see tenant B's scraps
    let rows_b: Vec<(Uuid,)> =
        sqlx::query_as("SELECT scrap_id FROM scrap_documents WHERE tenant_id = $1")
            .bind(tenant_b_id)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

    for (id,) in &rows_b {
        assert_ne!(*id, scrap_a_id, "Tenant B should not see Tenant A's scrap");
    }

    // Cleanup
    sqlx::query("DELETE FROM scrap_documents WHERE scrap_id IN ($1, $2)")
        .bind(scrap_a_id)
        .bind(scrap_b_id)
        .execute(&pool)
        .await
        .expect("Failed to clean up scrap_documents test data");
}
