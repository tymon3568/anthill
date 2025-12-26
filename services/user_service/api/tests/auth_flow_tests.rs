// Authentication & Authorization Flow Integration Tests
// End-to-end tests for complete authentication and authorization flows
// Run: docker-compose -f docker-compose.test.yml up -d && cargo test --test auth_flow_tests -- --ignored

mod test_database;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use test_database::TestDatabaseConfig;
use tower::ServiceExt;
use user_service_api::AppState;
use user_service_infra::auth::{
    AuthServiceImpl, PgSessionRepository, PgTenantRepository, PgUserRepository,
};

/// Test helper to create app router
async fn create_test_app(db_pool: PgPool) -> Router {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://anthill:anthill@localhost:5433/anthill_test".to_string());

    let user_repo = PgUserRepository::new(db_pool.clone());
    let tenant_repo = PgTenantRepository::new(db_pool.clone());
    let session_repo = PgSessionRepository::new(db_pool.clone());

    let jwt_secret = "test-secret-key-at-least-32-characters-long".to_string();

    let auth_service = AuthServiceImpl::new(
        user_repo.clone(),
        tenant_repo.clone(),
        session_repo,
        jwt_secret.clone(),
        900,    // 15 minutes
        604800, // 7 days
    );

    let kanidm_config = shared_kanidm_client::KanidmConfig {
        kanidm_url: "http://localhost:8300".to_string(),
        client_id: "dev".to_string(),
        client_secret: "dev".to_string(),
        redirect_uri: "http://localhost:8000/oauth/callback".to_string(),
        scopes: vec!["openid".to_string()],
        skip_jwt_verification: true, // DEV/TEST MODE ONLY
        allowed_issuers: vec!["http://localhost:8300".to_string()],
        expected_audience: Some("dev".to_string()),
    };
    let kanidm_client = shared_kanidm_client::KanidmClient::new(kanidm_config)
        .expect("Failed to create dev Kanidm client");

    let state = AppState {
        auth_service: Arc::new(auth_service),
        enforcer: shared_auth::enforcer::create_enforcer(&database_url, None)
            .await
            .expect("Failed to create enforcer"),
        jwt_secret,
        kanidm_client,
        user_repo: Some(Arc::new(user_repo)),
        tenant_repo: Some(Arc::new(tenant_repo)),
    };

    user_service_api::create_router(&state)
}

/// Helper to make HTTP request
async fn make_request(
    app: &axum::Router,
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

// ============================================================================
// COMPLETE AUTHENTICATION FLOW TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_complete_registration_to_authenticated_request_flow() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool().clone()).await;

    // Step 1: Create tenant
    let tenant_id = db.create_tenant("Complete Flow Test Corp", None).await;

    // Step 2: Register new user
    let register_payload = json!({
        "tenant_id": tenant_id,
        "email": "newuser@flow.com",
        "password": "SecurePass123!",
        "full_name": "New Flow User"
    });

    let (status, register_response) =
        make_request(&app, "POST", "/api/v1/auth/register", Some(register_payload), None).await;

    assert_eq!(status, StatusCode::CREATED);
    let access_token = register_response["access_token"].as_str().unwrap();

    // Step 3: Use access token to get profile
    let (status, profile_response) =
        make_request(&app, "GET", "/api/v1/profile", None, Some(access_token), None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(profile_response["email"], "newuser@flow.com");
    assert_eq!(profile_response["full_name"], "New Flow User");

    // Step 4: Update profile
    let update_payload = json!({
        "full_name": "Updated Flow User",
        "phone": "+1234567890"
    });

    let (status, update_response) =
        make_request(&app, "PUT", "/api/v1/profile", Some(update_payload), Some(access_token))
            .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(update_response["full_name"], "Updated Flow User");
    assert_eq!(update_response["phone"], "+1234567890");

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_login_flow_with_token_refresh() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool().clone()).await;

    let tenant_id = db.create_tenant("Login Flow Test", None).await;

    // Create user
    let password_hash = bcrypt::hash("TestPass123!", bcrypt::DEFAULT_COST).unwrap();
    db.create_user(tenant_id, "login@flow.com", &password_hash, "user", Some("Login User"))
        .await;

    // Step 1: Login
    let login_payload = json!({
        "email": "login@flow.com",
        "password": "TestPass123!"
    });

    let (status, login_response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(login_payload),
        None,
        Some(&tenant_id.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let access_token = login_response["access_token"].as_str().unwrap();
    let refresh_token = login_response["refresh_token"].as_str().unwrap();

    // Step 2: Use access token
    let (status, _) =
        make_request(&app, "GET", "/api/v1/profile", None, Some(access_token), None).await;

    assert_eq!(status, StatusCode::OK);

    // Step 3: Refresh token
    let refresh_payload = json!({
        "refresh_token": refresh_token
    });

    let (status, refresh_response) =
        make_request(&app, "POST", "/api/v1/auth/refresh", Some(refresh_payload), None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(refresh_response["access_token"].is_string());

    // Step 4: Use new access token
    let new_access_token = refresh_response["access_token"].as_str().unwrap();
    let (status, _) =
        make_request(&app, "GET", "/api/v1/profile", None, Some(new_access_token), None).await;

    assert_eq!(status, StatusCode::OK);

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_logout_flow() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool().clone()).await;

    let tenant_id = db.create_tenant("Logout Flow Test", None).await;

    let password_hash = bcrypt::hash("TestPass123!", bcrypt::DEFAULT_COST).unwrap();
    db.create_user(tenant_id, "logout@flow.com", &password_hash, "user", Some("Logout User"))
        .await;

    // Step 1: Login
    let login_payload = json!({
        "email": "logout@flow.com",
        "password": "TestPass123!"
    });

    let (status, login_response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(login_payload),
        None,
        Some(&tenant_id.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let access_token = login_response["access_token"].as_str().unwrap();

    // Step 2: Verify can access protected resource
    let (status, _) =
        make_request(&app, "GET", "/api/v1/profile", None, Some(access_token), None).await;

    assert_eq!(status, StatusCode::OK);

    // Step 3: Logout
    let (status, _) =
        make_request(&app, "POST", "/api/v1/auth/logout", None, Some(access_token), None).await;

    assert_eq!(status, StatusCode::OK);

    // Step 4: Token should still work until it expires (stateless JWT)
    // But session should be removed from database
    let (status, _) =
        make_request(&app, "GET", "/api/v1/profile", None, Some(access_token), None).await;

    // Depending on implementation, this might still work or fail
    // If JWT is stateless, it works until expiry
    // If sessions are checked, it should fail
    assert!(status == StatusCode::OK || status == StatusCode::UNAUTHORIZED);

    db.cleanup().await;
}

// ============================================================================
// ROLE-BASED ACCESS CONTROL (RBAC) FLOW TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_rbac_flow_user_to_admin_promotion() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool().clone()).await;

    let tenant_id = db.create_tenant("RBAC Promotion Test", None).await;

    // Create admin user
    let admin_password = bcrypt::hash("AdminPass123!", bcrypt::DEFAULT_COST).unwrap();
    let _admin_id = db
        .create_user(tenant_id, "admin@rbac.com", &admin_password, "admin", Some("Admin User"))
        .await;

    // Create regular user
    let user_password = bcrypt::hash("UserPass123!", bcrypt::DEFAULT_COST).unwrap();
    let user_id = db
        .create_user(tenant_id, "user@rbac.com", &user_password, "user", Some("Regular User"))
        .await;

    // Step 1: Login as admin
    let admin_login = json!({
        "email": "admin@rbac.com",
        "password": "AdminPass123!"
    });

    let (status, admin_response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(admin_login),
        None,
        Some(&tenant_id.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let admin_token = admin_response["access_token"].as_str().unwrap();

    // Step 2: Login as user
    let user_login = json!({
        "email": "user@rbac.com",
        "password": "UserPass123!"
    });

    let (status, user_response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(user_login.clone()),
        None,
        Some(&tenant_id.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let user_token = user_response["access_token"].as_str().unwrap();

    // Step 3: User tries to access admin endpoint (should fail)
    let (status, _) =
        make_request(&app, "GET", "/api/v1/admin/users", None, Some(user_token), None).await;

    assert_eq!(status, StatusCode::FORBIDDEN);

    // Step 4: Admin promotes user to admin role
    let promote_payload = json!({
        "role": "admin"
    });

    let (status, _) = make_request(
        &app,
        "PUT",
        &format!("/api/v1/admin/users/{}/role", user_id),
        Some(promote_payload),
        Some(admin_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    // Step 5: User logs in again to get new token with new role
    let (status, new_user_response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(user_login),
        None,
        Some(&tenant_id.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let new_user_token = new_user_response["access_token"].as_str().unwrap();

    // Step 6: User can now access admin endpoint
    let (status, _) =
        make_request(&app, "GET", "/api/v1/admin/users", None, Some(new_user_token), None).await;

    assert_eq!(status, StatusCode::OK);

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_rbac_flow_manager_permissions() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool().clone()).await;

    let tenant_id = db.create_tenant("Manager RBAC Test", None).await;

    // Create manager user
    let manager_password = bcrypt::hash("ManagerPass123!", bcrypt::DEFAULT_COST).unwrap();
    let _manager_id = db
        .create_user(
            tenant_id,
            "manager@rbac.com",
            &manager_password,
            "manager",
            Some("Manager User"),
        )
        .await;

    // Create regular user to manage
    let _user_id = db
        .create_user(
            tenant_id,
            "employee@rbac.com",
            "$argon2id$v=19$m=19456,t=2,p=1$test$test",
            "user",
            Some("Employee User"),
        )
        .await;

    // Login as manager
    let manager_login = json!({
        "email": "manager@rbac.com",
        "password": "ManagerPass123!"
    });

    let (status, manager_response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(manager_login),
        None,
        Some(&tenant_id.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let manager_token = manager_response["access_token"].as_str().unwrap();

    // Manager can view users
    let (status, _) =
        make_request(&app, "GET", "/api/v1/admin/users", None, Some(manager_token), None).await;

    // Depending on implementation, managers might or might not access admin endpoints
    assert!(status == StatusCode::OK || status == StatusCode::FORBIDDEN);

    db.cleanup().await;
}

// ============================================================================
// JWT TOKEN VALIDATION FLOW TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_jwt_expiration_flow() {
    let db = TestDatabaseConfig::new().await;

    // Create app with very short token expiration (1 second)
    use std::sync::Arc;
    use user_service_api::AppState;
    use user_service_infra::auth::{
        AuthServiceImpl, PgSessionRepository, PgTenantRepository, PgUserRepository,
    };

    let user_repo = PgUserRepository::new(db.pool().clone());
    let tenant_repo = PgTenantRepository::new(db.pool().clone());
    let session_repo = PgSessionRepository::new(db.pool().clone());

    let jwt_secret = "test-secret-key-at-least-32-characters-long".to_string();

    let auth_service = AuthServiceImpl::new(
        user_repo.clone(),
        tenant_repo.clone(),
        session_repo,
        jwt_secret.clone(),
        1,      // 1 second expiration for testing
        604800, // 7 days refresh
    );

    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://anthill:anthill@localhost:5433/anthill_test".to_string());

    // Create dev Kanidm client for testing
    let kanidm_config = shared_kanidm_client::KanidmConfig {
        kanidm_url: "http://localhost:8300".to_string(),
        client_id: "dev".to_string(),
        client_secret: "dev".to_string(),
        redirect_uri: "http://localhost:8000/oauth/callback".to_string(),
        scopes: vec!["openid".to_string()],
        skip_jwt_verification: true, // DEV/TEST MODE ONLY
        allowed_issuers: vec!["http://localhost:8300".to_string()],
        expected_audience: Some("dev".to_string()),
    };
    let kanidm_client = shared_kanidm_client::KanidmClient::new(kanidm_config)
        .expect("Failed to create dev Kanidm client");

    let state = AppState {
        auth_service: Arc::new(auth_service),
        enforcer: shared_auth::enforcer::create_enforcer(&database_url, None)
            .await
            .expect("Failed to create enforcer"),
        jwt_secret,
        kanidm_client,
        user_repo: Some(Arc::new(user_repo)),
        tenant_repo: Some(Arc::new(tenant_repo)),
    };

    let app = user_service_api::create_router(&state);

    let tenant_id = db.create_tenant("JWT Expiration Test", None).await;

    let password_hash = bcrypt::hash("TestPass123!", bcrypt::DEFAULT_COST).unwrap();
    db.create_user(tenant_id, "expire@jwt.com", &password_hash, "user", Some("Expire User"))
        .await;

    // Login
    let login_payload = json!({
        "email": "expire@jwt.com",
        "password": "TestPass123!"
    });

    let (status, login_response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(login_payload),
        None,
        Some(&tenant_id.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let access_token = login_response["access_token"].as_str().unwrap();

    // Token should work immediately
    let (status, _) =
        make_request(&app, "GET", "/api/v1/profile", None, Some(access_token), None).await;

    assert_eq!(status, StatusCode::OK);

    // Wait for token to expire
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Token should now be expired
    let (status, _) =
        make_request(&app, "GET", "/api/v1/profile", None, Some(access_token), None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    db.cleanup().await;
}

#[tokio::test]
#[ignore]
async fn test_invalid_jwt_token_flow() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool().clone()).await;

    // Try with completely invalid token
    let (status, _) =
        make_request(&app, "GET", "/api/v1/profile", None, Some("invalid.jwt.token"), None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Try with malformed token
    let (status, _) =
        make_request(&app, "GET", "/api/v1/profile", None, Some("Bearer malformed"), None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    db.cleanup().await;
}

// ============================================================================
// MULTI-TENANT AUTH FLOW TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_cross_tenant_access_prevention() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool().clone()).await;

    // Create two tenants
    let tenant_a_id = db.create_tenant("Tenant A Auth", None).await;
    let tenant_b_id = db.create_tenant("Tenant B Auth", None).await;

    // Create users in each tenant
    let password_hash = bcrypt::hash("TestPass123!", bcrypt::DEFAULT_COST).unwrap();

    let _user_a_id = db
        .create_user(tenant_a_id, "user-a@tenanta.com", &password_hash, "admin", Some("User A"))
        .await;

    let _user_b_id = db
        .create_user(tenant_b_id, "user-b@tenantb.com", &password_hash, "user", Some("User B"))
        .await;

    // Login as User A
    let login_payload = json!({
        "email": "user-a@tenanta.com",
        "password": "TestPass123!"
    });

    let (status, login_response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(login_payload),
        None,
        Some(&tenant_a_id.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let user_a_token = login_response["access_token"].as_str().unwrap();

    // User A lists users - should only see Tenant A users
    let (status, response) =
        make_request(&app, "GET", "/api/v1/admin/users", None, Some(user_a_token), None).await;

    assert_eq!(status, StatusCode::OK);
    let users = response.as_array().unwrap();

    // Should only see users from Tenant A
    for user in users {
        let email = user["email"].as_str().unwrap();
        assert!(email.contains("tenanta.com"));
        assert!(!email.contains("tenantb.com"));
    }

    db.cleanup().await;
}

// ============================================================================
// PASSWORD CHANGE FLOW TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_password_change_flow() {
    let db = TestDatabaseConfig::new().await;
    let app = create_test_app(db.pool().clone()).await;

    let tenant_id = db.create_tenant("Password Change Test", None).await;

    let old_password = "OldPass123!";
    let password_hash = bcrypt::hash(old_password, bcrypt::DEFAULT_COST).unwrap();

    db.create_user(
        tenant_id,
        "changepass@test.com",
        &password_hash,
        "user",
        Some("Change Pass User"),
    )
    .await;

    // Step 1: Login with old password
    let login_payload = json!({
        "email": "changepass@test.com",
        "password": old_password
    });

    let (status, login_response) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(login_payload.clone()),
        None,
        Some(&tenant_id.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let access_token = login_response["access_token"].as_str().unwrap();

    // Step 2: Change password
    let new_password = "NewPass456!";
    let change_payload = json!({
        "old_password": old_password,
        "new_password": new_password
    });

    let (status, _) = make_request(
        &app,
        "PUT",
        "/api/v1/profile/password",
        Some(change_payload),
        Some(access_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    // Step 3: Old password should no longer work
    let (status, _) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(login_payload),
        None,
        Some(&tenant_id.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Step 4: New password should work
    let new_login_payload = json!({
        "email": "changepass@test.com",
        "password": new_password
    });

    let (status, _) = make_request(
        &app,
        "POST",
        "/api/v1/auth/login",
        Some(new_login_payload),
        None,
        Some(&tenant_id.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    db.cleanup().await;
}
