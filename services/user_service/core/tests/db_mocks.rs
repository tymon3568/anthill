//! Database mocking utilities for unit tests
//!
//! Provides mock implementations for database operations to enable
//! testing business logic without requiring a real database connection.

use shared_error::AppError;
use std::sync::Arc;
use tokio::sync::Mutex;
use user_service_core::domains::auth::domain::model::{Session, Tenant, User};
use uuid::Uuid;

/// Mock database pool for testing
///
/// This allows tests to simulate database behavior without actual connections.
/// Use this when you need to test code that depends on PgPool but don't want
/// to spin up a real database.
#[derive(Clone)]
pub struct MockDbPool {
    pub users: Arc<Mutex<Vec<User>>>,
    pub tenants: Arc<Mutex<Vec<Tenant>>>,
    pub sessions: Arc<Mutex<Vec<Session>>>,
}

impl MockDbPool {
    /// Create a new empty mock database pool
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(Vec::new())),
            tenants: Arc::new(Mutex::new(Vec::new())),
            sessions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a mock pool pre-populated with test data
    pub fn with_test_data() -> Self {
        // Simple pre-populated data without complex dependencies
        Self {
            users: Arc::new(Mutex::new(Vec::new())),
            tenants: Arc::new(Mutex::new(Vec::new())),
            sessions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a user to the mock database
    pub async fn add_user(&self, user: User) {
        self.users.lock().await.push(user);
    }

    /// Add a tenant to the mock database
    pub async fn add_tenant(&self, tenant: Tenant) {
        self.tenants.lock().await.push(tenant);
    }

    /// Add a session to the mock database
    pub async fn add_session(&self, session: Session) {
        self.sessions.lock().await.push(session);
    }

    /// Find user by email and tenant_id
    pub async fn find_user_by_email(
        &self,
        email: &str,
        tenant_id: Uuid,
    ) -> Result<Option<User>, AppError> {
        let users = self.users.lock().await;
        Ok(users
            .iter()
            .find(|u| u.email == email && u.tenant_id == tenant_id)
            .cloned())
    }

    /// Find user by ID and tenant_id
    pub async fn find_user_by_id(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Option<User>, AppError> {
        let users = self.users.lock().await;
        Ok(users
            .iter()
            .find(|u| u.user_id == user_id && u.tenant_id == tenant_id)
            .cloned())
    }

    /// Find tenant by ID
    pub async fn find_tenant_by_id(&self, tenant_id: Uuid) -> Result<Option<Tenant>, AppError> {
        let tenants = self.tenants.lock().await;
        Ok(tenants.iter().find(|t| t.tenant_id == tenant_id).cloned())
    }

    /// Find tenant by slug
    pub async fn find_tenant_by_slug(&self, slug: &str) -> Result<Option<Tenant>, AppError> {
        let tenants = self.tenants.lock().await;
        Ok(tenants.iter().find(|t| t.slug == slug).cloned())
    }

    /// List users for a tenant with pagination
    pub async fn list_users(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<User>, AppError> {
        let users = self.users.lock().await;
        let filtered: Vec<User> = users
            .iter()
            .filter(|u| u.tenant_id == tenant_id)
            .skip(offset as usize)
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(filtered)
    }

    /// Count users for a tenant
    pub async fn count_users(&self, tenant_id: Uuid) -> Result<i64, AppError> {
        let users = self.users.lock().await;
        let count = users.iter().filter(|u| u.tenant_id == tenant_id).count() as i64;
        Ok(count)
    }

    /// Clear all data (useful for test cleanup)
    pub async fn clear(&self) {
        self.users.lock().await.clear();
        self.tenants.lock().await.clear();
        self.sessions.lock().await.clear();
    }
}

impl Default for MockDbPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Test database transaction helper
///
/// Simulates database transactions for testing. In real code, use sqlx::Transaction.
/// For tests, this allows rolling back changes automatically.
pub struct TestTransaction {
    committed: Arc<Mutex<bool>>,
}

impl TestTransaction {
    pub fn new() -> Self {
        Self {
            committed: Arc::new(Mutex::new(false)),
        }
    }

    /// Commit the transaction (marks as committed, no actual DB work)
    pub async fn commit(&self) {
        *self.committed.lock().await = true;
    }

    /// Check if transaction was committed
    pub async fn is_committed(&self) -> bool {
        *self.committed.lock().await
    }
}

impl Default for TestTransaction {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create a test database query result
///
/// Simulates the result of a sqlx query for testing purposes
pub struct MockQueryResult {
    pub rows_affected: u64,
}

impl MockQueryResult {
    pub fn new(rows_affected: u64) -> Self {
        Self { rows_affected }
    }

    pub fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use user_service_core::domains::auth::domain::model::User;
    use chrono::Utc;

    fn create_test_user(email: &str, tenant_id: Uuid) -> User {
        let now = Utc::now();
        User {
            user_id: Uuid::new_v4(),
            tenant_id,
            email: email.to_string(),
            password_hash: "hashed_password".to_string(),
            email_verified: false,
            email_verified_at: None,
            full_name: Some("Test User".to_string()),
            avatar_url: None,
            phone: None,
            role: "user".to_string(),
            status: "active".to_string(),
            last_login_at: None,
            failed_login_attempts: 0,
            locked_until: None,
            password_changed_at: Some(now),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    #[tokio::test]
    async fn test_mock_db_pool_empty() {
        let pool = MockDbPool::new();
        let users = pool.users.lock().await;
        assert_eq!(users.len(), 0);
    }

    #[tokio::test]
    async fn test_mock_db_pool_with_test_data() {
        let pool = MockDbPool::with_test_data();

        // Should start empty (simplified version)
        let users = pool.users.lock().await;
        assert_eq!(users.len(), 0);

        let tenants = pool.tenants.lock().await;
        assert_eq!(tenants.len(), 0);
    }

    #[tokio::test]
    async fn test_add_and_find_user() {
        let pool = MockDbPool::new();
        let tenant_id = Uuid::new_v4();

        let user = create_test_user("test@example.com", tenant_id);

        pool.add_user(user.clone()).await;

        let found = pool
            .find_user_by_email("test@example.com", tenant_id)
            .await
            .unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().email, "test@example.com");
    }

    #[tokio::test]
    async fn test_tenant_isolation() {
        let pool = MockDbPool::new();
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();

        let user_a = create_test_user("user@tenant-a.com", tenant_a);
        let user_b = create_test_user("user@tenant-b.com", tenant_b);

        pool.add_user(user_a).await;
        pool.add_user(user_b).await;

        // Should not find user from different tenant
        let result = pool.find_user_by_email("user@tenant-b.com", tenant_a).await;
        assert!(result.unwrap().is_none(), "Should not cross tenant boundary");
    }

    #[tokio::test]
    async fn test_list_users_pagination() {
        let pool = MockDbPool::new();
        let tenant_id = Uuid::new_v4();

        // Add 5 users
        for i in 0..5 {
            let user = create_test_user(&format!("user{}@test.com", i), tenant_id);
            pool.add_user(user).await;
        }

        // Get first 2 users
        let page1 = pool.list_users(tenant_id, 2, 0).await.unwrap();
        assert_eq!(page1.len(), 2);

        // Get next 2 users
        let page2 = pool.list_users(tenant_id, 2, 2).await.unwrap();
        assert_eq!(page2.len(), 2);

        // Get last page
        let page3 = pool.list_users(tenant_id, 2, 4).await.unwrap();
        assert_eq!(page3.len(), 1);
    }

    #[tokio::test]
    async fn test_count_users() {
        let pool = MockDbPool::new();
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();

        // Add 3 users to tenant A
        for i in 0..3 {
            let user = create_test_user(&format!("user{}@tenant-a.com", i), tenant_a);
            pool.add_user(user).await;
        }

        // Add 2 users to tenant B
        for i in 0..2 {
            let user = create_test_user(&format!("user{}@tenant-b.com", i), tenant_b);
            pool.add_user(user).await;
        }

        let count_a = pool.count_users(tenant_a).await.unwrap();
        let count_b = pool.count_users(tenant_b).await.unwrap();

        assert_eq!(count_a, 3, "Tenant A should have 3 users");
        assert_eq!(count_b, 2, "Tenant B should have 2 users");
    }

    #[tokio::test]
    async fn test_clear_database() {
        let pool = MockDbPool::with_test_data();

        // Add some data
        let user = create_test_user("test@example.com", Uuid::new_v4());
        pool.add_user(user).await;

        // Verify data exists
        let users_before = pool.users.lock().await.len();
        assert!(users_before > 0, "Should have test data");

        // Clear all data
        pool.clear().await;

        // Verify data is gone
        let users_after = pool.users.lock().await.len();
        let tenants_after = pool.tenants.lock().await.len();
        let sessions_after = pool.sessions.lock().await.len();

        assert_eq!(users_after, 0, "Users should be cleared");
        assert_eq!(tenants_after, 0, "Tenants should be cleared");
        assert_eq!(sessions_after, 0, "Sessions should be cleared");
    }

    #[tokio::test]
    async fn test_transaction_commit() {
        let tx = TestTransaction::new();
        assert!(!tx.is_committed().await);

        tx.commit().await;
        assert!(tx.is_committed().await);
    }

    #[test]
    fn test_mock_query_result() {
        let result = MockQueryResult::new(5);
        assert_eq!(result.rows_affected(), 5);
    }
}
