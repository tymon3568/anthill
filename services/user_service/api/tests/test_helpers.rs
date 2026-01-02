use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use user_service_api::AppState;
use user_service_infra::auth::{
    AuthServiceImpl, PgSessionRepository, PgTenantRepository, PgUserRepository,
};

/// Create a test application router with a clean database connection
pub async fn create_test_app(
    db_pool: PgPool,
) -> (
    Router,
    AppState<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>,
) {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://anthill:anthill@localhost:5433/anthill_test".to_string());

    let user_repo = PgUserRepository::new(db_pool.clone());
    let tenant_repo = PgTenantRepository::new(db_pool.clone());
    let session_repo = PgSessionRepository::new(db_pool.clone());

    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "test-secret-key-at-least-32-characters-long".to_string());

    let auth_service = AuthServiceImpl::new(
        user_repo.clone(),
        tenant_repo.clone(),
        session_repo,
        jwt_secret.clone(),
        900,    // 15 minutes
        604800, // 7 days
    );

    let state = AppState {
        auth_service: Arc::new(auth_service),
        enforcer: shared_auth::enforcer::create_enforcer(&database_url, None)
            .await
            .expect("Failed to create enforcer"),
        jwt_secret,
        user_repo: Some(Arc::new(user_repo)),
        tenant_repo: Some(Arc::new(tenant_repo)),
    };

    (user_service_api::create_router(&state), state)
}

/// Helper to make HTTP request
pub async fn make_request(
    app: &Router,
    method: &str,
    path: &str,
    body: Option<Value>,
    auth_token: Option<&str>,
    tenant_id: Option<&str>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("Content-Type", "application/json");

    if let Some(token) = auth_token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    if let Some(tid) = tenant_id {
        request = request.header("X-Tenant-ID", tid);
    }

    let body_str = body
        .map(|b| serde_json::to_string(&b).unwrap())
        .unwrap_or_default();
    let request = request.body(Body::from(body_str)).unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap_or(json!({}));

    (status, json)
}
