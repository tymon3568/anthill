use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use user_service_core::domains::auth::domain::{
    model::{User, Tenant},
    repository::{UserRepository, TenantRepository},
};
use shared_error::AppError;

/// PostgreSQL implementation of UserRepository
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_by_email(&self, email: &str, tenant_id: Uuid) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1 AND tenant_id = $2 AND status = 'active' AND deleted_at IS NULL"
        )
        .bind(email)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    async fn find_by_id(&self, id: Uuid, tenant_id: Uuid) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE user_id = $1 AND tenant_id = $2 AND status = 'active' AND deleted_at IS NULL"
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    async fn create(&self, user: &User) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (
                user_id, tenant_id, email, password_hash, full_name, role, status,
                email_verified, failed_login_attempts, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#
        )
        .bind(user.user_id)
        .bind(user.tenant_id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.full_name)
        .bind(&user.role)
        .bind(&user.status)
        .bind(user.email_verified)
        .bind(user.failed_login_attempts)
        .bind(user.created_at)
        .bind(user.updated_at)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    async fn update(&self, user: &User) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET email = $2, password_hash = $3, full_name = $4, role = $5, status = $6, 
                updated_at = NOW(), last_login_at = $7
            WHERE user_id = $1 AND tenant_id = $8 AND deleted_at IS NULL
            RETURNING *
            "#
        )
        .bind(user.user_id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.full_name)
        .bind(&user.role)
        .bind(&user.status)
        .bind(user.last_login_at)
        .bind(user.tenant_id)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    async fn list(&self, tenant_id: Uuid, page: i32, page_size: i32) -> Result<(Vec<User>, i64), AppError> {
        let offset = (page - 1) * page_size;
        
        // Get total count
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users WHERE tenant_id = $1 AND status = 'active' AND deleted_at IS NULL"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;
        
        // Get users
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE tenant_id = $1 AND status = 'active' AND deleted_at IS NULL 
             ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(tenant_id)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;
        
        Ok((users, total.0))
    }
    
    async fn email_exists(&self, email: &str, tenant_id: Uuid) -> Result<bool, AppError> {
        let exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND tenant_id = $2)"
        )
        .bind(email)
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(exists.0)
    }
}

/// PostgreSQL implementation of TenantRepository
pub struct PgTenantRepository {
    pool: PgPool,
}

impl PgTenantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TenantRepository for PgTenantRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Tenant>, AppError> {
        let tenant = sqlx::query_as::<_, Tenant>(
            "SELECT * FROM tenants WHERE tenant_id = $1 AND status = 'active' AND deleted_at IS NULL"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(tenant)
    }
    
    async fn create(&self, tenant: &Tenant) -> Result<Tenant, AppError> {
        let tenant = sqlx::query_as::<_, Tenant>(
            r#"
            INSERT INTO tenants (tenant_id, name, slug, plan, status, settings, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#
        )
        .bind(tenant.tenant_id)
        .bind(&tenant.name)
        .bind(&tenant.slug)
        .bind(&tenant.plan)
        .bind(&tenant.status)
        .bind(&tenant.settings)
        .bind(tenant.created_at)
        .bind(tenant.updated_at)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(tenant)
    }
    
    async fn find_by_name(&self, name: &str) -> Result<Option<Tenant>, AppError> {
        let tenant = sqlx::query_as::<_, Tenant>(
            "SELECT * FROM tenants WHERE name = $1 AND status = 'active' AND deleted_at IS NULL"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(tenant)
    }
    
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Tenant>, AppError> {
        let tenant = sqlx::query_as::<_, Tenant>(
            "SELECT * FROM tenants WHERE slug = $1 AND status = 'active' AND deleted_at IS NULL"
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(tenant)
    }
}
