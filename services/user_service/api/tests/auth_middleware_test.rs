mod helpers;

use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use serde_json::json;
use shared_config::Config;
use sqlx::PgPool;
use tower::ServiceExt;
use user_service_api::get_app;

async fn setup_test_app() -> (Router, PgPool, Config) {
    // Calculate workspace root from CARGO_MANIFEST_DIR
    // CARGO_MANIFEST_DIR points to services/user_service/api
    // We need to go up 3 levels to reach workspace root
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let workspace_root = std::path::Path::new(&manifest_dir)
        .ancestors()
        .nth(3)
        .unwrap_or(std::path::Path::new("."));
    let casbin_model_path = workspace_root
        .join("shared/auth/model.conf")
        .to_string_lossy()
        .to_string();

    // Create test config directly
    let config = Config {
        database_url: "postgres://user:password@localhost:5432/inventory_db".to_string(),
        jwt_secret: "test-secret-key-for-integration-tests".to_string(),
        casbin_model_path,
        ..Default::default()
    };

    let db_pool = helpers::setup_test_db().await;

    // Clean up and seed test data
    helpers::cleanup_test_data(&db_pool).await;
    helpers::seed_test_data(&db_pool).await;

    let app = get_app(db_pool.clone(), &config).await;
    (app, db_pool, config)
}

#[tokio::test]
async fn test_admin_can_access_admin_route() {
    let (app, db_pool, config) = setup_test_app().await;

    let admin_user =
        sqlx::query!("SELECT user_id, tenant_id FROM users WHERE email = $1", "admin@test.com")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch admin user");

    let admin_token =
        helpers::generate_jwt(admin_user.user_id, admin_user.tenant_id, "admin", &config);

    let request = Request::builder()
        .uri("/api/v1/admin/policies")
        .method(http::Method::POST)
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", admin_token))
        .body(Body::from(
            json!({
                "role": "role:manager",
                "resource": "/api/v1/users",
                "action": "GET"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_manager_cannot_access_admin_route() {
    let (app, db_pool, config) = setup_test_app().await;

    let manager_user =
        sqlx::query!("SELECT user_id, tenant_id FROM users WHERE email = $1", "manager@test.com")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch manager user");

    let manager_token =
        helpers::generate_jwt(manager_user.user_id, manager_user.tenant_id, "manager", &config);

    let request = Request::builder()
        .uri("/api/v1/admin/policies")
        .method(http::Method::POST)
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", manager_token))
        .body(Body::from(
            json!({
                "role": "role:manager",
                "resource": "/api/v1/users",
                "action": "GET"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_user_can_access_read_only_route() {
    let (app, db_pool, config) = setup_test_app().await;

    let user =
        sqlx::query!("SELECT user_id, tenant_id FROM users WHERE email = $1", "user@test.com")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch user");

    let user_token = helpers::generate_jwt(user.user_id, user.tenant_id, "user", &config);

    let request = Request::builder()
        .uri("/api/v1/users")
        .method(http::Method::GET)
        .header(http::header::AUTHORIZATION, format!("Bearer {}", user_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_tenant_isolation() {
    let (app, db_pool, config) = setup_test_app().await;

    let user_a =
        sqlx::query!("SELECT user_id, tenant_id FROM users WHERE email = $1", "user@test.com")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch user_a");

    let user_b = sqlx::query!("SELECT user_id FROM users WHERE email = $1", "user_b@test.com")
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch user_b");

    let user_a_token = helpers::generate_jwt(user_a.user_id, user_a.tenant_id, "user", &config);

    let request = Request::builder()
        .uri(format!("/api/v1/users/{}", user_b.user_id))
        .method(http::Method::GET)
        .header(http::header::AUTHORIZATION, format!("Bearer {}", user_a_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_tenant_isolation_reverse() {
    let (app, db_pool, config) = setup_test_app().await;

    let user_a = sqlx::query!("SELECT user_id FROM users WHERE email = $1", "user@test.com")
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch user_a");

    let user_b =
        sqlx::query!("SELECT user_id, tenant_id FROM users WHERE email = $1", "user_b@test.com")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch user_b");

    let user_b_token = helpers::generate_jwt(user_b.user_id, user_b.tenant_id, "user", &config);

    let request = Request::builder()
        .uri(format!("/api/v1/users/{}", user_a.user_id))
        .method(http::Method::GET)
        .header(http::header::AUTHORIZATION, format!("Bearer {}", user_b_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_list_users_tenant_isolation() {
    let (app, db_pool, config) = setup_test_app().await;

    let user_a =
        sqlx::query!("SELECT user_id, tenant_id FROM users WHERE email = $1", "user@test.com")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch user_a");

    let user_a_token = helpers::generate_jwt(user_a.user_id, user_a.tenant_id, "user", &config);

    let request = Request::builder()
        .uri("/api/v1/users")
        .method(http::Method::GET)
        .header(http::header::AUTHORIZATION, format!("Bearer {}", user_a_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let resp: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let users = resp["users"].as_array().expect("users should be an array");

    // Tenant A has 3 users: admin, manager, user
    assert_eq!(users.len(), 3);
    for user in users {
        assert_eq!(user["tenant_id"], user_a.tenant_id.to_string());
    }
}
