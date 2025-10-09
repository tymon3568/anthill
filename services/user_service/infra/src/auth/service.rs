use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;
use user_service_core::domains::auth::{
    domain::{
        model::{User, Tenant},
        repository::{UserRepository, TenantRepository},
        service::AuthService,
    },
    dto::auth_dto::{RegisterReq, LoginReq, RefreshReq, AuthResp, UserInfo, UserListResp},
};
use shared_error::AppError;
use shared_jwt::{Claims, encode_jwt, decode_jwt};

/// Auth service implementation
pub struct AuthServiceImpl<UR, TR>
where
    UR: UserRepository,
    TR: TenantRepository,
{
    user_repo: UR,
    tenant_repo: TR,
    jwt_secret: String,
    jwt_expiration: i64,
    jwt_refresh_expiration: i64,
}

impl<UR, TR> AuthServiceImpl<UR, TR>
where
    UR: UserRepository,
    TR: TenantRepository,
{
    pub fn new(
        user_repo: UR,
        tenant_repo: TR,
        jwt_secret: String,
        jwt_expiration: i64,
        jwt_refresh_expiration: i64,
    ) -> Self {
        Self {
            user_repo,
            tenant_repo,
            jwt_secret,
            jwt_expiration,
            jwt_refresh_expiration,
        }
    }
    
    fn user_to_user_info(&self, user: &User) -> UserInfo {
        UserInfo {
            id: user.id,
            email: user.email.clone(),
            full_name: user.full_name.clone(),
            tenant_id: user.tenant_id,
            role: user.role.clone(),
            created_at: user.created_at,
        }
    }
}

#[async_trait]
impl<UR, TR> AuthService for AuthServiceImpl<UR, TR>
where
    UR: UserRepository + Send + Sync,
    TR: TenantRepository + Send + Sync,
{
    async fn register(&self, req: RegisterReq) -> Result<AuthResp, AppError> {
        // Determine tenant
        let tenant = if let Some(tenant_name) = &req.tenant_name {
            // Create new tenant
            let tenant_id = Uuid::new_v4();
            let now = Utc::now();
            let tenant = Tenant {
                id: tenant_id,
                name: tenant_name.clone(),
                is_active: true,
                created_at: now,
                updated_at: now,
            };
            self.tenant_repo.create(&tenant).await?
        } else {
            // TODO: Handle multi-tenant scenarios
            return Err(AppError::ValidationError("Tenant name required for registration".to_string()));
        };
        
        // Check if user already exists
        if self.user_repo.email_exists(&req.email, tenant.id).await? {
            return Err(AppError::UserAlreadyExists);
        }
        
        // Hash password
        let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::InternalError(format!("Failed to hash password: {}", e)))?;
        
        // Create user
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        let user = User {
            id: user_id,
            email: req.email.clone(),
            password_hash,
            full_name: req.full_name.clone(),
            tenant_id: tenant.id,
            role: "user".to_string(), // Default role
            is_active: true,
            created_at: now,
            updated_at: now,
        };
        
        let created_user = self.user_repo.create(&user).await?;
        
        // Generate tokens
        let access_claims = Claims::new_access(
            created_user.id,
            created_user.tenant_id,
            created_user.role.clone(),
            self.jwt_expiration,
        );
        let refresh_claims = Claims::new_refresh(
            created_user.id,
            created_user.tenant_id,
            created_user.role.clone(),
            self.jwt_refresh_expiration,
        );
        
        let access_token = encode_jwt(&access_claims, &self.jwt_secret)?;
        let refresh_token = encode_jwt(&refresh_claims, &self.jwt_secret)?;
        
        Ok(AuthResp {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.jwt_expiration,
            user: self.user_to_user_info(&created_user),
        })
    }
    
    async fn login(&self, req: LoginReq) -> Result<AuthResp, AppError> {
        // TODO: In production, implement tenant resolution from email domain or subdomain
        // For now, we'll search across all tenants (not production-ready)
        
        // This is a simplified implementation - in production, you'd need proper tenant isolation
        return Err(AppError::InvalidCredentials);
        
        /* Production implementation would be:
        // 1. Resolve tenant from email domain or request context
        let tenant_id = resolve_tenant_from_context()?;
        
        // 2. Find user by email within tenant
        let user = self.user_repo
            .find_by_email(&req.email, tenant_id)
            .await?
            .ok_or(AppError::InvalidCredentials)?;
        
        // 3. Verify password
        let valid = bcrypt::verify(&req.password, &user.password_hash)
            .map_err(|e| AppError::InternalError(format!("Password verification failed: {}", e)))?;
        
        if !valid {
            return Err(AppError::InvalidCredentials);
        }
        
        // 4. Generate tokens
        let access_claims = Claims::new_access(
            user.id,
            user.tenant_id,
            user.role.clone(),
            self.jwt_expiration,
        );
        let refresh_claims = Claims::new_refresh(
            user.id,
            user.tenant_id,
            user.role.clone(),
            self.jwt_refresh_expiration,
        );
        
        let access_token = encode_jwt(&access_claims, &self.jwt_secret)?;
        let refresh_token = encode_jwt(&refresh_claims, &self.jwt_secret)?;
        
        Ok(AuthResp {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.jwt_expiration,
            user: self.user_to_user_info(&user),
        })
        */
    }
    
    async fn refresh_token(&self, req: RefreshReq) -> Result<AuthResp, AppError> {
        // Decode and validate refresh token
        let claims = decode_jwt(&req.refresh_token, &self.jwt_secret)?;
        
        // Verify it's a refresh token
        if claims.token_type != "refresh" {
            return Err(AppError::InvalidToken);
        }
        
        // Get user to ensure still active
        let user = self.user_repo
            .find_by_id(claims.sub, claims.tenant_id)
            .await?
            .ok_or(AppError::UserNotFound)?;
        
        // Generate new tokens
        let new_access_claims = Claims::new_access(
            user.id,
            user.tenant_id,
            user.role.clone(),
            self.jwt_expiration,
        );
        let new_refresh_claims = Claims::new_refresh(
            user.id,
            user.tenant_id,
            user.role.clone(),
            self.jwt_refresh_expiration,
        );
        
        let access_token = encode_jwt(&new_access_claims, &self.jwt_secret)?;
        let refresh_token = encode_jwt(&new_refresh_claims, &self.jwt_secret)?;
        
        Ok(AuthResp {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.jwt_expiration,
            user: self.user_to_user_info(&user),
        })
    }
    
    async fn list_users(&self, tenant_id: Uuid, page: i32, page_size: i32) -> Result<UserListResp, AppError> {
        let (users, total) = self.user_repo.list(tenant_id, page, page_size).await?;
        
        let user_infos = users.iter().map(|u| self.user_to_user_info(u)).collect();
        
        Ok(UserListResp {
            users: user_infos,
            total,
            page,
            page_size,
        })
    }
    
    async fn get_user(&self, user_id: Uuid, tenant_id: Uuid) -> Result<UserInfo, AppError> {
        let user = self.user_repo
            .find_by_id(user_id, tenant_id)
            .await?
            .ok_or(AppError::UserNotFound)?;
        
        Ok(self.user_to_user_info(&user))
    }
}
