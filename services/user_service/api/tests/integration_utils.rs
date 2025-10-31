// Integration Test Utilities
// Common utilities and helpers for integration tests with real database

use sqlx::{PgPool, Executor};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Test database manager - handles setup/teardown for integration tests
pub struct TestDatabase {
    pool: PgPool,
    test_tenants: Arc<Mutex<Vec<Uuid>>>,
}

impl TestDatabase {
    /// Create a new test database instance
    pub async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| {
                "postgres://anthill:anthill@localhost:5432/anthill_test".to_string()
            });

        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database. Run: ./scripts/setup-test-db.sh");

        Self {
            pool,
            test_tenants: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get a reference to the database pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Create a test tenant and track it for cleanup
    pub async fn create_test_tenant(&self, name: &str) -> Uuid {
        let tenant_id = Uuid::now_v7();
        let slug = format!("test-{}", name.to_lowercase().replace(" ", "-"));

        sqlx::query!(
            r#"
            INSERT INTO tenants (tenant_id, name, slug, plan, status, settings, created_at, updated_at)
            VALUES ($1, $2, $3, 'free', 'active', '{}'::jsonb, NOW(), NOW())
            "#,
            tenant_id,
            name,
            slug
        )
        .execute(&self.pool)
        .await
        .expect("Failed to create test tenant");

        // Track for cleanup
        self.test_tenants.lock().await.push(tenant_id);

        tenant_id
    }

    /// Create a test user for a tenant
    pub async fn create_test_user(
        &self,
        tenant_id: Uuid,
        email: &str,
        role: &str,
    ) -> Uuid {
        let user_id = Uuid::now_v7();

        sqlx::query!(
            r#"
            INSERT INTO users (
                user_id, tenant_id, email, password_hash, role, status,
                email_verified, email_verified_at, full_name, created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, 'active',
                true, NOW(), $6, NOW(), NOW()
            )
            "#,
            user_id,
            tenant_id,
            email,
            "$argon2id$v=19$m=19456,t=2,p=1$test$test", // Test hash
            role,
            format!("Test User {}", email)
        )
        .execute(&self.pool)
        .await
        .expect("Failed to create test user");

        user_id
    }

    /// Clean up all test data created during this session
    pub async fn cleanup(&self) {
        // Delete all test tenants and cascading data
        let tenant_ids = self.test_tenants.lock().await.clone();

        for tenant_id in tenant_ids {
            // Sessions
            sqlx::query!("DELETE FROM sessions WHERE tenant_id = $1", tenant_id)
                .execute(&self.pool)
                .await
                .ok();

            // User profiles
            sqlx::query!(
                "DELETE FROM user_profiles WHERE user_id IN (SELECT user_id FROM users WHERE tenant_id = $1)",
                tenant_id
            )
            .execute(&self.pool)
            .await
            .ok();

            // Users
            sqlx::query!("DELETE FROM users WHERE tenant_id = $1", tenant_id)
                .execute(&self.pool)
                .await
                .ok();

            // Tenant
            sqlx::query!("DELETE FROM tenants WHERE tenant_id = $1", tenant_id)
                .execute(&self.pool)
                .await
                .ok();
        }

        self.test_tenants.lock().await.clear();
    }

    /// Get snapshot of tenant data for verification
    pub async fn snapshot_tenant(&self, tenant_id: Uuid) -> TenantSnapshot {
        let result = sqlx::query!(
            r#"
            SELECT
                (SELECT COUNT(*) FROM users WHERE tenant_id = $1) as "users_count!",
                (SELECT COUNT(*) FROM sessions WHERE tenant_id = $1) as "sessions_count!",
                (SELECT COUNT(*) FROM user_profiles WHERE user_id IN 
                    (SELECT user_id FROM users WHERE tenant_id = $1)) as "profiles_count!",
                (SELECT status FROM tenants WHERE tenant_id = $1) as "tenant_status!"
            "#,
            tenant_id
        )
        .fetch_one(&self.pool)
        .await
        .expect("Failed to get tenant snapshot");

        TenantSnapshot {
            users_count: result.users_count,
            sessions_count: result.sessions_count,
            profiles_count: result.profiles_count,
            tenant_status: result.tenant_status,
        }
    }

    /// Verify database is in clean state (no leftover test data)
    pub async fn verify_clean(&self) -> bool {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM tenants WHERE slug LIKE 'test-%'"
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(Some(0));

        count.unwrap_or(0) == 0
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        // Cleanup will happen via explicit cleanup() call in tests
        // or via test teardown hooks
    }
}

/// Snapshot of tenant data for verification
#[derive(Debug, Clone, PartialEq)]
pub struct TenantSnapshot {
    pub users_count: i64,
    pub sessions_count: i64,
    pub profiles_count: i64,
    pub tenant_status: String,
}

/// Integration test context - combines database and common test state
pub struct IntegrationTestContext {
    pub db: TestDatabase,
    pub jwt_secret: String,
}

impl IntegrationTestContext {
    /// Create a new integration test context
    pub async fn new() -> Self {
        let db = TestDatabase::new().await;
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "test-secret-key-at-least-32-characters-long".to_string());

        Self { db, jwt_secret }
    }

    /// Create a JWT token for testing
    pub fn create_jwt(&self, user_id: Uuid, tenant_id: Uuid, role: &str) -> String {
        use shared_jwt::{encode_jwt, Claims};

        let exp = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp();
        let claims = Claims::new_access(user_id, tenant_id, role.to_string(), exp);

        encode_jwt(&claims, &self.jwt_secret).expect("Failed to create test JWT")
    }

    /// Cleanup all test data
    pub async fn cleanup(&self) {
        self.db.cleanup().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_creation() {
        let db = TestDatabase::new().await;
        assert!(db.pool().acquire().await.is_ok());
    }

    #[tokio::test]
    async fn test_tenant_creation_and_cleanup() {
        let db = TestDatabase::new().await;

        // Create test tenant
        let tenant_id = db.create_test_tenant("Test Company").await;

        // Verify tenant exists
        let snapshot = db.snapshot_tenant(tenant_id).await;
        assert_eq!(snapshot.tenant_status, "active");

        // Cleanup
        db.cleanup().await;

        // Verify cleanup worked
        let result = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM tenants WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(db.pool())
        .await
        .unwrap();

        assert_eq!(result, Some(0));
    }

    #[tokio::test]
    async fn test_user_creation() {
        let db = TestDatabase::new().await;

        let tenant_id = db.create_test_tenant("User Test Tenant").await;
        let user_id = db.create_test_user(tenant_id, "test@example.com", "user").await;

        // Verify user exists
        let snapshot = db.snapshot_tenant(tenant_id).await;
        assert_eq!(snapshot.users_count, 1);

        // Cleanup
        db.cleanup().await;
    }

    #[tokio::test]
    async fn test_integration_context() {
        let ctx = IntegrationTestContext::new().await;

        // Create test data
        let tenant_id = ctx.db.create_test_tenant("Context Test").await;
        let user_id = ctx.db.create_test_user(tenant_id, "admin@test.com", "admin").await;

        // Create JWT
        let token = ctx.create_jwt(user_id, tenant_id, "admin");
        assert!(!token.is_empty());

        // Cleanup
        ctx.cleanup().await;
    }
}
