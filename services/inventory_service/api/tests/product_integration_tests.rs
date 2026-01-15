//! Integration tests for Product Search API endpoints
//!
//! Tests the product search endpoints with full HTTP request/response cycles.

use axum::{
    body::{to_bytes, Body},
    http::{Method, Request, StatusCode},
    Router,
};
use inventory_service_api::create_app;
use serde_json::Value;
use shared_config::Config;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;
use uuid::Uuid;

// ============================================================================
// Shared Test Infrastructure (inline to avoid pre-existing helpers.rs issues)
// ============================================================================

fn test_config() -> Config {
    Config {
        database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://anthill:anthill@localhost:5433/anthill_test".to_string()
        }),
        jwt_secret: std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "test-secret-key-at-least-32-characters-long".to_string()),
        ..Default::default()
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
            let _ = sqlx::query("DELETE FROM products WHERE tenant_id = $1")
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
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_products_empty() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Product Search Empty").await;

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/inventory/products/search")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::OK);
        let response: Value = serde_json::from_str(&body).unwrap();
        assert!(response["products"].as_array().unwrap().is_empty());
        assert_eq!(response["pagination"]["total_items"].as_i64().unwrap(), 0);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_search_products_with_query() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Product Search Query").await;

        app.db()
            .create_test_product(tenant_id, "LAPTOP-001", "Gaming Laptop")
            .await;
        app.db()
            .create_test_product(tenant_id, "LAPTOP-002", "Business Laptop")
            .await;
        app.db()
            .create_test_product(tenant_id, "PHONE-001", "Smartphone")
            .await;

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/inventory/products/search?query=laptop")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::OK);
        let response: Value = serde_json::from_str(&body).unwrap();
        let products = response["products"].as_array().unwrap();
        assert!(products.len() >= 2);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_search_products_with_filters() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Product Search Filters").await;

        app.db()
            .create_test_product(tenant_id, "FILTER-001", "Test Product 1")
            .await;
        app.db()
            .create_test_product(tenant_id, "FILTER-002", "Test Product 2")
            .await;

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/inventory/products/search?inStockOnly=true")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, _body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::OK);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_search_products_pagination() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Product Pagination").await;

        for i in 1..=15 {
            app.db()
                .create_test_product(
                    tenant_id,
                    &format!("PAGE-{:03}", i),
                    &format!("Product {}", i),
                )
                .await;
        }

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/inventory/products/search?page=1&limit=5")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::OK);
        let response: Value = serde_json::from_str(&body).unwrap();
        let products = response["products"].as_array().unwrap();
        assert!(products.len() <= 5);
        assert!(response["pagination"]["has_next"]
            .as_bool()
            .unwrap_or(false));

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_search_products_invalid_price_range() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Invalid Price").await;

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/inventory/products/search?priceMin=1000&priceMax=500")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, _body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_search_suggestions() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Search Suggestions").await;

        app.db()
            .create_test_product(tenant_id, "LAPTOP-001", "Gaming Laptop")
            .await;
        app.db()
            .create_test_product(tenant_id, "LAPTOP-002", "Business Laptop")
            .await;

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/inventory/products/suggestions?query=lap")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, _body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::OK);

        app.cleanup().await;
    }
}
