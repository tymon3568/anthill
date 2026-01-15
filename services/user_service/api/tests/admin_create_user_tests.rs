// Admin Create User Integration Tests
// Tests POST /api/v1/admin/users endpoint
// Run: docker-compose -f docker-compose.test.yml up -d && cargo test --test admin_create_user_tests -- --ignored

mod test_database;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use test_database::TestDatabaseConfig;
use tower::ServiceExt;
use uuid::Uuid;

/// Test helper to create app router
async fn create_test_app(pool: &sqlx::PgPool) -> axum::Router {
    use std::sync::Arc;
    use user_service_api::AppState;
    use user_service_infra::auth::{
        AuthServiceImpl, PgSessionRepository, PgTenantRepository, PgUserRepository,
    };

    let user_repo = Arc::new(PgUserRepository::new(pool.clone()));
    let tenant_repo = Arc::new(PgTenantRepository::new(pool.clone()));
    let session_repo = PgSessionRepository::new(pool.clone());

    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "test-secret-key-at-least-32-characters-long".to_string());

    let auth_service = AuthServiceImpl::new(
        (*user_repo).clone(),
        (*tenant_repo).clone(),
        session_repo,
        jwt_secret.clone(),
        900,    // 15 minutes
        604800, // 7 days
    );

    let database_url = TestDatabaseConfig::get_test_database_url();

    let model_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../shared/auth/model.conf")
        .canonicalize()
        .expect("Failed to resolve shared/auth/model.conf");

    let state = AppState {
        auth_service: Arc::new(auth_service),
        enforcer: shared_auth::enforcer::create_enforcer(
            &database_url,
            Some(model_path.to_str().expect("Invalid path")),
        )
        .await
        .expect("Failed to create enforcer"),
        jwt_secret: jwt_secret.clone(),
        user_repo: Some(user_repo),
        tenant_repo: Some(tenant_repo),
        invitation_service: None,
        config: shared_config::Config {
            database_url,
            jwt_secret,
            ..Default::default()
        },
    };

    user_service_api::create_router(&state)
}

/// Helper to create JWT for authenticated requests
fn create_jwt(user_id: Uuid, tenant_id: Uuid, role: &str) -> String {
    use shared_jwt::{encode_jwt, Claims};

    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "test-secret-key-at-least-32-characters-long".to_string());

    let claims = Claims::new_access(user_id, tenant_id, role.to_string(), 900);
    encode_jwt(&claims, &jwt_secret).expect("Failed to create JWT")
}

/// Helper to make HTTP request
async fn make_request(
    app: &axum::Router,
    method: &str,
    path: &str,
    body: Option<Value>,
    auth_token: Option<&str>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("Content-Type", "application/json");

    if let Some(token) = auth_token {
        request = request.header("Authorization", format!("Bearer {}", token));
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

// ============================================================================
// ADMIN CREATE USER TESTS
// ============================================================================

/// Test: Admin can successfully create a user in their tenant
#[tokio::test]
#[ignore]
async fn test_admin_create_user_success() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Admin Create User Test", None).await;

    // Create admin user
    let admin_id = db
        .create_user(
            tenant_id,
            "admin@example.com",
            "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.S3Rl4PqhL4LQVC", // bcrypt hash
            "admin",
            Some("Admin User"),
        )
        .await;

    // Add admin role grouping to Casbin
    db.add_casbin_grouping(admin_id, "admin", tenant_id).await;

    let token = create_jwt(admin_id, tenant_id, "admin");

    let payload = json!({
        "email": "newuser@example.com",
        "password": "SecurePass123!",
        "full_name": "New User"
    });

    let (status, response) =
        make_request(&app, "POST", "/api/v1/admin/users", Some(payload), Some(&token)).await;

    println!("📋 Response status: {}", status);
    println!("📋 Response body: {}", serde_json::to_string_pretty(&response).unwrap());

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(response["email"], "newuser@example.com");
    assert_eq!(response["full_name"], "New User");
    assert_eq!(response["role"], "user"); // Default role
    assert_eq!(response["tenant_id"], tenant_id.to_string());
    assert!(response["user_id"].is_string());
    assert!(response["created_at"].is_string());
    assert_eq!(response["message"], "User created successfully");

    // Verify user was created in database
    assert!(db.email_exists(tenant_id, "newuser@example.com").await);

    db.cleanup().await;
}

/// Test: Admin can create user with custom role
#[tokio::test]
#[ignore]
async fn test_admin_create_user_with_role() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Admin Create User Role Test", None).await;

    let admin_id = db
        .create_user(
            tenant_id,
            "admin@example.com",
            "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.S3Rl4PqhL4LQVC",
            "admin",
            Some("Admin User"),
        )
        .await;

    db.add_casbin_grouping(admin_id, "admin", tenant_id).await;

    let token = create_jwt(admin_id, tenant_id, "admin");

    let payload = json!({
        "email": "manager@example.com",
        "password": "SecurePass123!",
        "full_name": "Manager User",
        "role": "admin"
    });

    let (status, response) =
        make_request(&app, "POST", "/api/v1/admin/users", Some(payload), Some(&token)).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(response["email"], "manager@example.com");
    assert_eq!(response["role"], "admin");

    db.cleanup().await;
}

/// Test: Cannot create user with owner role
#[tokio::test]
#[ignore]
async fn test_admin_create_user_owner_role_forbidden() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Owner Role Test", None).await;

    let admin_id = db
        .create_user(
            tenant_id,
            "admin@example.com",
            "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.S3Rl4PqhL4LQVC",
            "admin",
            Some("Admin User"),
        )
        .await;

    db.add_casbin_grouping(admin_id, "admin", tenant_id).await;

    let token = create_jwt(admin_id, tenant_id, "admin");

    let payload = json!({
        "email": "owner@example.com",
        "password": "SecurePass123!",
        "full_name": "Would-be Owner",
        "role": "owner"
    });

    let (status, response) =
        make_request(&app, "POST", "/api/v1/admin/users", Some(payload), Some(&token)).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(response["error"].as_str().unwrap().contains("owner"));

    // Verify user was NOT created
    assert!(!db.email_exists(tenant_id, "owner@example.com").await);

    db.cleanup().await;
}

/// Test: Non-admin cannot create users
#[tokio::test]
#[ignore]
async fn test_non_admin_cannot_create_user() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Non-Admin Test", None).await;

    let user_id = db
        .create_user(
            tenant_id,
            "user@example.com",
            "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.S3Rl4PqhL4LQVC",
            "user",
            Some("Regular User"),
        )
        .await;

    db.add_casbin_grouping(user_id, "user", tenant_id).await;

    let token = create_jwt(user_id, tenant_id, "user");

    let payload = json!({
        "email": "newuser@example.com",
        "password": "SecurePass123!",
        "full_name": "New User"
    });

    let (status, _response) =
        make_request(&app, "POST", "/api/v1/admin/users", Some(payload), Some(&token)).await;

    // Should be forbidden (403) for non-admin
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Verify user was NOT created
    assert!(!db.email_exists(tenant_id, "newuser@example.com").await);

    db.cleanup().await;
}

/// Test: Duplicate email fails
#[tokio::test]
#[ignore]
async fn test_admin_create_user_duplicate_email() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Duplicate Email Test", None).await;

    let admin_id = db
        .create_user(
            tenant_id,
            "admin@example.com",
            "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.S3Rl4PqhL4LQVC",
            "admin",
            Some("Admin User"),
        )
        .await;

    // Create existing user
    db.create_user(
        tenant_id,
        "existing@example.com",
        "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.S3Rl4PqhL4LQVC",
        "user",
        Some("Existing User"),
    )
    .await;

    db.add_casbin_grouping(admin_id, "admin", tenant_id).await;

    let token = create_jwt(admin_id, tenant_id, "admin");

    let payload = json!({
        "email": "existing@example.com",
        "password": "SecurePass123!",
        "full_name": "Duplicate User"
    });

    let (status, _response) =
        make_request(&app, "POST", "/api/v1/admin/users", Some(payload), Some(&token)).await;

    // Should return conflict (409)
    assert_eq!(status, StatusCode::CONFLICT);

    db.cleanup().await;
}

/// Test: Invalid email validation
#[tokio::test]
#[ignore]
async fn test_admin_create_user_invalid_email() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Invalid Email Test", None).await;

    let admin_id = db
        .create_user(
            tenant_id,
            "admin@example.com",
            "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.S3Rl4PqhL4LQVC",
            "admin",
            Some("Admin User"),
        )
        .await;

    db.add_casbin_grouping(admin_id, "admin", tenant_id).await;

    let token = create_jwt(admin_id, tenant_id, "admin");

    let payload = json!({
        "email": "not-an-email",
        "password": "SecurePass123!",
        "full_name": "Invalid Email User"
    });

    let (status, response) =
        make_request(&app, "POST", "/api/v1/admin/users", Some(payload), Some(&token)).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"]
        .as_str()
        .unwrap()
        .to_lowercase()
        .contains("email"));

    db.cleanup().await;
}

/// Test: Weak password validation
#[tokio::test]
#[ignore]
async fn test_admin_create_user_weak_password() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Weak Password Test", None).await;

    let admin_id = db
        .create_user(
            tenant_id,
            "admin@example.com",
            "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.S3Rl4PqhL4LQVC",
            "admin",
            Some("Admin User"),
        )
        .await;

    db.add_casbin_grouping(admin_id, "admin", tenant_id).await;

    let token = create_jwt(admin_id, tenant_id, "admin");

    let payload = json!({
        "email": "newuser@example.com",
        "password": "short",
        "full_name": "Weak Password User"
    });

    let (status, response) =
        make_request(&app, "POST", "/api/v1/admin/users", Some(payload), Some(&token)).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"]
        .as_str()
        .unwrap()
        .to_lowercase()
        .contains("password"));

    db.cleanup().await;
}

/// Test: Tenant isolation - admin from tenant A cannot create user visible in tenant B
#[tokio::test]
#[ignore]
async fn test_admin_create_user_tenant_isolation() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    // Create two tenants
    let tenant_a = db.create_tenant("Tenant A", None).await;
    let tenant_b = db.create_tenant("Tenant B", None).await;

    // Create admin in tenant A
    let admin_a = db
        .create_user(
            tenant_a,
            "admin_a@example.com",
            "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.S3Rl4PqhL4LQVC",
            "admin",
            Some("Admin A"),
        )
        .await;

    db.add_casbin_grouping(admin_a, "admin", tenant_a).await;

    let token_a = create_jwt(admin_a, tenant_a, "admin");

    // Admin A creates a user - should be in tenant A
    let payload = json!({
        "email": "created_by_a@example.com",
        "password": "SecurePass123!",
        "full_name": "Created By Admin A"
    });

    let (status, response) =
        make_request(&app, "POST", "/api/v1/admin/users", Some(payload), Some(&token_a)).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(response["tenant_id"], tenant_a.to_string());

    // Verify user exists in tenant A
    assert!(db.email_exists(tenant_a, "created_by_a@example.com").await);

    // Verify user does NOT exist in tenant B
    assert!(!db.email_exists(tenant_b, "created_by_a@example.com").await);

    db.cleanup().await;
}

/// Test: Unauthenticated request fails
#[tokio::test]
#[ignore]
async fn test_admin_create_user_unauthorized() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let payload = json!({
        "email": "newuser@example.com",
        "password": "SecurePass123!",
        "full_name": "New User"
    });

    let (status, _response) = make_request(
        &app,
        "POST",
        "/api/v1/admin/users",
        Some(payload),
        None, // No auth token
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    db.cleanup().await;
}

/// Test: Invalid role name format
#[tokio::test]
#[ignore]
async fn test_admin_create_user_invalid_role_format() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Invalid Role Test", None).await;

    let admin_id = db
        .create_user(
            tenant_id,
            "admin@example.com",
            "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.S3Rl4PqhL4LQVC",
            "admin",
            Some("Admin User"),
        )
        .await;

    db.add_casbin_grouping(admin_id, "admin", tenant_id).await;

    let token = create_jwt(admin_id, tenant_id, "admin");

    // Role names must be lowercase alphanumeric with underscores
    let payload = json!({
        "email": "newuser@example.com",
        "password": "SecurePass123!",
        "full_name": "New User",
        "role": "Invalid-Role-Name!"
    });

    let (status, response) =
        make_request(&app, "POST", "/api/v1/admin/users", Some(payload), Some(&token)).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"]
        .as_str()
        .unwrap()
        .to_lowercase()
        .contains("role"));

    db.cleanup().await;
}

/// Test: Creating user without full_name (optional field)
#[tokio::test]
#[ignore]
async fn test_admin_create_user_without_full_name() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("No Full Name Test", None).await;

    let admin_id = db
        .create_user(
            tenant_id,
            "admin@example.com",
            "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.S3Rl4PqhL4LQVC",
            "admin",
            Some("Admin User"),
        )
        .await;

    db.add_casbin_grouping(admin_id, "admin", tenant_id).await;

    let token = create_jwt(admin_id, tenant_id, "admin");

    let payload = json!({
        "email": "minimal@example.com",
        "password": "SecurePass123!"
    });

    let (status, response) =
        make_request(&app, "POST", "/api/v1/admin/users", Some(payload), Some(&token)).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(response["email"], "minimal@example.com");
    assert!(response["full_name"].is_null());
    assert_eq!(response["role"], "user");

    db.cleanup().await;
}
