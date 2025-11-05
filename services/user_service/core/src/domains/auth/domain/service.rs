use crate::domains::auth::dto::auth_dto::{
    AuthResp, LoginReq, RefreshReq, RegisterReq, UserInfo, UserListResp,
};
use async_trait::async_trait;
use shared_error::AppError;
use uuid::Uuid;

/// Auth service trait
///
/// Defines the business logic interface for authentication operations.
#[async_trait]
pub trait AuthService: Send + Sync {
    /// Register a new user
    async fn register(
        &self,
        req: RegisterReq,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuthResp, AppError>;

    /// Login user
    async fn login(
        &self,
        req: LoginReq,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuthResp, AppError>;

    /// Refresh access token
    async fn refresh_token(
        &self,
        req: RefreshReq,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuthResp, AppError>;

    /// Logout user by revoking refresh token session
    async fn logout(&self, refresh_token: &str) -> Result<(), AppError>;

    /// List users (paginated, tenant-scoped, with optional filtering)
    async fn list_users(
        &self,
        tenant_id: Uuid,
        page: i32,
        page_size: i32,
        role: Option<String>,
        status: Option<String>,
    ) -> Result<UserListResp, AppError>;

    /// Get user info by ID
    async fn get_user(&self, user_id: Uuid, tenant_id: Uuid) -> Result<UserInfo, AppError>;

    /// Cleanup stale Kanidm sessions (admin operation)
    /// Revokes sessions for users that no longer exist in Kanidm
    async fn cleanup_stale_kanidm_sessions(&self) -> Result<u64, AppError>;
}
