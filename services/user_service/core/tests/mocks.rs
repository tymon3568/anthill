//! Mock implementations of repositories for unit testing
//!
//! These mocks allow testing business logic without database dependencies

use async_trait::async_trait;
use mockall::mock;
use shared_error::AppError;
use user_service_core::domains::auth::domain::model::{Session, Tenant, User};
use user_service_core::domains::auth::domain::repository::{
    SessionRepository, TenantRepository, UserRepository,
};
use uuid::Uuid;

// Mock UserRepository
mock! {
    pub UserRepo {}

    #[async_trait]
    impl UserRepository for UserRepo {
        async fn find_by_id(&self, user_id: Uuid, tenant_id: Uuid) -> Result<Option<User>, AppError>;
        async fn find_by_email(&self, email: &str, tenant_id: Uuid) -> Result<Option<User>, AppError>;
        async fn create(&self, user: &User) -> Result<User, AppError>;
        async fn update(&self, user: &User) -> Result<User, AppError>;
        async fn delete(&self, user_id: Uuid, tenant_id: Uuid) -> Result<(), AppError>;
        async fn list_by_tenant(&self, tenant_id: Uuid, limit: i64, offset: i64) -> Result<Vec<User>, AppError>;
        async fn count_by_tenant(&self, tenant_id: Uuid) -> Result<i64, AppError>;
        async fn update_failed_login_attempts(&self, user_id: Uuid, tenant_id: Uuid, attempts: i32) -> Result<(), AppError>;
        async fn lock_user(&self, user_id: Uuid, tenant_id: Uuid, locked_until: chrono::DateTime<chrono::Utc>) -> Result<(), AppError>;
        async fn update_last_login(&self, user_id: Uuid, tenant_id: Uuid) -> Result<(), AppError>;
        async fn verify_email(&self, user_id: Uuid, tenant_id: Uuid) -> Result<(), AppError>;
        async fn update_password(&self, user_id: Uuid, tenant_id: Uuid, password_hash: &str) -> Result<(), AppError>;
    }
}

// Mock TenantRepository
mock! {
    pub TenantRepo {}

    #[async_trait]
    impl TenantRepository for TenantRepo {
        async fn find_by_id(&self, tenant_id: Uuid) -> Result<Option<Tenant>, AppError>;
        async fn find_by_slug(&self, slug: &str) -> Result<Option<Tenant>, AppError>;
        async fn create(&self, tenant: &Tenant) -> Result<Tenant, AppError>;
        async fn update(&self, tenant: &Tenant) -> Result<Tenant, AppError>;
        async fn delete(&self, tenant_id: Uuid) -> Result<(), AppError>;
        async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Tenant>, AppError>;
        async fn count(&self) -> Result<i64, AppError>;
    }
}

// Mock SessionRepository
mock! {
    pub SessionRepo {}

    #[async_trait]
    impl SessionRepository for SessionRepo {
        async fn create(&self, session: &Session) -> Result<Session, AppError>;
        async fn find_by_refresh_token(&self, refresh_token: &str) -> Result<Option<Session>, AppError>;
        async fn delete_by_refresh_token(&self, refresh_token: &str) -> Result<(), AppError>;
        async fn delete_all_for_user(&self, user_id: Uuid, tenant_id: Uuid) -> Result<(), AppError>;
        async fn cleanup_expired(&self) -> Result<u64, AppError>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::UserBuilder;

    #[tokio::test]
    async fn test_mock_user_repository() {
        let mut mock_repo = MockUserRepo::new();
        let tenant_id = Uuid::now_v7();
        let user = UserBuilder::new()
            .with_tenant_id(tenant_id)
            .with_email("test@example.com")
            .build();

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
    async fn test_mock_tenant_repository() {
        let mut mock_repo = MockTenantRepo::new();
        let tenant_id = Uuid::now_v7();

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
}
