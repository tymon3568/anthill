//! Integration tests for Warehouse API endpoints
//!
//! Tests the warehouse CRUD, zones, and locations endpoints with full HTTP request/response cycles.

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
// Shared Test Infrastructure (inline to avoid pre-existing helpers.rs issues)
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

#[allow(dead_code)]
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

    async fn create_test_warehouse(
        &self,
        tenant_id: Uuid,
        code: &str,
        name: &str,
        parent_id: Option<Uuid>,
    ) -> Uuid {
        let warehouse_id = Uuid::now_v7();

        sqlx::query(
            "INSERT INTO warehouses (warehouse_id, tenant_id, code, name, parent_warehouse_id, is_active, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, true, NOW(), NOW())"
        )
        .bind(warehouse_id)
        .bind(tenant_id)
        .bind(code)
        .bind(name)
        .bind(parent_id)
        .execute(&self.pool)
        .await
        .expect("Failed to create test warehouse");

        warehouse_id
    }

    async fn cleanup(&self) {
        let tenant_ids = self.test_tenants.lock().await.clone();

        for tenant_id in tenant_ids {
            let _ = sqlx::query("DELETE FROM warehouse_locations WHERE tenant_id = $1")
                .bind(tenant_id)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM warehouse_zones WHERE tenant_id = $1")
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
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_warehouse() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Warehouse Create").await;

        let request_body = json!({
            "code": "WH-001",
            "name": "Main Warehouse",
            "description": "Primary warehouse"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/inventory/warehouses")
            .header("content-type", "application/json")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let (status, body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::CREATED);
        let response: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(response["code"].as_str().unwrap(), "WH-001");
        assert_eq!(response["name"].as_str().unwrap(), "Main Warehouse");

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_get_warehouse() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Warehouse Get").await;
        let warehouse_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, true, NOW(), NOW())"
        )
        .bind(warehouse_id)
        .bind(tenant_id)
        .bind("GET-WH")
        .bind("Get Test Warehouse")
        .execute(&app.db().pool)
        .await
        .expect("Failed to create test warehouse for GET test");

        let uri = format!("/api/v1/inventory/warehouses/{}", warehouse_id);
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::OK);
        let response: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(response["code"].as_str().unwrap(), "GET-WH");

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_list_warehouses() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Warehouse List").await;

        let warehouse_id_1 = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, true, NOW(), NOW())"
        )
        .bind(warehouse_id_1)
        .bind(tenant_id)
        .bind("LIST-01")
        .bind("Warehouse A")
        .execute(&app.db().pool)
        .await
        .expect("Failed to create test warehouse LIST-01 for list test");

        let warehouse_id_2 = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, true, NOW(), NOW())"
        )
        .bind(warehouse_id_2)
        .bind(tenant_id)
        .bind("LIST-02")
        .bind("Warehouse B")
        .execute(&app.db().pool)
        .await
        .expect("Failed to create test warehouse LIST-02 for list test");

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/inventory/warehouses")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::OK);
        let response: Value = serde_json::from_str(&body).unwrap();
        let warehouses = response.as_array().unwrap();
        assert!(warehouses.len() >= 2);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_update_warehouse() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Warehouse Update").await;
        let warehouse_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, true, NOW(), NOW())"
        )
        .bind(warehouse_id)
        .bind(tenant_id)
        .bind("UPD-WH")
        .bind("Original Name")
        .execute(&app.db().pool)
        .await
        .expect("Failed to create test warehouse for UPDATE test");

        let update_body = json!({
            "code": "UPD-WH",
            "name": "Updated Name",
            "description": "Updated description"
        });

        let uri = format!("/api/v1/inventory/warehouses/{}", warehouse_id);
        let request = Request::builder()
            .method(Method::PUT)
            .uri(uri)
            .header("content-type", "application/json")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::from(update_body.to_string()))
            .unwrap();

        let (status, body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::OK);
        let response: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(response["name"].as_str().unwrap(), "Updated Name");

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_delete_warehouse() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Warehouse Delete").await;
        let warehouse_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, true, NOW(), NOW())"
        )
        .bind(warehouse_id)
        .bind(tenant_id)
        .bind("DEL-WH")
        .bind("To Delete")
        .execute(&app.db().pool)
        .await
        .expect("Failed to create test warehouse for DELETE test");

        let uri = format!("/api/v1/inventory/warehouses/{}", warehouse_id);
        let request = Request::builder()
            .method(Method::DELETE)
            .uri(uri)
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, _) = app.send_request(request).await;

        assert_eq!(status, StatusCode::NO_CONTENT);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_warehouse_hierarchy() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Warehouse Hierarchy").await;

        let parent_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, true, NOW(), NOW())"
        )
        .bind(parent_id)
        .bind(tenant_id)
        .bind("PARENT")
        .bind("Parent Warehouse")
        .execute(&app.db().pool)
        .await
        .expect("Failed to create parent test warehouse");

        let child_id_1 = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, parent_warehouse_id, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, true, NOW(), NOW())"
        )
        .bind(child_id_1)
        .bind(tenant_id)
        .bind("CHILD-01")
        .bind("Child A")
        .bind(parent_id)
        .execute(&app.db().pool)
        .await
        .expect("Failed to create child test warehouse 1");

        let child_id_2 = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, parent_warehouse_id, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, true, NOW(), NOW())"
        )
        .bind(child_id_2)
        .bind(tenant_id)
        .bind("CHILD-02")
        .bind("Child B")
        .bind(parent_id)
        .execute(&app.db().pool)
        .await
        .expect("Failed to create child test warehouse 2");

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/inventory/warehouses/tree")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::OK);
        let response: Value = serde_json::from_str(&body).unwrap();
        let warehouses = response["warehouses"].as_array().unwrap();
        assert!(!warehouses.is_empty());

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_warehouse_not_found() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Warehouse Not Found").await;

        let fake_id = Uuid::new_v4();
        let uri = format!("/api/v1/inventory/warehouses/{}", fake_id);
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, _) = app.send_request(request).await;

        assert_eq!(status, StatusCode::NOT_FOUND);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_create_zone() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Zone Create").await;
        let warehouse_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO warehouses (warehouse_id, tenant_id, warehouse_code, warehouse_name, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, true, NOW(), NOW())"
        )
        .bind(warehouse_id)
        .bind(tenant_id)
        .bind("ZONE-WH")
        .bind("Zone Test Warehouse")
        .execute(&app.db().pool)
        .await
        .expect("Failed to create test warehouse for zone test");

        let request_body = json!({
            "zone_code": "ZONE-A",
            "zone_name": "Receiving Zone",
            "zone_type": "receiving",
            "description": "Zone for receiving goods"
        });

        let uri = format!("/api/v1/inventory/warehouses/{}/zones", warehouse_id);
        let request = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("content-type", "application/json")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let (status, body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::CREATED);
        let response: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(response["zone_code"].as_str().unwrap(), "ZONE-A");

        app.cleanup().await;
    }
}
