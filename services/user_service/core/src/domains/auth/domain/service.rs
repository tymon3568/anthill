use async_trait::async_trait;
use uuid::Uuid;
use shared_error::AppError;
use crate::domains::auth::dto::auth_dto::{
    RegisterReq, LoginReq, RefreshReq, AuthResp, UserInfo, UserListResp,
};

/// Auth service trait
/// 
/// Defines the business logic interface for authentication operations.
#[async_trait]
pub trait AuthService: Send + Sync {
    /// Register a new user
    async fn register(&self, req: RegisterReq) -> Result<AuthResp, AppError>;
    
    /// Login user
    async fn login(&self, req: LoginReq) -> Result<AuthResp, AppError>;
    
    /// Refresh access token
    async fn refresh_token(&self, req: RefreshReq) -> Result<AuthResp, AppError>;
    
    /// Logout user by revoking refresh token session
    async fn logout(&self, refresh_token: &str) -> Result<(), AppError>;
    
    /// List users (paginated, tenant-scoped)
    async fn list_users(&self, tenant_id: Uuid, page: i32, page_size: i32) -> Result<UserListResp, AppError>;
    
    /// Get user info by ID
    async fn get_user(&self, user_id: Uuid, tenant_id: Uuid) -> Result<UserInfo, AppError>;
}
