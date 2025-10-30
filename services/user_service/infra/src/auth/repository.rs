use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use user_service_core::domains::auth::domain::{
    model::{User, Tenant},
    repository::{UserRepository, TenantRepository},
};
use shared_error::AppError;

/// PostgreSQL implementation of UserRepository
#[derive(Clone)]
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
                user_id, tenant_id, email, password_hash, full_name, avatar_url, phone,
                role, status, email_verified, email_verified_at, last_login_at,
                failed_login_attempts, locked_until, password_changed_at,
                created_at, updated_at, deleted_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING *
            "#
        )
        .bind(user.user_id)
        .bind(user.tenant_id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.full_name)
        .bind(&user.avatar_url)
        .bind(&user.phone)
        .bind(&user.role)
        .bind(&user.status)
        .bind(user.email_verified)
        .bind(user.email_verified_at)
        .bind(user.last_login_at)
        .bind(user.failed_login_attempts)
        .bind(user.locked_until)
        .bind(user.password_changed_at)
        .bind(user.created_at)
        .bind(user.updated_at)
        .bind(user.deleted_at)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    async fn update(&self, user: &User) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET email = $2, 
                password_hash = $3, 
                full_name = $4, 
                avatar_url = $5,
                phone = $6,
                role = $7, 
                status = $8,
                email_verified = $9,
                email_verified_at = $10,
                last_login_at = $11,
                failed_login_attempts = $12,
                locked_until = $13,
                password_changed_at = $14,
                updated_at = NOW()
            WHERE user_id = $1 AND tenant_id = $15 AND deleted_at IS NULL
            RETURNING *
            "#
        )
        .bind(user.user_id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.full_name)
        .bind(&user.avatar_url)
        .bind(&user.phone)
        .bind(&user.role)
        .bind(&user.status)
        .bind(user.email_verified)
        .bind(user.email_verified_at)
        .bind(user.last_login_at)
        .bind(user.failed_login_attempts)
        .bind(user.locked_until)
        .bind(user.password_changed_at)
        .bind(user.tenant_id)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    async fn list(&self, tenant_id: Uuid, page: i32, page_size: i32, role: Option<String>, status: Option<String>) -> Result<(Vec<User>, i64), AppError> {
        // Clamp page to minimum of 1 and page_size to safe bounds (1-100)
        let page = page.max(1);
        let page_size = page_size.max(1).min(100);

        // Use saturating arithmetic and convert to u64 for database offset
        let offset: u64 = ((page as u64).saturating_sub(1)).saturating_mul(page_size as u64);

        // Build query dynamically using QueryBuilder to prevent SQL injection
        let mut query_builder = sqlx::QueryBuilder::new("SELECT * FROM users WHERE tenant_id = ");
        query_builder.push_bind(tenant_id);
        query_builder.push(" AND deleted_at IS NULL");

        let mut count_builder = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM users WHERE tenant_id = ");
        count_builder.push_bind(tenant_id);
        count_builder.push(" AND deleted_at IS NULL");

        // Add role filter with proper parameter binding
        if let Some(role_filter) = &role {
            query_builder.push(" AND role = ");
            query_builder.push_bind(role_filter);
            count_builder.push(" AND role = ");
            count_builder.push_bind(role_filter);
        }

        // Add status filter with proper parameter binding
        if let Some(status_filter) = &status {
            query_builder.push(" AND status = ");
            query_builder.push_bind(status_filter);
            count_builder.push(" AND status = ");
            count_builder.push_bind(status_filter);
        } else {
            // Default to active status
            query_builder.push(" AND status = ");
            query_builder.push_bind("active");
            count_builder.push(" AND status = ");
            count_builder.push_bind("active");
        }

        // Add ordering and pagination with proper parameter binding
        query_builder.push(" ORDER BY created_at DESC LIMIT ");
        query_builder.push_bind(page_size as i64);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset as i64);

        // Execute count query
        let total: (i64,) = count_builder
            .build_query_as::<(i64,)>()
            .fetch_one(&self.pool)
            .await?;

        // Execute data query
        let users = query_builder
            .build_query_as::<User>()
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
            INSERT INTO tenants (
                tenant_id, name, slug, plan, plan_expires_at, 
                status, settings, created_at, updated_at, deleted_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#
        )
        .bind(tenant.tenant_id)
        .bind(&tenant.name)
        .bind(&tenant.slug)
        .bind(&tenant.plan)
        .bind(tenant.plan_expires_at)
        .bind(&tenant.status)
        .bind(&tenant.settings)
        .bind(tenant.created_at)
        .bind(tenant.updated_at)
        .bind(tenant.deleted_at)
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
