//! Integration tests for reconciliation API flows
//!
//! These tests cover the complete reconciliation workflow:
//! - Create reconciliation
//! - Count items (manual and barcode scanning)
//! - Finalize reconciliation
//! - Approve reconciliation
//! - Analytics and reporting

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

use inventory_service_api::handlers::reconciliation::create_reconciliation_routes;
use inventory_service_api::state::AppState;
use inventory_service_core::domains::inventory::reconciliation::CycleType;
use inventory_service_core::dto::reconciliation::{
    CreateReconciliationRequest, ReconciliationListQuery,
};
use shared_auth::AuthUser;

// Test utilities
mod helpers;

use helpers::{create_test_app, create_test_user, setup_test_database};

/// This test requires full app setup including Redis, Casbin, etc.
/// Skip until proper test infrastructure is available.
#[cfg(feature = "integration_tests_reconciliation")]
#[tokio::test]
async fn test_complete_reconciliation_workflow() {
    // Setup
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;
    let auth_user = create_test_user(&pool).await;

    // Create reconciliation
    let create_request = CreateReconciliationRequest {
        name: "Test Reconciliation".to_string(),
        description: Some("Integration test reconciliation".to_string()),
        cycle_type: CycleType::Full,
        warehouse_id: None,
        location_filter: None,
        product_filter: None,
        notes: Some("Test notes".to_string()),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/reconciliations")
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(serde_json::to_string(&create_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let create_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let reconciliation_id: Uuid = create_response["reconciliation"]["reconciliation_id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    // Count items manually
    let count_request = json!({
        "items": [
            {
                "product_id": "550e8400-e29b-41d4-a716-446655440001",
                "warehouse_id": "550e8400-e29b-41d4-a716-446655440002",
                "counted_quantity": 50,
                "unit_cost": 10.0,
                "notes": "Manual count"
            }
        ]
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/reconciliations/{}/count", reconciliation_id))
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(count_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Scan barcode
    let scan_request = json!({
        "barcode": "550e8400-e29b-41d4-a716-446655440001",
        "quantity": 45,
        "notes": "Barcode scan"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/reconciliations/{}/scan", reconciliation_id))
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(scan_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Get reconciliation details
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/inventory/reconciliations/{}", reconciliation_id))
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Finalize reconciliation
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/reconciliations/{}/finalize", reconciliation_id))
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Approve reconciliation
    let approve_request = json!({
        "notes": "Approved by integration test"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/reconciliations/{}/approve", reconciliation_id))
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(approve_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Get analytics
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/inventory/reconciliations/analytics")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Get variance analysis
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/inventory/reconciliations/{}/variance", reconciliation_id))
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // List reconciliations
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/inventory/reconciliations?page=1&limit=10")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

/// This test requires full app setup including Redis, Casbin, etc.
/// Skip until proper test infrastructure is available.
#[cfg(feature = "integration_tests_reconciliation")]
#[tokio::test]
async fn test_reconciliation_validation_errors() {
    // Setup
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;
    let auth_user = create_test_user(&pool).await;

    // Test invalid reconciliation creation
    let invalid_request = json!({
        "name": "",  // Empty name should fail validation
        "cycle_type": "InvalidType"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/reconciliations")
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(invalid_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Test scanning invalid barcode
    let fake_reconciliation_id = Uuid::new_v4();
    let scan_request = json!({
        "barcode": "invalid-barcode",
        "quantity": 10
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/reconciliations/{}/scan", fake_reconciliation_id))
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(scan_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// This test requires full app setup including Redis, Casbin, etc.
/// Skip until proper test infrastructure is available.
#[cfg(feature = "integration_tests_reconciliation")]
#[tokio::test]
async fn test_reconciliation_business_rules() {
    // Setup
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;
    let auth_user = create_test_user(&pool).await;

    // Create reconciliation
    let create_request = CreateReconciliationRequest {
        name: "Business Rules Test".to_string(),
        description: Some("Testing business rules".to_string()),
        cycle_type: CycleType::Full,
        warehouse_id: None,
        location_filter: None,
        product_filter: None,
        notes: None,
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inventory/reconciliations")
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(serde_json::to_string(&create_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let create_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let reconciliation_id: Uuid = create_response["reconciliation"]["reconciliation_id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    // Try to finalize without counting all items (should fail)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/reconciliations/{}/finalize", reconciliation_id))
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Try to approve non-completed reconciliation (should fail)
    let approve_request = json!({
        "notes": "Should fail"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/inventory/reconciliations/{}/approve", reconciliation_id))
                .header("content-type", "application/json")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::from(approve_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// This test requires full app setup including Redis, Casbin, etc.
/// Skip until proper test infrastructure is available.
#[cfg(feature = "integration_tests_reconciliation")]
#[tokio::test]
async fn test_reconciliation_analytics_and_reporting() {
    // Setup
    let pool = setup_test_database().await;
    let app = create_test_app(pool.clone()).await;
    let auth_user = create_test_user(&pool).await;

    // Create multiple reconciliations for analytics
    for i in 1..=3 {
        let create_request = CreateReconciliationRequest {
            name: format!("Analytics Test {}", i),
            description: Some(format!("Analytics reconciliation {}", i)),
            cycle_type: CycleType::Full,
            warehouse_id: None,
            location_filter: None,
            product_filter: None,
            notes: Some(format!("Test reconciliation {}", i)),
        };

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/inventory/reconciliations")
                    .header("content-type", "application/json")
                    .header("x-user-id", auth_user.user_id.to_string())
                    .header("x-tenant-id", auth_user.tenant_id.to_string())
                    .body(Body::from(serde_json::to_string(&create_request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Test analytics endpoint
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/inventory/reconciliations/analytics")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let analytics: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Should have some basic analytics (even if placeholder)
    assert!(analytics["total_reconciliations"].is_number());
    assert!(analytics["completed_reconciliations"].is_number());

    // Test list with pagination
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/inventory/reconciliations?page=1&limit=2")
                .header("x-user-id", auth_user.user_id.to_string())
                .header("x-tenant-id", auth_user.tenant_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let list_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Should have pagination info
    assert!(list_response["pagination"]["page"].is_number());
    assert!(list_response["pagination"]["total"].is_number());
    assert!(list_response["reconciliations"].is_array());
}
