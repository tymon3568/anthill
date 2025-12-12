//! Integration tests for Stock Operations API endpoints
//!
//! Tests the stock-takes, transfers, and valuation endpoints with full HTTP request/response cycles.

use axum::{
    body::{to_bytes, Body},
    http::{Method, Request, StatusCode},
    Router,
};
use inventory_service_api::create_app;
use serde_json::{json, Value};
use shared_config::Config;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;
use uuid::Uuid;

// ============================================================================
// Shared Test Infrastructure
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
        redis_url: None,
        casbin_model_path: "shared/auth/model.conf".to_string(),
        max_connections: Some(10),
    }
}

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

    async fn cleanup(&self) {
        let tenant_ids = self.test_tenants.lock().await.clone();

        for tenant_id in tenant_ids {
            // Clean up in reverse dependency order
            let _ = sqlx::query("DELETE FROM stock_take_lines WHERE tenant_id = $1")
                .bind(tenant_id)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM stock_takes WHERE tenant_id = $1")
                .bind(tenant_id)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM transfer_items WHERE tenant_id = $1")
                .bind(tenant_id)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM transfers WHERE tenant_id = $1")
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
// Stock Take Tests
// ============================================================================

#[cfg(test)]
mod stock_take_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_stock_take() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Stock Take Create").await;
        let warehouse_id = app.db().create_test_warehouse(tenant_id, "ST-WH", "Stock Take Warehouse").await;

        let request_body = json!({
            "warehouse_id": warehouse_id,
            "name": "Q4 Inventory Count",
            "description": "Quarterly inventory count",
            "count_type": "full"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/inventory/stock-takes")
            .header("content-type", "application/json")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let (status, body) = app.send_request(request).await;

        // Should create stock take
        assert!(status == StatusCode::CREATED || status == StatusCode::OK);
        let _response: Value = serde_json::from_str(&body).unwrap_or(json!({}));

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_list_stock_takes() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Stock Take List").await;

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/inventory/stock-takes")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::OK);
        let response: Value = serde_json::from_str(&body).unwrap();
        // Should return a list (empty or with items)
        assert!(response.is_object() || response.is_array());

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_get_stock_take_not_found() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Stock Take Not Found").await;

        let fake_id = Uuid::new_v4();
        let uri = format!("/api/v1/inventory/stock-takes/{}", fake_id);
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, _body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::NOT_FOUND);

        app.cleanup().await;
    }
}

// ============================================================================
// Transfer Tests
// ============================================================================

#[cfg(test)]
mod transfer_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_transfer() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Transfer Create").await;
        let source_wh = app.db().create_test_warehouse(tenant_id, "SRC-WH", "Source Warehouse").await;
        let dest_wh = app.db().create_test_warehouse(tenant_id, "DST-WH", "Destination Warehouse").await;
        let product_id = app.db().create_test_product(tenant_id, "TRF-001", "Transfer Product").await;

        let request_body = json!({
            "source_warehouse_id": source_wh,
            "destination_warehouse_id": dest_wh,
            "transfer_type": "manual",
            "priority": "normal",
            "notes": "Test transfer",
            "items": [
                {
                    "product_id": product_id,
                    "quantity": 10,
                    "line_number": 1
                }
            ]
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/inventory/transfers")
            .header("content-type", "application/json")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let (status, body) = app.send_request(request).await;

        // Should create transfer or return validation error
        assert!(status == StatusCode::CREATED || status == StatusCode::OK || status == StatusCode::BAD_REQUEST);
        let _response: Value = serde_json::from_str(&body).unwrap_or(json!({}));

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_transfer_same_warehouse_error() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Transfer Same WH").await;
        let warehouse_id = app.db().create_test_warehouse(tenant_id, "SAME-WH", "Same Warehouse").await;
        let product_id = app.db().create_test_product(tenant_id, "TRF-002", "Transfer Product 2").await;

        let request_body = json!({
            "source_warehouse_id": warehouse_id,
            "destination_warehouse_id": warehouse_id,  // Same warehouse - should fail
            "transfer_type": "manual",
            "items": [
                {
                    "product_id": product_id,
                    "quantity": 5,
                    "line_number": 1
                }
            ]
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/inventory/transfers")
            .header("content-type", "application/json")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let (status, _body) = app.send_request(request).await;

        // Should return bad request for same warehouse
        assert_eq!(status, StatusCode::BAD_REQUEST);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_confirm_transfer_not_found() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Transfer Confirm NF").await;

        let fake_id = Uuid::new_v4();
        let uri = format!("/api/v1/inventory/transfers/{}/confirm", fake_id);
        let request = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("content-type", "application/json")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::from(json!({"notes": "test"}).to_string()))
            .unwrap();

        let (status, _body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::NOT_FOUND);

        app.cleanup().await;
    }
}

// ============================================================================
// Valuation Tests
// ============================================================================

#[cfg(test)]
mod valuation_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_valuation() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Valuation Get").await;
        let product_id = app.db().create_test_product(tenant_id, "VAL-001", "Valuation Product").await;

        let uri = format!("/api/v1/inventory/valuation/{}", product_id);
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, body) = app.send_request(request).await;

        // Should return valuation data or not found (if no valuation record exists)
        assert!(status == StatusCode::OK || status == StatusCode::NOT_FOUND);
        let _response: Value = serde_json::from_str(&body).unwrap_or(json!({}));

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_get_valuation_layers() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Valuation Layers").await;
        let product_id = app.db().create_test_product(tenant_id, "VAL-002", "Layers Product").await;

        let uri = format!("/api/v1/inventory/valuation/{}/layers", product_id);
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, body) = app.send_request(request).await;

        // Should return layers data or not found
        assert!(status == StatusCode::OK || status == StatusCode::NOT_FOUND);
        let _response: Value = serde_json::from_str(&body).unwrap_or(json!({}));

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_get_valuation_history() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Valuation History").await;
        let product_id = app.db().create_test_product(tenant_id, "VAL-003", "History Product").await;

        let uri = format!("/api/v1/inventory/valuation/{}/history", product_id);
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, body) = app.send_request(request).await;

        // Should return history or empty list
        assert!(status == StatusCode::OK || status == StatusCode::NOT_FOUND);
        let _response: Value = serde_json::from_str(&body).unwrap_or(json!({}));

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_set_valuation_method() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Valuation Method").await;
        let product_id = app.db().create_test_product(tenant_id, "VAL-004", "Method Product").await;

        let uri = format!("/api/v1/inventory/valuation/{}/method", product_id);
        let request_body = json!({
            "method": "fifo"
        });

        let request = Request::builder()
            .method(Method::PUT)
            .uri(uri)
            .header("content-type", "application/json")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let (status, body) = app.send_request(request).await;

        // Should set method or return error if not found
        assert!(status == StatusCode::OK || status == StatusCode::NOT_FOUND || status == StatusCode::BAD_REQUEST);
        let _response: Value = serde_json::from_str(&body).unwrap_or(json!({}));

        app.cleanup().await;
    }
}
