use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use user_service_core::domains::auth::{
    domain::{
        model::{User, Tenant, Session},
        repository::{UserRepository, TenantRepository, SessionRepository},
        service::AuthService,
    },
    dto::auth_dto::{RegisterReq, LoginReq, RefreshReq, AuthResp, UserInfo, UserListResp},
    utils::password_validator::validate_password_quick,
};
use shared_error::AppError;
use shared_jwt::{Claims, encode_jwt, decode_jwt};
use serde_json;

/// Auth service implementation
pub struct AuthServiceImpl<UR, TR, SR>
where
    UR: UserRepository,
    TR: TenantRepository,
    SR: SessionRepository,
{
    user_repo: UR,
    tenant_repo: TR,
    session_repo: SR,
    jwt_secret: String,
    jwt_expiration: i64,
    jwt_refresh_expiration: i64,
}

impl<UR, TR, SR> AuthServiceImpl<UR, TR, SR>
where
    UR: UserRepository,
    TR: TenantRepository,
    SR: SessionRepository,
{
    pub fn new(
        user_repo: UR,
        tenant_repo: TR,
        session_repo: SR,
        jwt_secret: String,
        jwt_expiration: i64,
        jwt_refresh_expiration: i64,
    ) -> Self {
        Self {
            user_repo,
            tenant_repo,
            session_repo,
            jwt_secret,
            jwt_expiration,
            jwt_refresh_expiration,
        }
    }
    
    fn user_to_user_info(&self, user: &User) -> UserInfo {
        UserInfo {
            id: user.user_id,
            email: user.email.clone(),
            full_name: user.full_name.clone(),
            tenant_id: user.tenant_id,
            role: user.role.clone(),
            created_at: user.created_at,
        }
    }
    
    /// Hash token using SHA-256 for secure storage
    fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Create session record for tokens
    async fn create_session(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        access_token: &str,
        refresh_token: &str,
        access_exp: i64,
        refresh_exp: i64,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<Session, AppError> {
        let now = Utc::now();
        let session = Session {
            session_id: Uuid::new_v4(),
            user_id,
            tenant_id,
            access_token_hash: self.hash_token(access_token),
            refresh_token_hash: self.hash_token(refresh_token),
            ip_address,
            user_agent,
            device_info: None,
            access_token_expires_at: now + chrono::Duration::seconds(access_exp),
            refresh_token_expires_at: now + chrono::Duration::seconds(refresh_exp),
            revoked: false,
            revoked_at: None,
            revoked_reason: None,
            created_at: now,
            last_used_at: now,
        };
        
        self.session_repo.create(&session).await
    }
}

#[async_trait]
impl<UR, TR, SR> AuthService for AuthServiceImpl<UR, TR, SR>
where
    UR: UserRepository + Send + Sync,
    TR: TenantRepository + Send + Sync,
    SR: SessionRepository + Send + Sync,
{
    async fn register(
        &self,
        req: RegisterReq,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuthResp, AppError> {
        // Determine tenant
        let tenant = if let Some(tenant_name) = &req.tenant_name {
            // Create new tenant
            let tenant_id = Uuid::new_v4();
            let now = Utc::now();
            // Generate slug from tenant name (simple implementation)
            let slug = tenant_name.to_lowercase().replace(" ", "-");
            let tenant = Tenant {
                tenant_id,
                name: tenant_name.clone(),
                slug,
                plan: "free".to_string(), // Default plan
                plan_expires_at: None,
                settings: sqlx::types::Json(serde_json::json!({})), // Empty settings
                status: "active".to_string(),
                created_at: now,
                updated_at: now,
                deleted_at: None,
            };
            self.tenant_repo.create(&tenant).await?
        } else {
            // TODO: Handle multi-tenant scenarios
            return Err(AppError::ValidationError("Tenant name required for registration".to_string()));
        };
        
        // Check if user already exists
        if self.user_repo.email_exists(&req.email, tenant.tenant_id).await? {
            return Err(AppError::UserAlreadyExists);
        }
        
        // Validate password strength
        let user_inputs = [
            req.email.as_str(),
            req.full_name.as_str(),
            tenant.name.as_str(),
        ];
        validate_password_quick(&req.password, &user_inputs)
            .map_err(|e| AppError::ValidationError(format!("Password validation failed: {}", e)))?;
        
        // Hash password
        let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::InternalError(format!("Failed to hash password: {}", e)))?;
        
        // Create user
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        let user = User {
            user_id,
            tenant_id: tenant.tenant_id,
            email: req.email.clone(),
            password_hash,
            email_verified: false, // Default to unverified
            email_verified_at: None,
            full_name: Some(req.full_name.clone()),
            avatar_url: None,
            phone: None,
            role: "user".to_string(), // Default role
            status: "active".to_string(),
            last_login_at: None,
            failed_login_attempts: 0,
            locked_until: None,
            password_changed_at: Some(now), // Password just set
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };
        
        let created_user = self.user_repo.create(&user).await?;
        
        // Generate tokens
        let access_claims = Claims::new_access(
            created_user.user_id,
            created_user.tenant_id,
            created_user.role.clone(),
            self.jwt_expiration,
        );
        let refresh_claims = Claims::new_refresh(
            created_user.user_id,
            created_user.tenant_id,
            created_user.role.clone(),
            self.jwt_refresh_expiration,
        );
        
        let access_token = encode_jwt(&access_claims, &self.jwt_secret)?;
        let refresh_token = encode_jwt(&refresh_claims, &self.jwt_secret)?;
        
        // Create session record
        self.create_session(
            created_user.user_id,
            created_user.tenant_id,
            &access_token,
            &refresh_token,
            self.jwt_expiration,
            self.jwt_refresh_expiration,
            ip_address,
            user_agent,
        ).await?;
        
        Ok(AuthResp {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.jwt_expiration,
            user: self.user_to_user_info(&created_user),
        })
    }
    
    async fn login(
        &self,
        _req: LoginReq,
        _ip_address: Option<String>,
        _user_agent: Option<String>,
    ) -> Result<AuthResp, AppError> {
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
            user.user_id,
            user.tenant_id,
            user.role.clone(),
            self.jwt_expiration,
        );
        let refresh_claims = Claims::new_refresh(
            user.user_id,
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
    
    async fn refresh_token(
        &self,
        req: RefreshReq,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuthResp, AppError> {
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
            user.user_id,
            user.tenant_id,
            user.role.clone(),
            self.jwt_expiration,
        );
        let new_refresh_claims = Claims::new_refresh(
            user.user_id,
            user.tenant_id,
            user.role.clone(),
            self.jwt_refresh_expiration,
        );
        
        let access_token = encode_jwt(&new_access_claims, &self.jwt_secret)?;
        let refresh_token = encode_jwt(&new_refresh_claims, &self.jwt_secret)?;
        
        // Revoke old session and create new one
        let old_token_hash = self.hash_token(&req.refresh_token);
        if let Some(old_session) = self.session_repo.find_by_refresh_token(&old_token_hash).await? {
            self.session_repo.revoke(old_session.session_id, "Token refreshed").await?;
        }
        
        // Create new session
        self.create_session(
            user.user_id,
            user.tenant_id,
            &access_token,
            &refresh_token,
            self.jwt_expiration,
            self.jwt_refresh_expiration,
            ip_address,
            user_agent,
        ).await?;
        
        Ok(AuthResp {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.jwt_expiration,
            user: self.user_to_user_info(&user),
        })
    }
    
    async fn logout(&self, refresh_token: &str) -> Result<(), AppError> {
        // Hash the refresh token to find the session
        let token_hash = self.hash_token(refresh_token);
        
        // Find and revoke the session
        if let Some(session) = self.session_repo.find_by_refresh_token(&token_hash).await? {
            self.session_repo.revoke(session.session_id, "User logout").await?;
        } else {
            // If session not found, it might be already revoked or invalid
            return Err(AppError::InvalidToken);
        }
        
        Ok(())
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
