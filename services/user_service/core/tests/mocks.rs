//! Mock implementations of repository traits for testing
//!
//! Uses mockall to create mock implementations that can be configured
//! to return specific values or verify interactions in tests.

use async_trait::async_trait;
use mockall::mock;
use shared_error::AppError;
use user_service_core::domains::auth::domain::model::{Session, Tenant, User};
use user_service_core::domains::auth::domain::repository::{
    SessionRepository, TenantRepository, UserRepository,
};
use uuid::Uuid;

/// Mock implementation of UserRepository
///
/// Matches the exact trait signature in core/src/domains/auth/domain/repository.rs
mock! {
    pub UserRepo {}

    #[async_trait]
    impl UserRepository for UserRepo {
        async fn find_by_email(&self, email: &str, tenant_id: Uuid) -> Result<Option<User>, AppError>;
        async fn find_by_id(&self, id: Uuid, tenant_id: Uuid) -> Result<Option<User>, AppError>;
        async fn create(&self, user: &User) -> Result<User, AppError>;
        async fn update(&self, user: &User) -> Result<User, AppError>;
        async fn list(
            &self,
            tenant_id: Uuid,
            page: i32,
            page_size: i32,
            role: Option<String>,
            status: Option<String>,
        ) -> Result<(Vec<User>, i64), AppError>;
        async fn email_exists(&self, email: &str, tenant_id: Uuid) -> Result<bool, AppError>;
    }
}

/// Mock implementation of TenantRepository
///
/// Matches the exact trait signature in core/src/domains/auth/domain/repository.rs
mock! {
    pub TenantRepo {}

    #[async_trait]
    impl TenantRepository for TenantRepo {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Tenant>, AppError>;
        async fn create(&self, tenant: &Tenant) -> Result<Tenant, AppError>;
        async fn find_by_name(&self, name: &str) -> Result<Option<Tenant>, AppError>;
        async fn find_by_slug(&self, slug: &str) -> Result<Option<Tenant>, AppError>;
    }
}

/// Mock implementation of SessionRepository
///
/// Matches the exact trait signature in core/src/domains/auth/domain/repository.rs
mock! {
    pub SessionRepo {}

    #[async_trait]
    impl SessionRepository for SessionRepo {
        async fn create(&self, session: &Session) -> Result<Session, AppError>;
        async fn find_by_refresh_token(&self, token_hash: &str) -> Result<Option<Session>, AppError>;
        async fn revoke(&self, session_id: Uuid, reason: &str) -> Result<(), AppError>;
        async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<u64, AppError>;
        async fn update_last_used(&self, session_id: Uuid) -> Result<(), AppError>;
        async fn delete_expired(&self) -> Result<u64, AppError>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_user_repository_find_by_email() {
        use chrono::Utc;

        let mut mock_repo = MockUserRepo::new();
        let tenant_id = Uuid::new_v4();
        let now = Utc::now();

        let user = User {
            user_id: Uuid::new_v4(),
            tenant_id,
            email: "test@example.com".to_string(),
            password_hash: Some("hashed".to_string()),  // Now Option<String>
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
            kanidm_user_id: None,
            kanidm_synced_at: None,
            auth_method: "password".to_string(),  // NEW
            migration_invited_at: None,  // NEW
            migration_completed_at: None,  // NEW
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        // Setup expectation
        let user_clone = user.clone();
        mock_repo
            .expect_find_by_email()
            .with(
                mockall::predicate::eq("test@example.com"),
                mockall::predicate::eq(tenant_id),
            )
            .times(1)
            .returning(move |_, _| Ok(Some(user_clone.clone())));

        // Test
        let result = mock_repo
            .find_by_email("test@example.com", tenant_id)
            .await
            .unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().email, "test@example.com");
    }

    #[tokio::test]
    async fn test_mock_user_repository_email_exists() {
        let mut mock_repo = MockUserRepo::new();
        let tenant_id = Uuid::new_v4();

        // Setup expectation
        mock_repo
            .expect_email_exists()
            .with(
                mockall::predicate::eq("existing@example.com"),
                mockall::predicate::eq(tenant_id),
            )
            .times(1)
            .returning(|_, _| Ok(true));

        // Test
        let exists = mock_repo
            .email_exists("existing@example.com", tenant_id)
            .await
            .unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_mock_user_repository_list() {
        let mut mock_repo = MockUserRepo::new();
        let tenant_id = Uuid::new_v4();

        // Setup expectation - return empty list
        mock_repo
            .expect_list()
            .with(
                mockall::predicate::eq(tenant_id),
                mockall::predicate::eq(1),
                mockall::predicate::eq(10),
                mockall::predicate::eq(None::<String>),
                mockall::predicate::eq(None::<String>),
            )
            .times(1)
            .returning(|_, _, _, _, _| Ok((vec![], 0)));

        // Test
        let (users, count) = mock_repo.list(tenant_id, 1, 10, None, None).await.unwrap();
        assert_eq!(users.len(), 0);
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_mock_tenant_repository() {
        let mut mock_repo = MockTenantRepo::new();
        let tenant_id = Uuid::new_v4();

        // Setup expectation
        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(tenant_id))
            .times(1)
            .returning(|_| Ok(None));

        // Test
        let result = mock_repo.find_by_id(tenant_id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_mock_session_repository() {
        let mut mock_repo = MockSessionRepo::new();
        let session_id = Uuid::new_v4();

        // Setup expectation
        mock_repo
            .expect_revoke()
            .with(
                mockall::predicate::eq(session_id),
                mockall::predicate::eq("user_logout"),
            )
            .times(1)
            .returning(|_, _| Ok(()));

        // Test
        let result = mock_repo.revoke(session_id, "user_logout").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_session_repository_delete_expired() {
        let mut mock_repo = MockSessionRepo::new();

        // Setup expectation - deleted 5 expired sessions
        mock_repo
            .expect_delete_expired()
            .times(1)
            .returning(|| Ok(5));

        // Test
        let deleted = mock_repo.delete_expired().await.unwrap();
        assert_eq!(deleted, 5);
    }
}
