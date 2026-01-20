use async_trait::async_trait;
use shared_error::AppError;
use sqlx::PgPool;
use user_service_core::domains::auth::domain::{
    model::{Tenant, User},
    repository::{TenantRepository, UserRepository},
};
use uuid::Uuid;

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
    async fn find_by_email(&self, tenant_id: Uuid, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE tenant_id = $1 AND email = $2 AND status = 'active' AND deleted_at IS NULL"
        )
        .bind(tenant_id)
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_by_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE tenant_id = $1 AND user_id = $2 AND status = 'active' AND deleted_at IS NULL"
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_by_id_any_status(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE tenant_id = $1 AND user_id = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_by_ids(&self, tenant_id: Uuid, user_ids: &[Uuid]) -> Result<Vec<User>, AppError> {
        if user_ids.is_empty() {
            return Ok(vec![]);
        }

        let _placeholders: Vec<String> = (1..=user_ids.len())
            .map(|i| format!("${}", i + 1))
            .collect();
        let query_str = "SELECT * FROM users WHERE tenant_id = $1 AND user_id = ANY($2) AND status = 'active' AND deleted_at IS NULL".to_string();

        let users = sqlx::query_as::<_, User>(&query_str)
            .bind(tenant_id)
            .bind(user_ids)
            .fetch_all(&self.pool)
            .await?;

        Ok(users)
    }

    async fn create(&self, user: &User) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (
                user_id, tenant_id, email, password_hash, full_name, avatar_url, phone,
                role, status, email_verified, email_verified_at, last_login_at,
                failed_login_attempts, locked_until, password_changed_at,
                auth_method, migration_invited_at, migration_completed_at,
                created_at, updated_at, deleted_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            RETURNING *
            "#,
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
        .bind(&user.auth_method)
        .bind(user.migration_invited_at)
        .bind(user.migration_completed_at)
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
                auth_method = $15,
                migration_invited_at = $16,
                migration_completed_at = $17,
                deleted_at = $19,
                updated_at = NOW()
            WHERE user_id = $1 AND tenant_id = $18 AND deleted_at IS NULL
            RETURNING *
            "#,
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
        .bind(&user.auth_method)
        .bind(user.migration_invited_at)
        .bind(user.migration_completed_at)
        .bind(user.tenant_id)
        .bind(user.deleted_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn list(
        &self,
        tenant_id: Uuid,
        page: i32,
        page_size: i32,
        role: Option<String>,
        status: Option<String>,
    ) -> Result<(Vec<User>, i64), AppError> {
        // Clamp page to minimum of 1 and page_size to safe bounds (1-100)
        let page = page.max(1);
        let page_size = page_size.clamp(1, 100);

        // Use saturating arithmetic and convert to u64 for database offset
        let offset: u64 = ((page as u64).saturating_sub(1)).saturating_mul(page_size as u64);

        // Build query dynamically using QueryBuilder to prevent SQL injection
        let mut query_builder = sqlx::QueryBuilder::new("SELECT * FROM users WHERE tenant_id = ");
        query_builder.push_bind(tenant_id);
        query_builder.push(" AND deleted_at IS NULL");

        let mut count_builder =
            sqlx::QueryBuilder::new("SELECT COUNT(*) FROM users WHERE tenant_id = ");
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
        }
        // When status is None, show all statuses (active and suspended)
        // Do NOT default to active - allow admin to see all users

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

    /// Check if email exists for active (non-deleted) users only.
    /// Soft-deleted users (deleted_at IS NOT NULL) are excluded, allowing
    /// email re-registration after account deletion.
    async fn email_exists(&self, tenant_id: Uuid, email: &str) -> Result<bool, AppError> {
        let exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM users WHERE tenant_id = $1 AND email = $2 AND deleted_at IS NULL)",
        )
        .bind(tenant_id)
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists.0)
    }

    async fn find_by_email_global(&self, email: &str) -> Result<Option<User>, AppError> {
        // Find user by email across all tenants (for password reset when tenant is unknown)
        // Returns the most recently created active, non-deleted user found with this email
        // Note: ORDER BY ensures deterministic results when duplicates exist
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1 AND status = 'active' AND deleted_at IS NULL ORDER BY created_at DESC LIMIT 1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn hard_delete_by_id(&self, tenant_id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query("DELETE FROM users WHERE tenant_id = $1 AND user_id = $2")
            .bind(tenant_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

/// PostgreSQL implementation of TenantRepository
#[derive(Clone)]
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
                status, settings, owner_user_id, created_at, updated_at, deleted_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(tenant.tenant_id)
        .bind(&tenant.name)
        .bind(&tenant.slug)
        .bind(&tenant.plan)
        .bind(tenant.plan_expires_at)
        .bind(&tenant.status)
        .bind(&tenant.settings)
        .bind(tenant.owner_user_id)
        .bind(tenant.created_at)
        .bind(tenant.updated_at)
        .bind(tenant.deleted_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(tenant)
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Tenant>, AppError> {
        let tenant = sqlx::query_as::<_, Tenant>(
            "SELECT * FROM tenants WHERE name = $1 AND status = 'active' AND deleted_at IS NULL",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(tenant)
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Tenant>, AppError> {
        let tenant = sqlx::query_as::<_, Tenant>(
            "SELECT * FROM tenants WHERE slug = $1 AND status = 'active' AND deleted_at IS NULL",
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;

        Ok(tenant)
    }

    async fn set_owner(&self, tenant_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        // Wrap in transaction to avoid race conditions between existence check and update
        let mut tx = self.pool.begin().await?;

        // Verify user belongs to the tenant and is active before setting as owner
        let user_exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM users WHERE user_id = $1 AND tenant_id = $2 AND status = 'active' AND deleted_at IS NULL)"
        )
        .bind(user_id)
        .bind(tenant_id)
        .fetch_one(&mut *tx)
        .await?;

        if !user_exists.0 {
            return Err(AppError::ValidationError(format!(
                "User {} does not belong to tenant {} or is not active",
                user_id, tenant_id
            )));
        }

        let rows_affected = sqlx::query(
            r#"
            UPDATE tenants
            SET owner_user_id = $1, updated_at = NOW()
            WHERE tenant_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(user_id)
        .bind(tenant_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!("Tenant {} not found", tenant_id)));
        }

        tx.commit().await?;
        Ok(())
    }
}
