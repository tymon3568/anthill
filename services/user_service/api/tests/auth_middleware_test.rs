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
    // Create test config directly
    let mut config = Config {
        database_url: "postgres://user:password@localhost:5432/inventory_db".to_string(),
        jwt_secret: "test-secret-key-for-integration-tests".to_string(),
        jwt_expiration: 3600,
        jwt_refresh_expiration: 2592000,
        host: "0.0.0.0".to_string(),
        port: 8000,
    };
    
    let db_pool = helpers::setup_test_db(&mut config).await;
    let app = get_app(db_pool.clone(), &config).await;
    (app, db_pool, config)
}

#[tokio::test]
async fn test_admin_can_access_admin_route() {
    let (app, db_pool, config) = setup_test_app().await;

    let admin_user = sqlx::query!("SELECT user_id, tenant_id FROM users WHERE email = 'admin@test.com'")
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch admin user");

    let admin_token =
        helpers::generate_jwt(admin_user.user_id, admin_user.tenant_id, "role:admin", &config);

    let request = Request::builder()
        .uri("/api/v1/admin/policies")
        .method(http::Method::POST)
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", admin_token),
        )
        .body(Body::from(
            json!({
                "ptype": "p",
                "v0": "role:manager",
                "v1": admin_user.tenant_id.to_string(),
                "v2": "/api/v1/users",
                "v3": "GET"
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
        sqlx::query!("SELECT user_id, tenant_id FROM users WHERE email = 'manager@test.com'")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch manager user");

    let manager_token = helpers::generate_jwt(
        manager_user.user_id,
        manager_user.tenant_id,
        "role:manager",
        &config,
    );

    let request = Request::builder()
        .uri("/api/v1/admin/policies")
        .method(http::Method::POST)
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", manager_token),
        )
        .body(Body::from(
            json!({
                "ptype": "p",
                "v0": "role:manager",
                "v1": manager_user.tenant_id.to_string(),
                "v2": "/api/v1/users",
                "v3": "GET"
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

    let user = sqlx::query!("SELECT user_id, tenant_id FROM users WHERE email = 'user@test.com'")
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch user");

    let user_token = helpers::generate_jwt(user.user_id, user.tenant_id, "role:user", &config);

    let request = Request::builder()
        .uri("/api/v1/users")
        .method(http::Method::GET)
        .header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", user_token),
        )
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_tenant_isolation() {
    let (app, db_pool, config) = setup_test_app().await;

    let user_a = sqlx::query!("SELECT user_id, tenant_id FROM users WHERE email = 'user@test.com'")
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch user_a");

    let user_b = sqlx::query!("SELECT user_id FROM users WHERE email = 'user_b@test.com'")
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch user_b");

    let user_a_token = helpers::generate_jwt(user_a.user_id, user_a.tenant_id, "role:user", &config);

    let request = Request::builder()
        .uri(format!("/api/v1/users/{}", user_b.user_id))
        .method(http::Method::GET)
        .header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", user_a_token),
        )
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_tenant_isolation_reverse() {
    let (app, db_pool, config) = setup_test_app().await;

    let user_a = sqlx::query!("SELECT user_id FROM users WHERE email = 'user@test.com'")
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch user_a");

    let user_b = sqlx::query!("SELECT user_id, tenant_id FROM users WHERE email = 'user_b@test.com'")
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch user_b");

    let user_b_token = helpers::generate_jwt(user_b.user_id, user_b.tenant_id, "role:user", &config);

    let request = Request::builder()
        .uri(format!("/api/v1/users/{}", user_a.user_id))
        .method(http::Method::GET)
        .header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", user_b_token),
        )
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_list_users_tenant_isolation() {
    let (app, db_pool, config) = setup_test_app().await;

    let user_a = sqlx::query!("SELECT user_id, tenant_id FROM users WHERE email = 'user@test.com'")
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch user_a");

    let user_a_token = helpers::generate_jwt(user_a.user_id, user_a.tenant_id, "role:user", &config);

    let request = Request::builder()
        .uri("/api/v1/users")
        .method(http::Method::GET)
        .header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", user_a_token),
        )
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let users: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    assert_eq!(users.len(), 3);
    for user in users {
        assert_eq!(user["tenant_id"], user_a.tenant_id.to_string());
    }
}
