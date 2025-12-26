use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    Router,
};
use inventory_service_api::create_app;
use inventory_service_core::dto::category::*;
use serde_json::{self, json};
use shared_config::Config;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;
use uuid::{self, Uuid};

/// Test database manager for inventory service
pub struct InventoryTestDatabase {
    pool: PgPool,
    test_tenants: Arc<Mutex<Vec<Uuid>>>,
}

impl InventoryTestDatabase {
    pub async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://anthill:anthill@localhost:5432/anthill_test".to_string()
        });

        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        Self {
            pool,
            test_tenants: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn create_test_tenant(&self, name: &str) -> Uuid {
        let tenant_id = Uuid::now_v7();
        let slug = format!("test-{}-{}", name.to_lowercase().replace(" ", "-"), tenant_id);

        sqlx::query(
            r#"
            INSERT INTO tenants (tenant_id, name, slug, plan, status, settings, created_at, updated_at)
            VALUES ($1, $2, $3, 'free', 'active', '{}'::jsonb, NOW(), NOW())
            "#,
        )
        .bind(tenant_id)
        .bind(name)
        .bind(slug)
        .execute(&self.pool)
        .await
        .expect("Failed to create test tenant");

        self.test_tenants.lock().await.push(tenant_id);
        tenant_id
    }

    pub async fn create_test_category(
        &self,
        tenant_id: Uuid,
        name: &str,
        parent_id: Option<Uuid>,
    ) -> Uuid {
        let category_id = Uuid::now_v7();
        let path = if let Some(parent) = parent_id {
            // Get parent path and append
            let parent_path: String = sqlx::query_scalar(
                "SELECT path FROM product_categories WHERE category_id = $1 AND tenant_id = $2",
            )
            .bind(parent)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
            .expect("Parent category not found");
            format!("{}/{}", parent_path, category_id)
        } else {
            category_id.to_string()
        };

        let level = path.split('/').count() as i32 - 1;

        sqlx::query(
            r#"
            INSERT INTO product_categories (
                category_id, tenant_id, parent_category_id, name, path, level,
                display_order, is_active, is_visible, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, 0, true, true, NOW(), NOW())
            "#,
        )
        .bind(category_id)
        .bind(tenant_id)
        .bind(parent_id)
        .bind(name)
        .bind(path)
        .bind(level)
        .execute(&self.pool)
        .await
        .expect("Failed to create test category");

        category_id
    }

    pub async fn cleanup(&self) {
        let tenant_ids = self.test_tenants.lock().await.clone();

        for tenant_id in tenant_ids {
            // Clean up in reverse dependency order
            sqlx::query("DELETE FROM product_categories WHERE tenant_id = $1")
                .bind(tenant_id)
                .execute(&self.pool)
                .await
                .ok();
            sqlx::query("DELETE FROM tenants WHERE tenant_id = $1")
                .bind(tenant_id)
                .execute(&self.pool)
                .await
                .ok();
        }

        self.test_tenants.lock().await.clear();
    }
}

/// Test application context
pub struct TestApp {
    router: Router,
    db: InventoryTestDatabase,
}

impl TestApp {
    pub async fn new() -> Self {
        let db = InventoryTestDatabase::new().await;

        // Create a minimal config for testing
        let config = Config {
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://anthill:anthill@localhost:5432/anthill_test".to_string()
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
        };

        let router = create_app(config).await;

        Self { router, db }
    }

    pub async fn send_request(&self, request: Request<Body>) -> (StatusCode, String) {
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        (status, body_str)
    }

    pub fn db(&self) -> &InventoryTestDatabase {
        &self.db
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Method;

    fn create_auth_header(tenant_id: Uuid, user_id: Uuid) -> String {
        // Create a mock JWT for testing - in real tests you'd use proper JWT creation
        format!("Bearer mock-jwt-{}-{}", tenant_id, user_id)
    }

    #[tokio::test]
    async fn test_create_category() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Category Test").await;

        let request_body = json!({
            "name": "Electronics",
            "description": "Electronic products",
            "display_order": 1
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/inventory/categories")
            .header("content-type", "application/json")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let (status, body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::CREATED);
        let response: CategoryResponse = serde_json::from_str(&body).unwrap();
        assert_eq!(response.name, "Electronics");
        assert_eq!(response.tenant_id, tenant_id);

        app.db().cleanup().await;
    }

    #[tokio::test]
    async fn test_list_categories() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("List Test").await;

        // Create some test categories
        app.db()
            .create_test_category(tenant_id, "Electronics", None)
            .await;
        app.db()
            .create_test_category(tenant_id, "Clothing", None)
            .await;

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/inventory/categories")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::OK);
        let response: CategoryListResponse = serde_json::from_str(&body).unwrap();
        assert!(response.categories.len() >= 2);

        app.db().cleanup().await;
    }

    #[tokio::test]
    async fn test_get_category_tree() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Tree Test").await;

        // Create hierarchical categories
        let electronics_id = app
            .db()
            .create_test_category(tenant_id, "Electronics", None)
            .await;
        app.db()
            .create_test_category(tenant_id, "Phones", Some(electronics_id))
            .await;
        app.db()
            .create_test_category(tenant_id, "Laptops", Some(electronics_id))
            .await;

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/inventory/categories/tree")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::OK);
        let tree: Vec<CategoryTreeResponse> = serde_json::from_str(&body).unwrap();
        assert!(!tree.is_empty());

        // Find electronics and check it has children
        let electronics = tree.iter().find(|c| c.name == "Electronics").unwrap();
        assert!(!electronics.children.is_empty());

        app.db().cleanup().await;
    }

    #[tokio::test]
    async fn test_update_category() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Update Test").await;
        let category_id = app
            .db()
            .create_test_category(tenant_id, "Old Name", None)
            .await;

        let update_body = json!({
            "name": "New Name",
            "description": "Updated description"
        });

        let uri = format!("/api/v1/inventory/categories/{}", category_id);
        let request = Request::builder()
            .method(Method::PUT)
            .uri(uri)
            .header("content-type", "application/json")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::from(update_body.to_string()))
            .unwrap();

        let (status, body) = app.send_request(request).await;

        assert_eq!(status, StatusCode::OK);
        let response: CategoryResponse = serde_json::from_str(&body).unwrap();
        assert_eq!(response.name, "New Name");

        app.db().cleanup().await;
    }

    #[tokio::test]
    async fn test_delete_category() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Delete Test").await;
        let category_id = app
            .db()
            .create_test_category(tenant_id, "To Delete", None)
            .await;

        let uri = format!("/api/v1/inventory/categories/{}", category_id);
        let request = Request::builder()
            .method(Method::DELETE)
            .uri(uri)
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, _) = app.send_request(request).await;

        assert_eq!(status, StatusCode::NO_CONTENT);

        app.db().cleanup().await;
    }

    #[tokio::test]
    async fn test_category_not_found() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Not Found Test").await;

        let fake_id = Uuid::new_v4();
        let uri = format!("/api/v1/inventory/categories/{}", fake_id);
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let (status, _) = app.send_request(request).await;

        assert_eq!(status, StatusCode::NOT_FOUND);

        app.db().cleanup().await;
    }

    #[tokio::test]
    async fn test_category_validation() {
        let app = TestApp::new().await;
        let tenant_id = app.db().create_test_tenant("Validation Test").await;

        // Test empty name
        let invalid_body = json!({
            "name": "",
            "description": "Test"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/inventory/categories")
            .header("content-type", "application/json")
            .header("authorization", create_auth_header(tenant_id, Uuid::new_v4()))
            .body(Body::from(invalid_body.to_string()))
            .unwrap();

        let (status, _) = app.send_request(request).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);

        app.db().cleanup().await;
    }
}
