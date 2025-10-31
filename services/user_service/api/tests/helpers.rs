use axum::{
    body::Body,
    http::{Request, Response},
    Router,
};
use http_body_util::BodyExt;
use serde_json::Value;
use shared_config::Config;
use shared_jwt::{encode_jwt as create_jwt, Claims};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::Arc;
use tower::ServiceExt;
use user_service_api::AppState;
use user_service_core::domains::auth::domain::model::{Tenant, User};
use user_service_infra::auth::{
    AuthServiceImpl, PgSessionRepository, PgTenantRepository, PgUserRepository,
};
use uuid::Uuid;

/// Get test database URL from environment or use default
pub fn get_test_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://anthill:anthill@localhost:5432/anthill_test".to_string())
}

/// Get test JWT secret from environment or use default
pub fn get_test_jwt_secret() -> String {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "test-secret-key-at-least-32-characters-long".to_string())
}

/// Setup test database with migrations
pub async fn setup_test_db() -> PgPool {
    let database_url = get_test_database_url();
    let pool = shared_db::init_pool(&database_url, 1)
        .await
        .expect("Failed to connect to test database");

    // Run migrations manually since these are raw SQL files
    let migration_files = vec![
        include_str!("../../../../migrations/20250110000001_initial_extensions.sql"),
        include_str!("../../../../migrations/20250110000002_create_tenants_users.sql"),
        include_str!("../../../../migrations/20250110000003_create_casbin_tables.sql"),
        include_str!("../../../../migrations/20250110000004_seed_default_casbin_policies.sql"),
        include_str!("../../../../migrations/20250110000010_create_user_profiles.sql"),
    ];

    for migration in migration_files {
        sqlx::raw_sql(migration)
            .execute(&pool)
            .await
            .expect("Failed to run migration");
    }

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
        password_hash,
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
    };

    let user_repo = PgUserRepository::new(pool.clone());
    user_repo
        .create(&user)
        .await
        .expect("Failed to create test user");
    user
}

/// Create a JWT token for testing
pub fn create_test_jwt(user_id: Uuid, tenant_id: Uuid, role: &str) -> String {
    let claims = Claims::new_access(user_id, tenant_id, role.to_string(), 900);
    let jwt_secret = get_test_jwt_secret();
    create_jwt(&claims, &jwt_secret).expect("Failed to create JWT")
}

/// Create test app with all routes
pub async fn create_test_app(pool: &PgPool) -> Router {
    let user_repo = PgUserRepository::new(pool.clone());
    let tenant_repo = PgTenantRepository::new(pool.clone());
    let session_repo = PgSessionRepository::new(pool.clone());

    let auth_service = AuthServiceImpl::new(
        user_repo,
        tenant_repo,
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
    };

    user_service_api::create_router(state)
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

pub async fn seed_test_policies(pool: &PgPool, tenant_a_id: Uuid, tenant_b_id: Uuid) {
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
