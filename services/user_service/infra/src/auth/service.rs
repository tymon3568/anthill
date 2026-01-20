use async_trait::async_trait;
use chrono::Utc;
use serde_json;
use sha2::{Digest, Sha256};
use shared_error::AppError;
use shared_jwt::{decode_jwt, encode_jwt, Claims};
use user_service_core::domains::auth::{
    domain::{
        model::{Session, Tenant, User},
        repository::{SessionRepository, TenantRepository, UserRepository},
        service::AuthService,
    },
    dto::admin_dto::{AdminCreateUserReq, AdminCreateUserResp, PROTECTED_ROLES},
    dto::auth_dto::{
        AuthResp, LoginReq, RefreshReq, RegisterReq, RegisterResp, UserInfo, UserListResp,
    },
    utils::password_validator::validate_password_quick,
};
use uuid::Uuid;

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
            roles: vec![], // Will be populated by handler with Casbin roles
            status: user.status.clone(),
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
            access_token_hash: Some(self.hash_token(access_token)), // Now Option<String>
            refresh_token_hash: Some(self.hash_token(refresh_token)), // Now Option<String>
            ip_address,
            user_agent,
            device_info: None,
            access_token_expires_at: now + chrono::Duration::seconds(access_exp),
            refresh_token_expires_at: now + chrono::Duration::seconds(refresh_exp),
            revoked: false,
            revoked_at: None,
            revoked_reason: None,
            auth_method: "jwt".to_string(),
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
        _ip_address: Option<String>,
        _user_agent: Option<String>,
    ) -> Result<RegisterResp, AppError> {
        // Determine tenant and whether it's a new or existing tenant
        let (tenant, is_new_tenant) = if let Some(tenant_name) = &req.tenant_name {
            // Generate URL-safe slug from tenant name
            // Handles special characters, multiple spaces, and Unicode
            let slug = generate_slug(tenant_name).ok_or_else(|| {
                AppError::ValidationError(
                    "Tenant name must contain at least one alphanumeric character".to_string(),
                )
            })?;

            // Check if tenant with this slug already exists
            if let Some(existing_tenant) = self.tenant_repo.find_by_slug(&slug).await? {
                // Tenant exists - user will join with default 'user' role
                (existing_tenant, false)
            } else {
                // Create new tenant - user will become 'owner'
                let tenant_id = Uuid::now_v7();
                let now = Utc::now();
                let tenant = Tenant {
                    tenant_id,
                    name: tenant_name.clone(),
                    slug: slug.clone(),
                    plan: "free".to_string(), // Default plan
                    plan_expires_at: None,
                    settings: sqlx::types::Json(serde_json::json!({})), // Empty settings
                    status: "active".to_string(),
                    owner_user_id: None, // Will be set after user creation
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                };

                // Handle potential race condition: another request may have created
                // the same tenant between our check and create. On error, re-check
                // if the tenant now exists and treat as existing if so.
                match self.tenant_repo.create(&tenant).await {
                    Ok(created_tenant) => (created_tenant, true),
                    Err(e) => {
                        // Log the original error for debugging
                        tracing::warn!(error = ?e, slug = %slug, "Tenant creation failed, checking for race condition");
                        // Re-check if tenant was created by a concurrent request
                        if let Some(existing_tenant) = self.tenant_repo.find_by_slug(&slug).await? {
                            // Tenant was created by another request - join as user
                            (existing_tenant, false)
                        } else {
                            // Still doesn't exist - propagate the original error
                            return Err(AppError::InternalError(
                                "Failed to create tenant".to_string(),
                            ));
                        }
                    },
                }
            }
        } else {
            // Tenant name is required for registration
            return Err(AppError::ValidationError(
                "Tenant name required for registration".to_string(),
            ));
        };

        // Check if user already exists
        if self
            .user_repo
            .email_exists(tenant.tenant_id, &req.email)
            .await?
        {
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

        // Determine user role based on tenant creation:
        // - New tenant: user becomes 'owner' (tenant creator)
        // - Existing tenant: user gets default 'user' role
        let user_role = if is_new_tenant { "owner" } else { "user" };

        // Create user
        let user_id = Uuid::now_v7();
        let now = Utc::now();
        let user = User {
            user_id,
            tenant_id: tenant.tenant_id,
            email: req.email.clone(),
            password_hash: Some(password_hash), // Now Option<String>
            email_verified: false,              // Default to unverified
            email_verified_at: None,
            full_name: Some(req.full_name.clone()),
            avatar_url: None,
            phone: None,
            role: user_role.to_string(), // 'owner' for new tenant, 'user' for existing
            status: "active".to_string(),
            last_login_at: None,
            failed_login_attempts: 0,
            locked_until: None,
            password_changed_at: Some(now),
            auth_method: "password".to_string(),
            migration_invited_at: None,   // Not invited yet
            migration_completed_at: None, // Not migrated
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created_user = self.user_repo.create(&user).await?;

        // If this is a new tenant, set the registering user as the tenant owner
        if is_new_tenant {
            self.tenant_repo
                .set_owner(tenant.tenant_id, created_user.user_id)
                .await?;
        }

        // Return RegisterResp without tokens - user must verify email first
        // No session is created until user logs in after email verification
        Ok(RegisterResp {
            message: "Registration successful. Please check your email to verify your account."
                .to_string(),
            user_id: created_user.user_id,
            tenant_id: created_user.tenant_id,
            email: created_user.email,
            requires_email_verification: true,
        })
    }

    async fn login(
        &self,
        req: LoginReq,
        tenant_identifier: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuthResp, AppError> {
        // Resolve tenant from identifier
        let tenant_id = match tenant_identifier {
            Some(id_str) => {
                // Try parsing as UUID first (direct ID)
                if let Ok(id) = Uuid::parse_str(&id_str) {
                    if let Some(tenant) = self.tenant_repo.find_by_id(id).await? {
                        // Check if tenant is active
                        if tenant.status != "active" {
                            return Err(AppError::ValidationError(
                                "Tenant is not active".to_string(),
                            ));
                        }
                        tenant.tenant_id
                    } else {
                        return Err(AppError::ValidationError("Tenant not found".to_string()));
                    }
                } else {
                    // Treat as slug/subdomain
                    if let Some(tenant) = self.tenant_repo.find_by_slug(&id_str).await? {
                        // Check if tenant is active
                        if tenant.status != "active" {
                            return Err(AppError::ValidationError(
                                "Tenant is not active".to_string(),
                            ));
                        }
                        tenant.tenant_id
                    } else {
                        return Err(AppError::ValidationError("Tenant not found".to_string()));
                    }
                }
            },
            None => {
                return Err(AppError::ValidationError(
                    "Tenant context required for login".to_string(),
                ));
            },
        };

        let user = self
            .user_repo
            .find_by_email(tenant_id, &req.email)
            .await?
            .ok_or(AppError::InvalidCredentials)?;

        // Verify password (must have password_hash for password auth)
        let password_hash = user.password_hash.as_ref().ok_or_else(|| {
            AppError::ValidationError(
                "Password authentication not configured for this account. Please use the password reset flow to set a password."
                    .to_string(),
            )
        })?;

        let valid = bcrypt::verify(&req.password, password_hash)
            .map_err(|e| AppError::InternalError(format!("Password verification failed: {}", e)))?;

        if !valid {
            return Err(AppError::InvalidCredentials);
        }

        // Check if account is active
        if user.status != "active" {
            return Err(AppError::InvalidCredentials);
        }

        // Check if email is verified
        if !user.email_verified {
            return Err(AppError::ValidationError(
                "Email not verified. Please check your inbox and verify your email before logging in.".to_string(),
            ));
        }

        // Check if account is locked
        if let Some(locked_until) = user.locked_until {
            if locked_until > chrono::Utc::now() {
                return Err(AppError::ValidationError("Account is temporarily locked".to_string()));
            }
        }

        // Generate JWT tokens
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

        // Hash tokens before storing in database (SHA-256)
        let access_token_hash = format!("{:x}", Sha256::digest(access_token.as_bytes()));
        let refresh_token_hash = format!("{:x}", Sha256::digest(refresh_token.as_bytes()));

        // Create session
        let session = Session {
            session_id: Uuid::now_v7(),
            user_id: user.user_id,
            tenant_id: user.tenant_id,
            access_token_hash: Some(access_token_hash), // Now Option<String>
            refresh_token_hash: Some(refresh_token_hash), // Now Option<String>
            ip_address,
            user_agent,
            device_info: None,
            access_token_expires_at: chrono::Utc::now()
                + chrono::Duration::seconds(self.jwt_expiration),
            refresh_token_expires_at: chrono::Utc::now()
                + chrono::Duration::seconds(self.jwt_refresh_expiration),
            revoked: false,
            revoked_at: None,
            revoked_reason: None,
            auth_method: "jwt".to_string(),
            created_at: chrono::Utc::now(),
            last_used_at: chrono::Utc::now(),
        };

        self.session_repo.create(&session).await?;

        // Update last login timestamp (via repository if available, otherwise skip for now)
        // TODO: Add update_last_login to UserRepository

        Ok(AuthResp {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.jwt_expiration,
            user: UserInfo {
                id: user.user_id,
                tenant_id: user.tenant_id,
                email: user.email,
                full_name: user.full_name,
                role: user.role,
                roles: vec![], // Will be populated by handler with Casbin roles if needed
                status: user.status,
                created_at: user.created_at,
            },
        })
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
        let user = self
            .user_repo
            .find_by_id(claims.tenant_id, claims.sub)
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
        if let Some(old_session) = self
            .session_repo
            .find_by_refresh_token(&old_token_hash)
            .await?
        {
            self.session_repo
                .revoke(old_session.session_id, "Token refreshed")
                .await?;
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
        )
        .await?;

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
            self.session_repo
                .revoke(session.session_id, "User logout")
                .await?;
        } else {
            // If session not found, it might be already revoked or invalid
            return Err(AppError::InvalidToken);
        }

        Ok(())
    }

    async fn list_users(
        &self,
        tenant_id: Uuid,
        page: i32,
        page_size: i32,
        role: Option<String>,
        status: Option<String>,
    ) -> Result<UserListResp, AppError> {
        let (users, total) = self
            .user_repo
            .list(tenant_id, page, page_size, role, status)
            .await?;

        let user_infos = users.iter().map(|u| self.user_to_user_info(u)).collect();

        Ok(UserListResp {
            users: user_infos,
            total,
            page,
            page_size,
        })
    }

    async fn get_user(&self, user_id: Uuid, tenant_id: Uuid) -> Result<UserInfo, AppError> {
        let user = self
            .user_repo
            .find_by_id(tenant_id, user_id)
            .await?
            .ok_or(AppError::UserNotFound)?;

        Ok(self.user_to_user_info(&user))
    }

    async fn get_user_any_status(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<UserInfo, AppError> {
        let user = self
            .user_repo
            .find_by_id_any_status(tenant_id, user_id)
            .await?
            .ok_or(AppError::UserNotFound)?;

        Ok(self.user_to_user_info(&user))
    }

    async fn cleanup_stale_sessions(&self) -> Result<u64, AppError> {
        // Cleanup stale sessions (expired refresh tokens and old revoked sessions)
        tracing::info!("Running cleanup of stale sessions");

        let deleted = self.session_repo.delete_expired().await?;
        tracing::info!("Cleanup finished, {} sessions removed", deleted);

        Ok(deleted)
    }

    async fn admin_create_user(
        &self,
        admin_tenant_id: Uuid,
        req: AdminCreateUserReq,
    ) -> Result<AdminCreateUserResp, AppError> {
        // Validate tenant exists and is active before creating user
        let tenant = self
            .tenant_repo
            .find_by_id(admin_tenant_id)
            .await?
            .ok_or_else(|| AppError::ValidationError("Tenant not found".to_string()))?;

        if tenant.status != "active" {
            return Err(AppError::ValidationError(
                "Cannot create user in inactive tenant".to_string(),
            ));
        }

        // Determine the role (default to "user" if not specified)
        let role = req.role.as_deref().unwrap_or("user");

        // Prevent creating users with protected roles (e.g., "owner") via this endpoint
        // These roles can only be assigned via specific flows (e.g., tenant bootstrap)
        if PROTECTED_ROLES.contains(&role) {
            return Err(AppError::Forbidden(format!(
                "Cannot create user with '{}' role. This role is protected.",
                role
            )));
        }

        // Validate role: system roles (admin, user) are always valid
        // Custom roles are allowed if they pass DTO format validation
        // Actual authorization for custom roles is handled by Casbin at runtime
        let is_system_role = matches!(role, "admin" | "user");
        if !is_system_role {
            tracing::info!(
                role = %role,
                tenant_id = %admin_tenant_id,
                "Creating user with custom role"
            );
        }

        // Check if user already exists in this tenant
        if self
            .user_repo
            .email_exists(admin_tenant_id, &req.email)
            .await?
        {
            return Err(AppError::UserAlreadyExists);
        }

        // Validate password strength (include tenant name for consistency with registration)
        let full_name = req.full_name.as_deref().unwrap_or("");
        let user_inputs = [req.email.as_str(), full_name, tenant.name.as_str()];
        validate_password_quick(&req.password, &user_inputs)
            .map_err(|e| AppError::ValidationError(format!("Password validation failed: {}", e)))?;

        // Hash password with bcrypt
        let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::InternalError(format!("Failed to hash password: {}", e)))?;

        // Create user with UUID v7
        let user_id = Uuid::now_v7();
        let now = Utc::now();
        let user = User {
            user_id,
            tenant_id: admin_tenant_id,
            email: req.email.clone(),
            password_hash: Some(password_hash),
            email_verified: false,
            email_verified_at: None,
            full_name: req.full_name.clone(),
            avatar_url: None,
            phone: None,
            role: role.to_string(),
            status: "active".to_string(),
            last_login_at: None,
            failed_login_attempts: 0,
            locked_until: None,
            password_changed_at: Some(now),
            auth_method: "password".to_string(),
            migration_invited_at: None,
            migration_completed_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created_user = self.user_repo.create(&user).await?;

        tracing::info!(
            user_id = %created_user.user_id,
            tenant_id = %admin_tenant_id,
            role = %created_user.role,
            "Admin created new user in tenant"
        );

        Ok(AdminCreateUserResp {
            user_id: created_user.user_id,
            tenant_id: created_user.tenant_id,
            email: created_user.email,
            full_name: created_user.full_name,
            role: created_user.role,
            created_at: created_user.created_at,
            message: "User created successfully".to_string(),
        })
    }

    async fn internal_delete_user(&self, user_id: Uuid, tenant_id: Uuid) -> Result<bool, AppError> {
        tracing::warn!(
            user_id = %user_id,
            tenant_id = %tenant_id,
            "Performing compensating delete of user (rolling back failed operation)"
        );

        let deleted = self.user_repo.hard_delete_by_id(tenant_id, user_id).await?;

        if deleted {
            tracing::info!(
                user_id = %user_id,
                tenant_id = %tenant_id,
                "Successfully deleted user as compensating action"
            );
        } else {
            tracing::warn!(
                user_id = %user_id,
                tenant_id = %tenant_id,
                "User not found during compensating delete (may have been already deleted)"
            );
        }

        Ok(deleted)
    }

    async fn admin_suspend_user(
        &self,
        admin_tenant_id: Uuid,
        admin_user_id: Uuid,
        target_user_id: Uuid,
        reason: Option<String>,
    ) -> Result<user_service_core::domains::auth::dto::admin_dto::SuspendUserResp, AppError> {
        // 1. Fetch target user (tenant isolation enforced)
        // Use find_by_id_any_status to allow suspending users with any status
        let mut user = self
            .user_repo
            .find_by_id_any_status(admin_tenant_id, target_user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found in tenant".to_string()))?;

        // 2. Owner protection: check if target is tenant owner
        let tenant = self
            .tenant_repo
            .find_by_id(admin_tenant_id)
            .await?
            .ok_or_else(|| AppError::InternalError("Tenant not found".to_string()))?;

        if let Some(owner_id) = tenant.owner_user_id {
            if user.user_id == owner_id && admin_user_id != owner_id {
                return Err(AppError::Forbidden("Cannot suspend tenant owner".to_string()));
            }
        }

        // 3. Check if already suspended
        if user.status == "suspended" {
            return Err(AppError::ValidationError("User is already suspended".to_string()));
        }

        // 4. Update user status
        user.status = "suspended".to_string();
        user.updated_at = Utc::now();
        let updated_user = self.user_repo.update(&user).await?;

        // 5. Revoke all sessions (force logout)
        let sessions_revoked = self
            .session_repo
            .revoke_all_for_user(target_user_id)
            .await?;

        tracing::info!(
            admin_user_id = %admin_user_id,
            target_user_id = %target_user_id,
            tenant_id = %admin_tenant_id,
            sessions_revoked = %sessions_revoked,
            reason = ?reason,
            "User suspended by admin"
        );

        Ok(user_service_core::domains::auth::dto::admin_dto::SuspendUserResp {
            user_id: updated_user.user_id,
            email: updated_user.email,
            status: updated_user.status,
            message: format!("User suspended successfully. {} sessions revoked.", sessions_revoked),
        })
    }

    async fn admin_unsuspend_user(
        &self,
        admin_tenant_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<user_service_core::domains::auth::dto::admin_dto::UnsuspendUserResp, AppError> {
        // 1. Fetch target user
        // Use find_by_id_any_status to allow unsuspending users with any status
        let mut user = self
            .user_repo
            .find_by_id_any_status(admin_tenant_id, target_user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found in tenant".to_string()))?;

        // 2. Check if not suspended
        if user.status != "suspended" {
            return Err(AppError::ValidationError("User is not suspended".to_string()));
        }

        // 3. Update status to active
        user.status = "active".to_string();
        user.updated_at = Utc::now();
        let updated_user = self.user_repo.update(&user).await?;

        tracing::info!(
            target_user_id = %target_user_id,
            tenant_id = %admin_tenant_id,
            "User unsuspended by admin"
        );

        Ok(user_service_core::domains::auth::dto::admin_dto::UnsuspendUserResp {
            user_id: updated_user.user_id,
            email: updated_user.email,
            status: updated_user.status,
            message: "User unsuspended successfully".to_string(),
        })
    }

    async fn admin_delete_user(
        &self,
        admin_tenant_id: Uuid,
        admin_user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<user_service_core::domains::auth::dto::admin_dto::DeleteUserResp, AppError> {
        // 1. Fetch target user
        // Use find_by_id_any_status to allow deleting users with any status
        let mut user = self
            .user_repo
            .find_by_id_any_status(admin_tenant_id, target_user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found in tenant".to_string()))?;

        // 2. Owner protection
        let tenant = self
            .tenant_repo
            .find_by_id(admin_tenant_id)
            .await?
            .ok_or_else(|| AppError::InternalError("Tenant not found".to_string()))?;

        if let Some(owner_id) = tenant.owner_user_id {
            if user.user_id == owner_id && admin_user_id != owner_id {
                return Err(AppError::Forbidden("Cannot delete tenant owner".to_string()));
            }
        }

        // 3. Check if already deleted
        if user.deleted_at.is_some() {
            return Err(AppError::ValidationError("User is already deleted".to_string()));
        }

        // 4. Soft delete user
        let now = Utc::now();
        user.deleted_at = Some(now);
        user.status = "inactive".to_string();
        user.updated_at = now;
        let updated_user = self.user_repo.update(&user).await?;

        // 5. Revoke all sessions
        let sessions_revoked = self
            .session_repo
            .revoke_all_for_user(target_user_id)
            .await?;

        tracing::warn!(
            admin_user_id = %admin_user_id,
            target_user_id = %target_user_id,
            tenant_id = %admin_tenant_id,
            sessions_revoked = %sessions_revoked,
            "User soft-deleted by admin"
        );

        Ok(user_service_core::domains::auth::dto::admin_dto::DeleteUserResp {
            user_id: updated_user.user_id,
            email: updated_user.email,
            deleted_at: now,
            message: format!("User deleted successfully. {} sessions revoked.", sessions_revoked),
        })
    }

    async fn admin_reset_password(
        &self,
        admin_tenant_id: Uuid,
        target_user_id: Uuid,
        new_password: String,
        force_logout: bool,
    ) -> Result<user_service_core::domains::auth::dto::admin_dto::AdminResetPasswordResp, AppError>
    {
        // 1. Validate password strength
        validate_password_quick(&new_password, &[]).map_err(AppError::ValidationError)?;

        // 2. Fetch target user
        // Use find_by_id_any_status to allow resetting password for suspended users
        let mut user = self
            .user_repo
            .find_by_id_any_status(admin_tenant_id, target_user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found in tenant".to_string()))?;

        // 3. Hash new password
        let password_hash = bcrypt::hash(&new_password, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::InternalError(format!("Failed to hash password: {}", e)))?;

        // 4. Update user password and timestamp
        user.password_hash = Some(password_hash);
        user.password_changed_at = Some(Utc::now());
        user.updated_at = Utc::now();
        let updated_user = self.user_repo.update(&user).await?;

        // 5. Optionally revoke all sessions (force logout)
        let sessions_revoked = if force_logout {
            self.session_repo
                .revoke_all_for_user(target_user_id)
                .await?
        } else {
            0
        };

        tracing::warn!(
            target_user_id = %target_user_id,
            tenant_id = %admin_tenant_id,
            force_logout = %force_logout,
            sessions_revoked = %sessions_revoked,
            "Admin reset user password"
        );

        Ok(user_service_core::domains::auth::dto::admin_dto::AdminResetPasswordResp {
            user_id: updated_user.user_id,
            email: updated_user.email,
            sessions_revoked,
            message: if force_logout {
                format!("Password reset successfully. {} sessions revoked.", sessions_revoked)
            } else {
                "Password reset successfully. User sessions remain active.".to_string()
            },
        })
    }
}

/// Generate a URL-safe slug from a name
/// - Converts to lowercase
/// - Replaces non-alphanumeric chars with hyphens
/// - Removes consecutive hyphens
/// - Trims leading/trailing hyphens
/// - Returns None if result would be empty
fn generate_slug(name: &str) -> Option<String> {
    let slug = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if slug.is_empty() {
        None
    } else {
        Some(slug)
    }
}
