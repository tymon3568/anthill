// Test Database Module
// Provides test database setup, teardown, and utilities for integration testing
// This module ensures test isolation and proper cleanup

use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Test database configuration and management
pub struct TestDatabaseConfig {
    /// Database connection pool
    pool: PgPool,

    /// Track created resources for cleanup
    tracked_tenants: Arc<Mutex<Vec<Uuid>>>,
    tracked_users: Arc<Mutex<Vec<Uuid>>>,
    tracked_sessions: Arc<Mutex<Vec<Uuid>>>,
    /// Track Casbin grouping policies for cleanup (user_id, role, tenant_id)
    tracked_casbin_rules: Arc<Mutex<Vec<(String, String, String)>>>,

    /// Whether to automatically cleanup on drop
    auto_cleanup: bool,
}

impl TestDatabaseConfig {
    /// Create a new test database configuration
    pub async fn new() -> Self {
        let database_url = Self::get_test_database_url();

        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database. Run: docker-compose -f docker-compose.test.yml up -d && ./scripts/setup-test-db.sh");

        Self {
            pool,
            tracked_tenants: Arc::new(Mutex::new(Vec::new())),
            tracked_users: Arc::new(Mutex::new(Vec::new())),
            tracked_sessions: Arc::new(Mutex::new(Vec::new())),
            tracked_casbin_rules: Arc::new(Mutex::new(Vec::new())),
            auto_cleanup: true,
        }
    }

    /// Create instance without auto-cleanup (for manual control)
    #[allow(dead_code)]
    pub async fn new_no_cleanup() -> Self {
        let mut config = Self::new().await;
        config.auto_cleanup = false;
        config
    }

    /// Get test database URL from environment or use default
    pub fn get_test_database_url() -> String {
        std::env::var("TEST_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .unwrap_or_else(|_| {
                "postgres://anthill:anthill@localhost:5433/anthill_test".to_string()
            })
    }

    /// Get reference to database pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Begin a transaction for testing rollback scenarios
    pub async fn begin_transaction(&self) -> Result<Transaction<'_, Postgres>, sqlx::Error> {
        self.pool.begin().await
    }

    /// Track a tenant for cleanup
    async fn track_tenant(&self, tenant_id: Uuid) {
        self.tracked_tenants.lock().await.push(tenant_id);
    }

    /// Track a user for cleanup
    async fn track_user(&self, user_id: Uuid) {
        self.tracked_users.lock().await.push(user_id);
    }

    /// Track a session for cleanup
    #[allow(dead_code)]
    async fn track_session(&self, session_id: Uuid) {
        self.tracked_sessions.lock().await.push(session_id);
    }

    /// Create a test tenant with automatic tracking
    pub async fn create_tenant(&self, name: &str, slug_suffix: Option<&str>) -> Uuid {
        let tenant_id = Uuid::now_v7();

        // Generate unique slug to avoid conflicts
        let base_slug = name.to_lowercase().replace(" ", "-");
        let slug = match slug_suffix {
            Some(suffix) => format!("{}-{}", base_slug, suffix),
            None => format!("{}-{}", base_slug, &tenant_id.to_string()[..8]),
        };

        // Using runtime queries instead of macros for test compatibility without DB connection at compile time
        sqlx::query(
            r#"
            INSERT INTO tenants (tenant_id, name, slug, plan, status, settings, created_at, updated_at)
            VALUES ($1, $2, $3, 'free', 'active', '{}'::jsonb, NOW(), NOW())
            "#,
        )
        .bind(tenant_id)
        .bind(name)
        .bind(&slug)
        .execute(&self.pool)
        .await
        .expect("Failed to create test tenant");

        self.track_tenant(tenant_id).await;
        tenant_id
    }

    /// Create a test user with automatic tracking
    pub async fn create_user(
        &self,
        tenant_id: Uuid,
        email: &str,
        password_hash: &str,
        role: &str,
        full_name: Option<&str>,
    ) -> Uuid {
        let user_id = Uuid::now_v7();

        // Using runtime queries instead of macros for test compatibility
        sqlx::query(
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
        )
        .bind(user_id)
        .bind(tenant_id)
        .bind(email)
        .bind(password_hash)
        .bind(role)
        .bind(full_name)
        .execute(&self.pool)
        .await
        .expect("Failed to create test user");

        self.track_user(user_id).await;
        user_id
    }

    /// Create a test session with automatic tracking
    #[allow(dead_code)]
    pub async fn create_session(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        access_token_hash: &str,
        refresh_token_hash: &str,
        access_expires_at: chrono::DateTime<chrono::Utc>,
        refresh_expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Uuid {
        let session_id = Uuid::now_v7();

        // Using runtime queries instead of macros for test compatibility
        sqlx::query(
            r#"
            INSERT INTO sessions (
                session_id, user_id, tenant_id,
                access_token_hash, refresh_token_hash,
                access_token_expires_at, refresh_token_expires_at,
                revoked, created_at, last_used_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, false, NOW(), NOW())
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .bind(tenant_id)
        .bind(access_token_hash)
        .bind(refresh_token_hash)
        .bind(access_expires_at)
        .bind(refresh_expires_at)
        .execute(&self.pool)
        .await
        .expect("Failed to create test session");

        self.track_session(session_id).await;
        session_id
    }

    /// Clean up all tracked resources
    pub async fn cleanup(&self) {
        // Clean sessions first (foreign key to users)
        // Using runtime queries instead of macros for test compatibility
        let session_ids = self.tracked_sessions.lock().await.clone();
        for session_id in session_ids {
            let _ = sqlx::query("DELETE FROM sessions WHERE session_id = $1")
                .bind(session_id)
                .execute(&self.pool)
                .await;
        }
        self.tracked_sessions.lock().await.clear();

        // Clean user profiles (foreign key to users)
        let user_ids = self.tracked_users.lock().await.clone();
        for user_id in &user_ids {
            let _ = sqlx::query("DELETE FROM user_profiles WHERE user_id = $1")
                .bind(user_id)
                .execute(&self.pool)
                .await;
        }

        // Clean users
        for user_id in user_ids {
            let _ = sqlx::query("DELETE FROM users WHERE user_id = $1")
                .bind(user_id)
                .execute(&self.pool)
                .await;
        }
        self.tracked_users.lock().await.clear();

        // Clean tenants last
        let tenant_ids = self.tracked_tenants.lock().await.clone();
        for tenant_id in tenant_ids {
            let _ = sqlx::query("DELETE FROM tenants WHERE tenant_id = $1")
                .bind(tenant_id)
                .execute(&self.pool)
                .await;
        }
        self.tracked_tenants.lock().await.clear();
    }

    /// Clean up all test data using Rust function
    #[allow(dead_code)]
    pub async fn cleanup_all_test_data(&self) {
        // Delete in reverse dependency order to avoid foreign key constraints
        // Using runtime queries instead of macros for test compatibility
        sqlx::query("DELETE FROM sessions")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query("DELETE FROM casbin_rule")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query("DELETE FROM user_profiles")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query("DELETE FROM users")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query("DELETE FROM tenants")
            .execute(&self.pool)
            .await
            .ok();
    }

    /// Verify database is in clean state
    #[allow(dead_code)]
    pub async fn verify_clean(&self) -> bool {
        // Using runtime queries instead of macros for test compatibility
        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM tenants WHERE slug LIKE 'test-%' OR slug LIKE '%-test-%'"#,
        )
        .fetch_one(&self.pool)
        .await
        .expect("Failed to verify clean state");

        count.0 == 0
    }

    /// Get count of resources in database
    #[allow(dead_code)]
    pub async fn get_resource_counts(&self) -> ResourceCounts {
        // Using runtime queries instead of macros for test compatibility
        let tenants: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tenants")
            .fetch_one(&self.pool)
            .await
            .expect("Failed to count tenants");

        let users: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .expect("Failed to count users");

        let sessions: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&self.pool)
            .await
            .expect("Failed to count sessions");

        let profiles: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM user_profiles")
            .fetch_one(&self.pool)
            .await
            .expect("Failed to count profiles");

        ResourceCounts {
            tenants: tenants.0,
            users: users.0,
            sessions: sessions.0,
            profiles: profiles.0,
        }
    }

    /// Get tenant details
    pub async fn get_tenant(&self, tenant_id: Uuid) -> Option<TenantDetails> {
        // Using runtime queries instead of macros for test compatibility
        let row: Option<(Uuid, String, String, String, String)> = sqlx::query_as(
            r#"
            SELECT tenant_id, name, slug, plan, status
            FROM tenants
            WHERE tenant_id = $1
            "#,
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await
        .expect("Failed to fetch tenant details");

        match row {
            Some((tid, name, slug, plan, status)) => {
                let user_count: (i64,) =
                    sqlx::query_as("SELECT COUNT(*) FROM users WHERE tenant_id = $1")
                        .bind(tenant_id)
                        .fetch_one(&self.pool)
                        .await
                        .expect("Failed to count users");

                let session_count: (i64,) =
                    sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE tenant_id = $1")
                        .bind(tenant_id)
                        .fetch_one(&self.pool)
                        .await
                        .expect("Failed to count sessions");

                Some(TenantDetails {
                    tenant_id: tid,
                    name,
                    slug,
                    plan,
                    status,
                    user_count: user_count.0,
                    session_count: session_count.0,
                })
            },
            None => None,
        }
    }

    /// Get user details
    pub async fn get_user(&self, user_id: Uuid) -> Option<UserDetails> {
        // Using runtime queries instead of macros for test compatibility
        let row: Option<(Uuid, Uuid, String, String, String, bool, Option<String>)> =
            sqlx::query_as(
                r#"
            SELECT
                user_id,
                tenant_id,
                email,
                role,
                status,
                email_verified,
                full_name
            FROM users
            WHERE user_id = $1
            "#,
            )
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .expect("Failed to fetch user details");

        row.map(|(user_id, tenant_id, email, role, status, email_verified, full_name)| {
            UserDetails {
                user_id,
                tenant_id,
                email,
                role,
                status,
                email_verified,
                full_name,
            }
        })
    }

    /// Check if email exists in tenant
    #[allow(dead_code)]
    pub async fn email_exists(&self, tenant_id: Uuid, email: &str) -> bool {
        // Using runtime queries instead of macros for test compatibility
        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM users WHERE tenant_id = $1 AND email = $2)",
        )
        .bind(tenant_id)
        .bind(email)
        .fetch_one(&self.pool)
        .await
        .expect("Failed to check email existence");

        result.0
    }

    /// Reset auto_increment sequences (useful for predictable IDs in tests)
    #[allow(dead_code)]
    pub async fn reset_sequences(&self) {
        // Note: UUID v7 doesn't use sequences, but this is here for future use
        // if we add any serial/sequence-based IDs
    }

    /// Add a Casbin grouping policy (g, user_id, role, tenant_id)
    /// This assigns a role to a user for authorization
    /// Note: The policy is tracked and will be automatically cleaned up on drop
    #[allow(dead_code)]
    pub async fn add_casbin_grouping(&self, user_id: Uuid, role: &str, tenant_id: Uuid) {
        // Note: casbin_rule table has v3 as NOT NULL, v4/v5 have defaults
        // For grouping policies (ptype='g'), v3/v4/v5 are typically empty strings
        sqlx::query(
            "INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5) VALUES ('g', $1, $2, $3, '', '', '') ON CONFLICT DO NOTHING",
        )
        .bind(user_id.to_string())
        .bind(role)
        .bind(tenant_id.to_string())
        .execute(&self.pool)
        .await
        .expect("Failed to add Casbin grouping");

        // Track for automatic cleanup
        self.tracked_casbin_rules.lock().await.push((
            user_id.to_string(),
            role.to_string(),
            tenant_id.to_string(),
        ));
    }

    /// Remove a Casbin grouping policy
    #[allow(dead_code)]
    pub async fn remove_casbin_grouping(&self, user_id: Uuid, role: &str, tenant_id: Uuid) {
        sqlx::query(
            "DELETE FROM casbin_rule WHERE ptype = 'g' AND v0 = $1 AND v1 = $2 AND v2 = $3",
        )
        .bind(user_id.to_string())
        .bind(role)
        .bind(tenant_id.to_string())
        .execute(&self.pool)
        .await
        .expect("Failed to remove Casbin grouping");
    }
}

impl Drop for TestDatabaseConfig {
    fn drop(&mut self) {
        if self.auto_cleanup {
            // Spawn cleanup task in background
            let pool = self.pool.clone();
            let tenants = self.tracked_tenants.clone();
            let users = self.tracked_users.clone();
            let sessions = self.tracked_sessions.clone();
            let casbin_rules = self.tracked_casbin_rules.clone();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    // Clean Casbin rules first (no FK dependencies)
                    let rules = casbin_rules.lock().await.clone();
                    for (user_id, role, tenant_id) in rules {
                        let _ = sqlx::query(
                            "DELETE FROM casbin_rule WHERE ptype = 'g' AND v0 = $1 AND v1 = $2 AND v2 = $3",
                        )
                        .bind(&user_id)
                        .bind(&role)
                        .bind(&tenant_id)
                        .execute(&pool)
                        .await;
                    }

                    // Clean sessions - using runtime queries for test compatibility
                    let session_ids = sessions.lock().await.clone();
                    for session_id in session_ids {
                        let _ = sqlx::query("DELETE FROM sessions WHERE session_id = $1")
                            .bind(session_id)
                            .execute(&pool)
                            .await;
                    }

                    // Clean user profiles
                    let user_ids = users.lock().await.clone();
                    for user_id in &user_ids {
                        let _ = sqlx::query("DELETE FROM user_profiles WHERE user_id = $1")
                            .bind(user_id)
                            .execute(&pool)
                            .await;
                    }

                    // Clean users
                    for user_id in user_ids {
                        let _ = sqlx::query("DELETE FROM users WHERE user_id = $1")
                            .bind(user_id)
                            .execute(&pool)
                            .await;
                    }

                    // Clean tenants
                    let tenant_ids = tenants.lock().await.clone();
                    for tenant_id in tenant_ids {
                        let _ = sqlx::query("DELETE FROM tenants WHERE tenant_id = $1")
                            .bind(tenant_id)
                            .execute(&pool)
                            .await;
                    }
                });
            });
        }
    }
}

/// Resource counts in database
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct ResourceCounts {
    pub tenants: i64,
    pub users: i64,
    pub sessions: i64,
    pub profiles: i64,
}

/// Tenant details for verification
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TenantDetails {
    pub tenant_id: Uuid,
    pub name: String,
    pub slug: String,
    pub plan: String,
    pub status: String,
    pub user_count: i64,
    pub session_count: i64,
}

/// User details for verification
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UserDetails {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub role: String,
    pub status: String,
    pub email_verified: bool,
    pub full_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_database_config_creation() {
        let config = TestDatabaseConfig::new().await;
        assert!(config.pool().acquire().await.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_tenant_creation_and_cleanup() {
        let config = TestDatabaseConfig::new().await;

        let tenant_id = config.create_tenant("Test Tenant", None).await;

        // Verify tenant exists
        let tenant = config.get_tenant(tenant_id).await;
        assert!(tenant.is_some());
        assert_eq!(tenant.unwrap().name, "Test Tenant");

        // Cleanup
        config.cleanup().await;

        // Verify tenant removed
        let tenant = config.get_tenant(tenant_id).await;
        assert!(tenant.is_none());
    }

    #[tokio::test]
    #[ignore]
    async fn test_user_creation() {
        let config = TestDatabaseConfig::new().await;

        let tenant_id = config.create_tenant("User Test Tenant", None).await;
        let user_id = config
            .create_user(
                tenant_id,
                "test@example.com",
                "$argon2id$v=19$m=19456,t=2,p=1$test$test",
                "user",
                Some("Test User"),
            )
            .await;

        // Verify user exists
        let user = config.get_user(user_id).await;
        assert!(user.is_some());

        let user = user.unwrap();
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.role, "user");

        // Cleanup
        config.cleanup().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_transaction_rollback() {
        let config = TestDatabaseConfig::new().await;

        let tenant_id = config.create_tenant("Transaction Test", None).await;

        // Begin transaction
        let mut tx = config.begin_transaction().await.unwrap();

        // Create user in transaction
        let user_id = Uuid::now_v7();
        // Using runtime query instead of macro for test compatibility
        sqlx::query(
            r#"
            INSERT INTO users (
                user_id, tenant_id, email, password_hash, role, status,
                email_verified, email_verified_at, full_name, created_at, updated_at
            )
            VALUES (
                $1, $2, 'rollback@test.com', $3, 'user', 'active',
                true, NOW(), 'Rollback User', NOW(), NOW()
            )
            "#,
        )
        .bind(user_id)
        .bind(tenant_id)
        .bind("$argon2id$v=19$m=19456,t=2,p=1$test$test")
        .execute(&mut *tx)
        .await
        .unwrap();

        // Rollback
        tx.rollback().await.unwrap();

        // Verify user was not created
        let user = config.get_user(user_id).await;
        assert!(user.is_none());

        // Cleanup
        config.cleanup().await;
    }
}
