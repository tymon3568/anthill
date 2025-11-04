use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    response::Response,
    routing::{get, post},
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use user_service_api::AppState;
use user_service_core::domains::auth::{
    domain::{
        model::{Tenant, User},
        repository::{TenantRepository, UserRepository},
    },
    dto::auth_dto::RegisterReq,
};
use user_service_infra::auth::{
    AuthServiceImpl, PgSessionRepository, PgTenantRepository, PgUserRepository,
};
use uuid::Uuid;

/// Test database URL - should be configured in CI/CD
fn get_test_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://anthill:anthill@localhost:5432/anthill_test".to_string())
}

/// Helper function to get JWT secret from environment
fn get_test_jwt_secret() -> String {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "test-secret-key-at-least-32-characters-long".to_string())
}

/// Helper function to setup test database
async fn setup_test_db() -> PgPool {
    let database_url = get_test_database_url();
    let pool = shared_db::init_pool(&database_url, 1)
        .await
        .expect("Failed to connect to test database");

    // Run migrations manually since these are raw SQL files, not sqlx migrations
    let migration_files = vec![
        include_str!("../../../../migrations/20250110000001_initial_extensions.sql"),
        include_str!("../../../../migrations/20250110000002_create_tenants_users.sql"),
        include_str!("../../../../migrations/20250110000003_create_casbin_tables.sql"),
        include_str!("../../../../migrations/20250110000004_seed_default_casbin_policies.sql"),
    ];

    for migration in migration_files {
        sqlx::raw_sql(migration)
            .execute(&pool)
            .await
            .expect("Failed to run migration");
    }

    pool
}

/// Helper function to create test tenant
async fn create_test_tenant(pool: &PgPool, name: &str) -> Tenant {
    let tenant_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let slug = name.to_lowercase().replace(" ", "-");

    let tenant = Tenant {
        tenant_id,
        name: name.to_string(),
        slug,
        plan: "free".to_string(),
        plan_expires_at: None,
        settings: sqlx::types::Json(serde_json::json!({})),
        status: "active".to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
    };

    let tenant_repo = PgTenantRepository::new(pool.clone());
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create test tenant");
    tenant
}

/// Helper function to create test user
async fn create_test_user(
    pool: &PgPool,
    tenant_id: Uuid,
    email: &str,
    full_name: &str,
    role: &str,
) -> User {
    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let password_hash =
        bcrypt::hash("TestPass123!", bcrypt::DEFAULT_COST).expect("Failed to hash password");

    let user = User {
        user_id,
        tenant_id,
        email: email.to_string(),
        password_hash: Some(password_hash),
        email_verified: true,
        email_verified_at: Some(now),
        full_name: Some(full_name.to_string()),
        avatar_url: None,
        phone: None,
        role: role.to_string(),
        status: "active".to_string(),
        last_login_at: None,
        failed_login_attempts: 0,
        locked_until: None,
        password_changed_at: Some(now),
        kanidm_user_id: None,
        kanidm_synced_at: None,
        auth_method: "password".to_string(),
        migration_invited_at: None,
        migration_completed_at: None,
        created_at: now,
        updated_at: now,
        deleted_at: None,
    };

    let user_repo = PgUserRepository::new(pool.clone());
    user_repo
        .create(&user)
        .await
        .expect("Failed to create test user");
    user
}

/// Helper function to create JWT token for user
fn create_test_jwt(user_id: Uuid, tenant_id: Uuid, role: &str) -> String {
    use shared_jwt::{encode_jwt, Claims};

    let claims = Claims::new_access(user_id, tenant_id, role.to_string(), 900);
    let jwt_secret = get_test_jwt_secret();

    encode_jwt(&claims, &jwt_secret).expect("Failed to create JWT")
}

/// Helper function to make authenticated request
async fn make_authenticated_request(
    router: &Router,
    method: &str,
    path: &str,
    token: &str,
    body: Option<Value>,
) -> Response<Body> {
    let request_body = body
        .map(|b| serde_json::to_string(&b).unwrap())
        .unwrap_or_default();

    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(request_body))
        .expect("Failed to build request");

    router
        .clone()
        .oneshot(request)
        .await
        .expect("Failed to execute request")
}

#[tokio::test]
#[ignore] // Integration test - requires database
async fn test_tenant_isolation_users_cannot_see_other_tenants() {
    // Setup test database
    let pool = setup_test_db().await;

    // Create two tenants
    let tenant_a = create_test_tenant(&pool, "Tenant A").await;
    let tenant_b = create_test_tenant(&pool, "Tenant B").await;

    // Create users in each tenant
    let user_a =
        create_test_user(&pool, tenant_a.tenant_id, "user_a@test.com", "User A", "user").await;
    let user_b =
        create_test_user(&pool, tenant_b.tenant_id, "user_b@test.com", "User B", "user").await;

    // Setup auth service and router
    let user_repo = PgUserRepository::new(pool.clone());
    let tenant_repo = PgTenantRepository::new(pool.clone());
    let session_repo = PgSessionRepository::new(pool.clone());

    let auth_service = AuthServiceImpl::new(
        user_repo.clone(),
        tenant_repo.clone(),
        session_repo,
        get_test_jwt_secret(),
        900,    // 15 minutes
        604800, // 7 days
    );

    let state = AppState {
        auth_service: Arc::new(auth_service),
        enforcer: shared_auth::enforcer::create_enforcer(&get_test_database_url(), None)
            .await
            .expect("Failed to create enforcer"),
        jwt_secret: get_test_jwt_secret(),
        kanidm_client: shared_kanidm_client::KanidmClient::new(
            shared_kanidm_client::KanidmConfig {
                kanidm_url: "http://localhost:8300".to_string(),
                client_id: "dev".to_string(),
                client_secret: "dev".to_string(),
                redirect_uri: "http://localhost:3000/oauth/callback".to_string(),
                scopes: vec!["openid".to_string()],
                skip_jwt_verification: true,
                allowed_issuers: vec!["http://localhost:8300".to_string()],
                expected_audience: Some("dev".to_string()),
            },
        )
        .expect("Failed to create dev Kanidm client"),
        user_repo: Some(Arc::new(user_repo)),
        tenant_repo: Some(Arc::new(tenant_repo)),
    };

    let app = Router::new()
        .route("/api/v1/users", get(user_service_api::handlers::list_users))
        .route("/api/v1/users/:user_id", get(user_service_api::handlers::get_user))
        .with_state(state);

    // Create JWT tokens
    let token_a = create_test_jwt(user_a.user_id, tenant_a.tenant_id, &user_a.role);
    let token_b = create_test_jwt(user_b.user_id, tenant_b.tenant_id, &user_b.role);

    // Test 1: User A should NOT be able to see User B's details
    let response = make_authenticated_request(
        &app,
        "GET",
        &format!("/api/v1/users/{}", user_b.user_id),
        &token_a,
        None,
    )
    .await;

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "User A should not be able to access User B from different tenant - expected 404"
    );

    // Test 2: User A should NOT be able to list users from Tenant B
    let response = make_authenticated_request(&app, "GET", "/api/v1/users", &token_a, None).await;

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "User A should be able to list users from their own tenant"
    );

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let users_response: Value = serde_json::from_slice(&body).unwrap();

    // User A should only see themselves in the list
    let users = users_response["users"].as_array().unwrap();
    assert_eq!(users.len(), 1, "User A should only see one user in their tenant");
    assert_eq!(
        users[0]["id"],
        user_a.user_id.to_string(),
        "User A should only see themselves in the user list"
    );

    // Test 3: User B should NOT be able to see User A's details
    let response = make_authenticated_request(
        &app,
        "GET",
        &format!("/api/v1/users/{}", user_a.user_id),
        &token_b,
        None,
    )
    .await;

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "User B should not be able to access User A from different tenant - expected 404"
    );

    // Test 4: User B should only see themselves in their own tenant
    let response = make_authenticated_request(&app, "GET", "/api/v1/users", &token_b, None).await;

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "User B should be able to list users from their own tenant"
    );

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let users_response: Value = serde_json::from_slice(&body).unwrap();

    let users = users_response["users"].as_array().unwrap();
    assert_eq!(users.len(), 1, "User B should only see one user in their tenant");
    assert_eq!(
        users[0]["id"],
        user_b.user_id.to_string(),
        "User B should only see themselves in the user list"
    );
}

#[tokio::test]
#[ignore] // Integration test - requires database
async fn test_cross_tenant_access_admin_cannot_access_other_tenant_users() {
    // Setup test database
    let pool = setup_test_db().await;

    // Create two tenants
    let tenant_a = create_test_tenant(&pool, "Tenant A Corp").await;
    let tenant_b = create_test_tenant(&pool, "Tenant B Corp").await;

    // Create admin users in each tenant
    let admin_a =
        create_test_user(&pool, tenant_a.tenant_id, "admin_a@test.com", "Admin A", "admin").await;
    let user_b =
        create_test_user(&pool, tenant_b.tenant_id, "user_b@test.com", "User B", "user").await;

    // Setup auth service and router (same as above)
    let user_repo = PgUserRepository::new(pool.clone());
    let tenant_repo = PgTenantRepository::new(pool.clone());
    let session_repo = PgSessionRepository::new(pool.clone());

    let auth_service = AuthServiceImpl::new(
        user_repo.clone(),
        tenant_repo.clone(),
        session_repo,
        get_test_jwt_secret(),
        900,
        604800,
    );

    let state = AppState {
        auth_service: Arc::new(auth_service),
        enforcer: shared_auth::enforcer::create_enforcer(&get_test_database_url(), None)
            .await
            .expect("Failed to create enforcer"),
        jwt_secret: get_test_jwt_secret(),
        kanidm_client: shared_kanidm_client::KanidmClient::new(
            shared_kanidm_client::KanidmConfig {
                kanidm_url: "http://localhost:8300".to_string(),
                client_id: "dev".to_string(),
                client_secret: "dev".to_string(),
                redirect_uri: "http://localhost:3000/oauth/callback".to_string(),
                scopes: vec!["openid".to_string()],
                skip_jwt_verification: true,
                allowed_issuers: vec!["http://localhost:8300".to_string()],
                expected_audience: Some("dev".to_string()),
            },
        )
        .expect("Failed to create dev Kanidm client"),
        user_repo: Some(Arc::new(user_repo)),
        tenant_repo: Some(Arc::new(tenant_repo)),
    };

    let app = Router::new()
        .route("/api/v1/users", get(user_service_api::handlers::list_users))
        .route("/api/v1/users/:user_id", get(user_service_api::handlers::get_user))
        .with_state(state);

    // Create JWT tokens
    let token_admin_a = create_test_jwt(admin_a.user_id, tenant_a.tenant_id, &admin_a.role);
    let token_user_b = create_test_jwt(user_b.user_id, tenant_b.tenant_id, &user_b.role);

    // Test: Admin A should NOT be able to access User B from different tenant
    let response = make_authenticated_request(
        &app,
        "GET",
        &format!("/api/v1/users/{}", user_b.user_id),
        &token_admin_a,
        None,
    )
    .await;

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Admin A should not be able to access User B from different tenant - expected 404"
    );

    // Test: Admin A should NOT see User B in their user list
    let response =
        make_authenticated_request(&app, "GET", "/api/v1/users", &token_admin_a, None).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let users_response: Value = serde_json::from_slice(&body).unwrap();

    let users = users_response["users"].as_array().unwrap();
    assert_eq!(users.len(), 1, "Admin A should only see users from their own tenant");
    assert_eq!(
        users[0]["id"],
        admin_a.user_id.to_string(),
        "Admin A should only see themselves in the user list"
    );

    // Test: User B should NOT be able to access Admin A (even though User B is regular user)
    let response = make_authenticated_request(
        &app,
        "GET",
        &format!("/api/v1/users/{}", admin_a.user_id),
        &token_user_b,
        None,
    )
    .await;

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "User B should not be able to access Admin A from different tenant - expected 404"
    );
}

#[tokio::test]
#[ignore] // Integration test - requires database
async fn test_tenant_isolation_with_multiple_users_per_tenant() {
    // Setup test database
    let pool = setup_test_db().await;

    // Create two tenants
    let tenant_a = create_test_tenant(&pool, "Multi-User Tenant A").await;
    let tenant_b = create_test_tenant(&pool, "Multi-User Tenant B").await;

    // Create multiple users in each tenant
    let user_a1 =
        create_test_user(&pool, tenant_a.tenant_id, "user_a1@test.com", "User A1", "user").await;
    let user_a2 =
        create_test_user(&pool, tenant_a.tenant_id, "user_a2@test.com", "User A2", "manager").await;
    let user_a3 =
        create_test_user(&pool, tenant_a.tenant_id, "user_a3@test.com", "User A3", "user").await;

    let user_b1 =
        create_test_user(&pool, tenant_b.tenant_id, "user_b1@test.com", "User B1", "user").await;
    let user_b2 =
        create_test_user(&pool, tenant_b.tenant_id, "user_b2@test.com", "User B2", "user").await;

    // Setup auth service and router (same as above)
    let user_repo = PgUserRepository::new(pool.clone());
    let tenant_repo = PgTenantRepository::new(pool.clone());
    let session_repo = PgSessionRepository::new(pool.clone());

    let auth_service = AuthServiceImpl::new(
        user_repo.clone(),
        tenant_repo.clone(),
        session_repo,
        get_test_jwt_secret(),
        900,
        604800,
    );

    let state = AppState {
        auth_service: Arc::new(auth_service),
        enforcer: shared_auth::enforcer::create_enforcer(&get_test_database_url(), None)
            .await
            .expect("Failed to create enforcer"),
        jwt_secret: get_test_jwt_secret(),
        kanidm_client: shared_kanidm_client::KanidmClient::new(
            shared_kanidm_client::KanidmConfig {
                kanidm_url: "http://localhost:8300".to_string(),
                client_id: "dev".to_string(),
                client_secret: "dev".to_string(),
                redirect_uri: "http://localhost:3000/oauth/callback".to_string(),
                scopes: vec!["openid".to_string()],
                skip_jwt_verification: true,
                allowed_issuers: vec!["http://localhost:8300".to_string()],
                expected_audience: Some("dev".to_string()),
            },
        )
        .expect("Failed to create dev Kanidm client"),
        user_repo: Some(Arc::new(user_repo)),
        tenant_repo: Some(Arc::new(tenant_repo)),
    };

    let app = Router::new()
        .route("/api/v1/users", get(user_service_api::handlers::list_users))
        .route("/api/v1/users/:user_id", get(user_service_api::handlers::get_user))
        .with_state(state);

    // Create JWT tokens
    let token_a1 = create_test_jwt(user_a1.user_id, tenant_a.tenant_id, &user_a1.role);
    let token_b1 = create_test_jwt(user_b1.user_id, tenant_b.tenant_id, &user_b1.role);

    // Test: User A1 should see all 3 users from Tenant A
    let response = make_authenticated_request(&app, "GET", "/api/v1/users", &token_a1, None).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let users_response: Value = serde_json::from_slice(&body).unwrap();

    let users = users_response["users"].as_array().unwrap();
    assert_eq!(users.len(), 3, "User A1 should see all 3 users from Tenant A");

    // Verify all users are from Tenant A
    let user_ids: Vec<String> = users
        .iter()
        .map(|u| u["id"].as_str().unwrap().to_string())
        .collect();
    assert!(user_ids.contains(&user_a1.user_id.to_string()));
    assert!(user_ids.contains(&user_a2.user_id.to_string()));
    assert!(user_ids.contains(&user_a3.user_id.to_string()));

    // Test: User A1 should NOT see any users from Tenant B
    let response = make_authenticated_request(
        &app,
        "GET",
        &format!("/api/v1/users/{}", user_b1.user_id),
        &token_a1,
        None,
    )
    .await;

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "User A1 should not be able to access User B1 from different tenant - expected 404"
    );

    // Test: User B1 should see all 2 users from Tenant B
    let response = make_authenticated_request(&app, "GET", "/api/v1/users", &token_b1, None).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let users_response: Value = serde_json::from_slice(&body).unwrap();

    let users = users_response["users"].as_array().unwrap();
    assert_eq!(users.len(), 2, "User B1 should see all 2 users from Tenant B");

    // Verify all users are from Tenant B
    let user_ids: Vec<String> = users
        .iter()
        .map(|u| u["id"].as_str().unwrap().to_string())
        .collect();
    assert!(user_ids.contains(&user_b1.user_id.to_string()));
    assert!(user_ids.contains(&user_b2.user_id.to_string()));
}
