// API Endpoint Integration Tests
// Tests all user service API endpoints with real database
// Run: docker-compose -f docker-compose.test.yml up -d && cargo test --test api_endpoint_tests -- --ignored

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

    let user_repo = PgUserRepository::new(pool.clone());
    let tenant_repo = PgTenantRepository::new(pool.clone());
    let session_repo = PgSessionRepository::new(pool.clone());

    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "test-secret-key-at-least-32-characters-long".to_string());

    let auth_service = AuthServiceImpl::new(
        user_repo,
        tenant_repo,
        session_repo,
        jwt_secret.clone(),
        900,    // 15 minutes
        604800, // 7 days
    );

    let database_url = TestDatabaseConfig::get_test_database_url();

    // Use CARGO_MANIFEST_DIR to get reliable path to workspace root
    // CARGO_MANIFEST_DIR points to the directory containing the package's Cargo.toml
    // From services/user_service/api, go up 3 levels to workspace root
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
        jwt_secret,
        kanidm_client: shared_kanidm_client::KanidmClient::new(
            shared_kanidm_client::KanidmConfig {
                kanidm_url: "http://localhost:8300".to_string(),
                client_id: "test".to_string(),
                client_secret: "test".to_string(),
                redirect_uri: "http://localhost:8000/oauth/callback".to_string(),
                scopes: vec!["openid".to_string()],
                skip_jwt_verification: true,
                allowed_issuers: vec!["http://localhost:8300".to_string()],
                expected_audience: Some("test".to_string()),
            },
        )
        .expect("Failed to create test Kanidm client"),
        user_repo: None,
        tenant_repo: None,
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
// REGISTRATION & LOGIN TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_user_registration_success() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    // Registration creates a new tenant automatically
    let payload = json!({
        "email": "newuser@example.com",
        "password": "SecurePass123!",
        "full_name": "New User",
        "tenant_name": "Registration Test Corp"
    });

    let (status, response) =
        make_request(&app, "POST", "/api/v1/auth/register", Some(payload), None).await;

    println!("📋 Response status: {}", status);
    println!("📋 Response body: {}", serde_json::to_string_pretty(&response).unwrap());

    assert_eq!(status, StatusCode::CREATED);
    assert!(response["access_token"].is_string());
    assert!(response["refresh_token"].is_string());
    assert_eq!(response["user"]["email"], "newuser@example.com");

    // Extract tenant_id from response for verification
    let tenant_id = Uuid::parse_str(response["user"]["tenant_id"].as_str().unwrap()).unwrap();

    // Verify user was created in database
    assert!(db.email_exists(tenant_id, "newuser@example.com").await);

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_user_registration_duplicate_email() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Duplicate Email Test", None).await;

    // Create first user
    db.create_user(
        tenant_id,
        "existing@example.com",
        "$argon2id$v=19$m=19456,t=2,p=1$test$test",
        "user",
        Some("Existing User"),
    )
    .await;

    // Try to register with same email
    let payload = json!({
        "tenant_id": tenant_id,
        "email": "existing@example.com",
        "password": "SecurePass123!",
        "full_name": "Duplicate User"
    });

    let (status, response) =
        make_request(&app, "POST", "/api/v1/auth/register", Some(payload), None).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert!(response["error"]
        .as_str()
        .unwrap()
        .contains("already exists"));

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_user_login_success() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Login Test Corp", None).await;

    // Create user with bcrypt hash
    let password_hash = bcrypt::hash("TestPass123!", bcrypt::DEFAULT_COST).unwrap();
    db.create_user(tenant_id, "login@example.com", &password_hash, "user", Some("Login User"))
        .await;

    let payload = json!({
        "email": "login@example.com",
        "password": "TestPass123!"
    });

    let (status, response) =
        make_request(&app, "POST", "/api/v1/auth/login", Some(payload), None).await;

    println!("📋 Login response status: {}", status);
    println!("📋 Login response body: {}", serde_json::to_string_pretty(&response).unwrap());

    assert_eq!(status, StatusCode::OK);
    assert!(response["access_token"].is_string());
    assert!(response["refresh_token"].is_string());
    assert_eq!(response["user"]["email"], "login@example.com");

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_user_login_invalid_credentials() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Invalid Login Test", None).await;

    let password_hash = bcrypt::hash("CorrectPass123!", bcrypt::DEFAULT_COST).unwrap();
    db.create_user(tenant_id, "user@example.com", &password_hash, "user", Some("Test User"))
        .await;

    let payload = json!({
        "email": "user@example.com",
        "password": "WrongPassword"
    });

    let (status, response) =
        make_request(&app, "POST", "/api/v1/auth/login", Some(payload), None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(response["error"].as_str().is_some());

    db.cleanup().await;
}

// ============================================================================
// PROFILE MANAGEMENT TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_get_user_profile() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Profile Test Corp", None).await;
    let user_id = db
        .create_user(
            tenant_id,
            "profile@example.com",
            "$argon2id$v=19$m=19456,t=2,p=1$test$test",
            "user",
            Some("Profile User"),
        )
        .await;

    let token = create_jwt(user_id, tenant_id, "user");

    let (status, response) = make_request(&app, "GET", "/api/v1/profile", None, Some(&token)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["email"], "profile@example.com");
    assert_eq!(response["full_name"], "Profile User");
    assert_eq!(response["role"], "user");

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_update_user_profile() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Update Profile Test", None).await;
    let user_id = db
        .create_user(
            tenant_id,
            "update@example.com",
            "$argon2id$v=19$m=19456,t=2,p=1$test$test",
            "user",
            Some("Old Name"),
        )
        .await;

    let token = create_jwt(user_id, tenant_id, "user");

    let payload = json!({
        "full_name": "Updated Name",
        "phone": "+1234567890"
    });

    let (status, response) =
        make_request(&app, "PUT", "/api/v1/profile", Some(payload), Some(&token)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["full_name"], "Updated Name");
    assert_eq!(response["phone"], "+1234567890");

    // Verify in database
    let user = db.get_user(user_id).await.unwrap();
    assert_eq!(user.full_name, Some("Updated Name".to_string()));

    db.cleanup().await;
}

// ============================================================================
// ADMIN FUNCTIONALITY TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_admin_list_users() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Admin List Test", None).await;

    // Create admin user
    let admin_id = db
        .create_user(
            tenant_id,
            "admin@example.com",
            "$argon2id$v=19$m=19456,t=2,p=1$test$test",
            "admin",
            Some("Admin User"),
        )
        .await;

    // Create regular users
    for i in 1..=5 {
        db.create_user(
            tenant_id,
            &format!("user{}@example.com", i),
            "$argon2id$v=19$m=19456,t=2,p=1$test$test",
            "user",
            Some(&format!("User {}", i)),
        )
        .await;
    }

    let token = create_jwt(admin_id, tenant_id, "admin");

    let (status, response) =
        make_request(&app, "GET", "/api/v1/admin/users", None, Some(&token)).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response.is_array());
    assert_eq!(response.as_array().unwrap().len(), 6); // 5 users + 1 admin

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_admin_update_user_role() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Admin Role Update Test", None).await;

    let admin_id = db
        .create_user(
            tenant_id,
            "admin@example.com",
            "$argon2id$v=19$m=19456,t=2,p=1$test$test",
            "admin",
            Some("Admin User"),
        )
        .await;

    let user_id = db
        .create_user(
            tenant_id,
            "user@example.com",
            "$argon2id$v=19$m=19456,t=2,p=1$test$test",
            "user",
            Some("Regular User"),
        )
        .await;

    let token = create_jwt(admin_id, tenant_id, "admin");

    let payload = json!({
        "role": "manager"
    });

    let (status, response) = make_request(
        &app,
        "PUT",
        &format!("/api/v1/admin/users/{}/role", user_id),
        Some(payload),
        Some(&token),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["role"], "manager");

    // Verify in database
    let user = db.get_user(user_id).await.unwrap();
    assert_eq!(user.role, "manager");

    db.cleanup().await;
}

// ============================================================================
// AUTHORIZATION TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_unauthorized_access_without_token() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let (status, _response) = make_request(&app, "GET", "/api/v1/profile", None, None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_regular_user_cannot_access_admin_endpoints() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Forbidden Test", None).await;
    let user_id = db
        .create_user(
            tenant_id,
            "regular@example.com",
            "$argon2id$v=19$m=19456,t=2,p=1$test$test",
            "user",
            Some("Regular User"),
        )
        .await;

    let token = create_jwt(user_id, tenant_id, "user");

    let (status, _response) =
        make_request(&app, "GET", "/api/v1/admin/users", None, Some(&token)).await;

    assert_eq!(status, StatusCode::FORBIDDEN);

    db.cleanup().await;
}

// ============================================================================
// TENANT ISOLATION TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_tenant_isolation_users_cannot_see_other_tenants() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    // Create two tenants
    let tenant_a_id = db.create_tenant("Tenant A", None).await;
    let tenant_b_id = db.create_tenant("Tenant B", None).await;

    // Create admin in tenant A
    let admin_a_id = db
        .create_user(
            tenant_a_id,
            "admin-a@example.com",
            "$argon2id$v=19$m=19456,t=2,p=1$test$test",
            "admin",
            Some("Admin A"),
        )
        .await;

    // Create users in tenant B
    for i in 1..=3 {
        db.create_user(
            tenant_b_id,
            &format!("user-b-{}@example.com", i),
            "$argon2id$v=19$m=19456,t=2,p=1$test$test",
            "user",
            Some(&format!("User B {}", i)),
        )
        .await;
    }

    let token = create_jwt(admin_a_id, tenant_a_id, "admin");

    // Admin A should only see users from tenant A
    let (status, response) =
        make_request(&app, "GET", "/api/v1/admin/users", None, Some(&token)).await;

    assert_eq!(status, StatusCode::OK);
    let users = response.as_array().unwrap();
    assert_eq!(users.len(), 1); // Only admin A

    db.cleanup().await;
}

// ============================================================================
// VALIDATION TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_registration_invalid_email() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Invalid Email Test", None).await;

    let payload = json!({
        "tenant_id": tenant_id,
        "email": "not-an-email",
        "password": "SecurePass123!",
        "full_name": "Test User"
    });

    let (status, response) =
        make_request(&app, "POST", "/api/v1/auth/register", Some(payload), None).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        response["error"].as_str().unwrap().contains("email")
            || response["error"].as_str().unwrap().contains("validation")
    );

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_registration_weak_password() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool()).await;

    let tenant_id = db.create_tenant("Weak Password Test", None).await;

    let payload = json!({
        "tenant_id": tenant_id,
        "email": "test@example.com",
        "password": "123", // Too short
        "full_name": "Test User"
    });

    let (status, response) =
        make_request(&app, "POST", "/api/v1/auth/register", Some(payload), None).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        response["error"].as_str().unwrap().contains("password")
            || response["error"].as_str().unwrap().contains("validation")
    );

    db.cleanup().await;
}

// ============================================================================
// CLEANUP HELPER
// ============================================================================

#[tokio::test]
#[ignore]
async fn cleanup_all_test_data_helper() {
    let db = TestDatabaseConfig::new().await;
    db.cleanup_all_test_data().await;

    assert!(db.verify_clean().await);

    println!("✓ All test data cleaned successfully");
}
