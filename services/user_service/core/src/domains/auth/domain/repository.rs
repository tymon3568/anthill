use async_trait::async_trait;
use uuid::Uuid;
use super::model::{User, Tenant};
use shared_error::AppError;

/// User repository trait
/// 
/// Defines the interface for user data access operations.
/// Implementations must handle tenant isolation.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find user by email within a tenant
    async fn find_by_email(&self, email: &str, tenant_id: Uuid) -> Result<Option<User>, AppError>;
    
    /// Find user by ID within a tenant
    async fn find_by_id(&self, id: Uuid, tenant_id: Uuid) -> Result<Option<User>, AppError>;
    
    /// Create a new user
    async fn create(&self, user: &User) -> Result<User, AppError>;
    
    /// Update a user
    async fn update(&self, user: &User) -> Result<User, AppError>;
    
    /// List users with pagination (within a tenant)
    async fn list(&self, tenant_id: Uuid, page: i32, page_size: i32) -> Result<(Vec<User>, i64), AppError>;
    
    /// Check if email exists within a tenant
    async fn email_exists(&self, email: &str, tenant_id: Uuid) -> Result<bool, AppError>;
}

/// Tenant repository trait
#[async_trait]
pub trait TenantRepository: Send + Sync {
    /// Find tenant by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Tenant>, AppError>;
    
    /// Create a new tenant
    async fn create(&self, tenant: &Tenant) -> Result<Tenant, AppError>;
    
    /// Find tenant by name
    async fn find_by_name(&self, name: &str) -> Result<Option<Tenant>, AppError>;
    
    /// Find tenant by slug
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Tenant>, AppError>;
}
