use shared_config::Config;
use shared_jwt::{encode_jwt as create_jwt, Claims};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

pub async fn setup_test_db(config: &mut Config) -> PgPool {
    // Generate a unique database name for each test run
    let db_name = format!("test_db_{}", Uuid::new_v4().to_string().replace("-", "_"));

    // Parse the original database URL to replace the database name
    let base_url = if let Some(idx) = config.database_url.rfind('/') {
        &config.database_url[..idx]
    } else {
        &config.database_url
    };

    // Connect to the default database to create the new test database
    let mut conn = PgConnection::connect(&config.database_url)
        .await
        .expect("Failed to connect to default database");

    // Create the new test database
    conn.execute(format!(r#"CREATE DATABASE "{}""#, db_name).as_str())
        .await
        .expect("Failed to create test database");

    // Update the config to use the new test database
    config.database_url = format!("{}/{}", base_url, db_name);

    // Connect to the new test database
    let pool = PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("../../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    seed_test_data(&pool).await;

    pool
}

pub fn generate_jwt(user_id: Uuid, tenant_id: Uuid, role: &str, config: &Config) -> String {
    let claims = Claims::new_access(user_id, tenant_id, role.to_string(), config.jwt_expiration);
    create_jwt(&claims, &config.jwt_secret).unwrap()
}

pub async fn seed_test_data(pool: &PgPool) {
    // Create tenants
    let tenant_a_id = Uuid::new_v4();
    let tenant_b_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO tenants (tenant_id, name, slug) VALUES ($1, 'Tenant A', 'tenant-a'), ($2, 'Tenant B', 'tenant-b')",
        tenant_a_id,
        tenant_b_id
    )
    .execute(pool)
    .await
    .expect("Failed to seed tenants");

    // Create users
    let admin_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let user_b_id = Uuid::new_v4();

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
