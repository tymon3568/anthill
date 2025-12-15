#![allow(dead_code)]

use axum::{body::Body, http::Request, response::Response, Router};
use serde_json::Value;
use shared_config::Config;
use shared_jwt::{encode_jwt as create_jwt, Claims};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use user_service_api::AppState;
use user_service_core::domains::auth::domain::model::{Tenant, User};
use user_service_core::domains::auth::domain::repository::{TenantRepository, UserRepository};
use user_service_infra::auth::{
    AuthServiceImpl, PgSessionRepository, PgTenantRepository, PgUserRepository,
};
use uuid::Uuid;

/// Get test database URL from environment or use default
pub fn get_test_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:password@localhost:5432/inventory_db".to_string())
}

/// Get test JWT secret from environment or use default
pub fn get_test_jwt_secret() -> String {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "test-secret-key-at-least-32-characters-long".to_string())
}

/// Clean up all test data from the database
pub async fn cleanup_test_data(pool: &PgPool) {
    // Delete in reverse dependency order to avoid foreign key constraints
    sqlx::query!("DELETE FROM sessions")
        .execute(pool)
        .await
        .ok();
    sqlx::query!("DELETE FROM casbin_rule")
        .execute(pool)
        .await
        .ok();
    sqlx::query!("DELETE FROM users").execute(pool).await.ok();
    sqlx::query!("DELETE FROM tenants").execute(pool).await.ok();
}

/// Setup test database pool
pub async fn setup_test_db() -> PgPool {
    let database_url = get_test_database_url();

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Note: Cleanup is handled per-test to avoid conflicts between parallel tests
    // Migrations should already be run on the test database
    // No need to run them again in tests

    pool
}

/// Create a test tenant
pub async fn create_test_tenant(pool: &PgPool, name: &str) -> Tenant {
    let tenant_id = Uuid::now_v7();
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

/// Create a test user
pub async fn create_test_user(
    pool: &PgPool,
    tenant_id: Uuid,
    email: &str,
    full_name: &str,
    role: &str,
) -> User {
    let user_id = Uuid::now_v7();
    let now = chrono::Utc::now();
    let password_hash =
        bcrypt::hash("TestPass123!", bcrypt::DEFAULT_COST).expect("Failed to hash password");

    let user = User {
        user_id,
        tenant_id,
        email: email.to_string(),
        password_hash: Some(password_hash), // Now Option<String>
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
        created_at: now,
        updated_at: now,
        deleted_at: None,
        // New Phase 4 fields
        kanidm_user_id: None,
        kanidm_synced_at: None,
        auth_method: "password".to_string(), // String, not enum
        migration_invited_at: None,
        migration_completed_at: None,
    };

    let user_repo = PgUserRepository::new(pool.clone());
    user_repo
        .create(&user)
        .await
        .expect("Failed to create test user");

    // Add default Casbin policies for the user's role and assign user to role
    add_default_policies(pool, tenant_id, role, user.user_id).await;

    user
}

/// Create a JWT token for testing
pub fn create_test_jwt(user_id: Uuid, tenant_id: Uuid, role: &str) -> String {
    let claims = Claims::new_access(user_id, tenant_id, role.to_string(), 900);
    let jwt_secret = get_test_jwt_secret();
    create_jwt(&claims, &jwt_secret).expect("Failed to create JWT")
}

/// Add default Casbin policies for a role in a tenant
async fn add_default_policies(pool: &PgPool, tenant_id: Uuid, role: &str, user_id: Uuid) {
    let role_key = format!("role:{}", role);
    let tenant_str = tenant_id.to_string();
    let user_str = user_id.to_string();

    // First, assign user to role (g rule)
    sqlx::query!(
        "INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5) VALUES ('g', $1, $2, $3, '', '', '') ON CONFLICT DO NOTHING",
        user_str, role_key, tenant_str
    )
    .execute(pool)
    .await
    .expect("Failed to assign user to role");

    // Add policies based on role
    match role {
        "admin" => {
            // Admins can do everything in their tenant
            let policies = vec![
                ("p", &role_key, &tenant_str, "/api/v1/users", "GET"),
                ("p", &role_key, &tenant_str, "/api/v1/users/*", "GET"),
                ("p", &role_key, &tenant_str, "/api/v1/users", "POST"),
                ("p", &role_key, &tenant_str, "/api/v1/users/*", "PUT"),
                ("p", &role_key, &tenant_str, "/api/v1/users/*", "DELETE"),
                ("p", &role_key, &tenant_str, "/api/v1/admin/*", "POST"),
            ];

            for (ptype, v0, v1, v2, v3) in policies {
                sqlx::query!(
                    "INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5) VALUES ($1, $2, $3, $4, $5, '', '') ON CONFLICT DO NOTHING",
                    ptype, v0, v1, v2, v3
                )
                .execute(pool)
                .await
                .expect("Failed to insert policy");
            }
        },
        "manager" => {
            // Managers can read and update users
            let policies = vec![
                ("p", &role_key, &tenant_str, "/api/v1/users", "GET"),
                ("p", &role_key, &tenant_str, "/api/v1/users/*", "GET"),
                ("p", &role_key, &tenant_str, "/api/v1/users/*", "PUT"),
            ];

            for (ptype, v0, v1, v2, v3) in policies {
                sqlx::query!(
                    "INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5) VALUES ($1, $2, $3, $4, $5, '', '') ON CONFLICT DO NOTHING",
                    ptype, v0, v1, v2, v3
                )
                .execute(pool)
                .await
                .expect("Failed to insert policy");
            }
        },
        _ => {
            // Regular users can only read
            let policies = vec![
                ("p", &role_key, &tenant_str, "/api/v1/users", "GET"),
                ("p", &role_key, &tenant_str, "/api/v1/users/*", "GET"),
            ];

            for (ptype, v0, v1, v2, v3) in policies {
                sqlx::query!(
                    "INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5) VALUES ($1, $2, $3, $4, $5, '', '') ON CONFLICT DO NOTHING",
                    ptype, v0, v1, v2, v3
                )
                .execute(pool)
                .await
                .expect("Failed to insert policy");
            }
        },
    }
}

/// Create test app with all routes
pub async fn create_test_app(pool: &PgPool) -> Router {
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

    // Create dev Kanidm client
    let kanidm_config = shared_kanidm_client::KanidmConfig {
        kanidm_url: "http://localhost:8300".to_string(),
        client_id: "dev".to_string(),
        client_secret: "dev".to_string(),
        redirect_uri: "http://localhost:8000/oauth/callback".to_string(),
        scopes: vec!["openid".to_string()],
        skip_jwt_verification: true, // TEST MODE
        allowed_issuers: vec!["http://localhost:8300".to_string()],
        expected_audience: Some("dev".to_string()),
    };
    let kanidm_client = shared_kanidm_client::KanidmClient::new(kanidm_config)
        .expect("Failed to create test Kanidm client");

    let state = AppState {
        auth_service: Arc::new(auth_service),
        enforcer: shared_auth::enforcer::create_enforcer(
            &get_test_database_url(),
            Some("../../../shared/auth/model.conf"), // Path from tests/ to workspace root
        )
        .await
        .expect("Failed to create enforcer"),
        jwt_secret: get_test_jwt_secret(),
        kanidm_client,
        user_repo: Some(Arc::new(user_repo)),
        tenant_repo: Some(Arc::new(tenant_repo)),
    };

    user_service_api::create_router(&state)
}

/// Make an authenticated HTTP request
pub async fn make_authenticated_request(
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

/// Make an unauthenticated HTTP request
pub async fn make_unauthenticated_request(
    router: &Router,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> Response<Body> {
    let request_body = body
        .map(|b| serde_json::to_string(&b).unwrap())
        .unwrap_or_default();

    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("Content-Type", "application/json")
        .body(Body::from(request_body))
        .expect("Failed to build request");

    router
        .clone()
        .oneshot(request)
        .await
        .expect("Failed to execute request")
}

pub fn generate_jwt(user_id: Uuid, tenant_id: Uuid, role: &str, config: &Config) -> String {
    let claims = Claims::new_access(user_id, tenant_id, role.to_string(), config.jwt_expiration);
    create_jwt(&claims, &config.jwt_secret).unwrap()
}

pub async fn seed_test_data(pool: &PgPool) {
    // Create tenants
    let tenant_a_id = Uuid::now_v7();
    let tenant_b_id = Uuid::now_v7();
    sqlx::query!(
        "INSERT INTO tenants (tenant_id, name, slug) VALUES ($1, 'Tenant A', 'tenant-a'), ($2, 'Tenant B', 'tenant-b')",
        tenant_a_id,
        tenant_b_id
    )
    .execute(pool)
    .await
    .expect("Failed to seed tenants");

    // Create users
    let admin_id = Uuid::now_v7();
    let manager_id = Uuid::now_v7();
    let user_id = Uuid::now_v7();
    let user_b_id = Uuid::now_v7();

    sqlx::query!(
        "INSERT INTO users (user_id, tenant_id, email, password_hash) VALUES ($1, $2, 'admin@test.com', 'hash'), ($3, $2, 'manager@test.com', 'hash'), ($4, $2, 'user@test.com', 'hash'), ($5, $6, 'user_b@test.com', 'hash')",
        admin_id,
        tenant_a_id,
        manager_id,
        user_id,
        user_b_id,
        tenant_b_id
    )
    .execute(pool)
    .await
    .expect("Failed to seed users");

    // Assign roles
    sqlx::query!(
        "INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES ('g', $1, 'role:admin', $3, ''), ('g', $2, 'role:manager', $3, '')",
        admin_id.to_string(),
        manager_id.to_string(),
        tenant_a_id.to_string()
    )
    .execute(pool)
    .await
    .expect("Failed to assign roles");

    seed_test_policies(pool, tenant_a_id, tenant_b_id).await;
}

pub async fn seed_test_policies(pool: &PgPool, tenant_a_id: Uuid, _tenant_b_id: Uuid) {
    // Admin policies for tenant A
    sqlx::query!(
        "INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES \
        ('p', 'role:admin', $1, '/api/v1/admin/policies', 'POST')",
        tenant_a_id.to_string()
    )
    .execute(pool)
    .await
    .expect("Failed to seed admin policies");

    // User policies for tenant A
    sqlx::query!(
        "INSERT INTO casbin_rule (ptype, v0, v1, v2, v3) VALUES \
        ('p', 'role:user', $1, '/api/v1/users', 'GET')",
        tenant_a_id.to_string()
    )
    .execute(pool)
    .await
    .expect("Failed to seed user policies");
}
