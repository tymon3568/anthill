use crate::cookie_helper::{clear_auth_cookies, get_cookie_value, set_auth_cookies, CookieConfig};
use crate::rate_limiter::InvitationRateLimiter;
use axum::{
    extract::{Extension, Query},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use shared_auth::casbin::{CoreApi, MgmtApi};
use shared_auth::enforcer::{add_role_for_user, copy_policies_for_tenant, SharedEnforcer};
use shared_auth::extractors::{AuthUser, JwtSecretProvider, RequireAdmin};
use shared_error::AppError;
use std::sync::Arc;
use user_service_core::domains::auth::{
    domain::{
        authz_version_repository::AuthzVersionRepository,
        email_verification_service::EmailVerificationService,
        invitation_service::InvitationService,
        repository::{TenantRepository, UserRepository},
        service::AuthService,
    },
    dto::auth_dto::{
        AuthResp, CheckTenantSlugQuery, CheckTenantSlugResp, ErrorResp, HealthResp, LoginReq,
        OptionalRefreshReq, RefreshReq, RegisterReq, RegisterResp, UserInfo, UserListResp,
    },
};
use user_service_infra::auth::EmailSender;
use uuid::Uuid;

/// Application state containing service dependencies
pub struct AppState<S: AuthService> {
    pub auth_service: Arc<S>,
    pub enforcer: SharedEnforcer,
    pub jwt_secret: String,
    // Repositories for user/tenant management
    pub user_repo: Option<Arc<dyn UserRepository>>,
    pub tenant_repo: Option<Arc<dyn TenantRepository>>,
    // Invitation service
    pub invitation_service: Option<Arc<dyn InvitationService + Send + Sync>>,
    // Email verification service
    pub email_verification_service: Option<Arc<dyn EmailVerificationService + Send + Sync>>,
    // AuthZ version repository for immediate-effect permission invalidation
    pub authz_version_repo: Option<Arc<dyn AuthzVersionRepository>>,
    // Configuration for invitation settings
    pub config: shared_config::Config,
    // Rate limiter for invitation acceptance
    pub invitation_rate_limiter: Arc<InvitationRateLimiter>,
    // Email sender for sending invitation emails
    pub email_sender: Option<Arc<dyn EmailSender>>,
}

impl<S: AuthService> Clone for AppState<S> {
    fn clone(&self) -> Self {
        Self {
            auth_service: Arc::clone(&self.auth_service),
            enforcer: self.enforcer.clone(),
            jwt_secret: self.jwt_secret.clone(),
            user_repo: self.user_repo.clone(),
            tenant_repo: self.tenant_repo.clone(),
            invitation_service: self.invitation_service.clone(),
            email_verification_service: self.email_verification_service.clone(),
            authz_version_repo: self.authz_version_repo.clone(),
            config: self.config.clone(),
            invitation_rate_limiter: Arc::clone(&self.invitation_rate_limiter),
            email_sender: self.email_sender.clone(),
        }
    }
}

impl<S: AuthService> JwtSecretProvider for AppState<S> {
    fn get_jwt_secret(&self) -> &str {
        &self.jwt_secret
    }
}

// ============================================================================
// AuthZ Version Helpers
// ============================================================================

/// Bump the tenant authorization version after a role/policy change.
///
/// This helper is called after successful `save_policy()` to ensure tokens
/// minted before the change are rejected by the AuthZ version middleware.
///
/// If Redis is not configured, this is a no-op (logs a debug message).
/// If the bump fails, logs an error but does not fail the request.
pub(crate) async fn bump_tenant_authz_version<S: AuthService>(
    state: &AppState<S>,
    tenant_id: Uuid,
    operation: &str,
) {
    if let Some(ref version_repo) = state.authz_version_repo {
        match version_repo.bump_tenant_version(tenant_id).await {
            Ok(new_version) => {
                tracing::info!(
                    tenant_id = %tenant_id,
                    new_version = new_version,
                    operation = operation,
                    "Bumped tenant authz version after policy change"
                );
            },
            Err(e) => {
                // Log error but don't fail the request - middleware will fallback to DB
                tracing::error!(
                    tenant_id = %tenant_id,
                    operation = operation,
                    error = %e,
                    "Failed to bump tenant authz version (middleware will fallback to DB)"
                );
            },
        }
    } else {
        tracing::debug!(
            tenant_id = %tenant_id,
            operation = operation,
            "Skipping tenant authz version bump (Redis not configured)"
        );
    }
}

/// Bump the user authorization version after a user-level security change.
///
/// This helper is called after successful user mutations (role assignment,
/// suspend, password reset, etc.) to ensure tokens minted before the change
/// are rejected by the AuthZ version middleware.
///
/// If Redis is not configured, this is a no-op (logs a debug message).
/// If the bump fails, logs an error but does not fail the request.
pub(crate) async fn bump_user_authz_version<S: AuthService>(
    state: &AppState<S>,
    user_id: Uuid,
    tenant_id: Uuid,
    operation: &str,
) {
    if let Some(ref version_repo) = state.authz_version_repo {
        match version_repo.bump_user_version(user_id).await {
            Ok(new_version) => {
                tracing::info!(
                    user_id = %user_id,
                    tenant_id = %tenant_id,
                    new_version = new_version,
                    operation = operation,
                    "Bumped user authz version after security change"
                );
            },
            Err(e) => {
                // Log error but don't fail the request - middleware will fallback to DB
                tracing::error!(
                    user_id = %user_id,
                    tenant_id = %tenant_id,
                    operation = operation,
                    error = %e,
                    "Failed to bump user authz version (middleware will fallback to DB)"
                );
            },
        }
    } else {
        tracing::debug!(
            user_id = %user_id,
            tenant_id = %tenant_id,
            operation = operation,
            "Skipping user authz version bump (Redis not configured)"
        );
    }
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    operation_id = "user_health_check",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResp),
    )
)]
pub async fn health_check() -> Json<HealthResp> {
    Json(HealthResp {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: Utc::now(),
    })
}

/// Register a new user
///
/// ## Tenant Bootstrap Behavior
///
/// The registration endpoint implements automatic role assignment based on tenant state:
///
/// - **New Tenant**: If the `tenant_name` corresponds to a tenant that doesn't exist,
///   a new tenant is created and the registering user becomes the **owner** with full
///   tenant management privileges.
///
/// - **Existing Tenant**: If the `tenant_name` matches an existing tenant (by slug),
///   the user joins that tenant with the default **user** role.
///
/// ## Role Assignment (Option D - Single Role Per User)
///
/// | Scenario | Assigned Role | Description |
/// |----------|---------------|-------------|
/// | New Tenant | `owner` | Full tenant control, can manage billing, settings, users |
/// | Existing Tenant | `user` | Standard access, can view resources per Casbin policies |
///
/// ## Casbin Integration
///
/// Upon successful registration, a Casbin grouping policy is automatically created:
/// `(user_id, role, tenant_id)` - This ensures the user's role policies are enforced.
///
/// ## Email Verification Required
///
/// Registration does NOT return authentication tokens. Users must verify their email
/// before they can log in. A verification email is sent automatically.
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "auth",
    operation_id = "user_register",
    request_body = RegisterReq,
    responses(
        (status = 201, description = "User registered successfully. Email verification required before login.", body = RegisterResp),
        (status = 400, description = "Invalid request (validation error)", body = ErrorResp),
        (status = 409, description = "User already exists in the tenant", body = ErrorResp),
    )
)]
pub async fn register<S: AuthService>(
    Extension(state): Extension<AppState<S>>,
    client_info: crate::extractors::ClientInfo,
    Json(payload): Json<RegisterReq>,
) -> Result<impl IntoResponse, AppError> {
    // Validate request
    use validator::Validate;
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let resp = state
        .auth_service
        .register(payload, client_info.ip_address, client_info.user_agent)
        .await?;

    // Get user info for Casbin setup - need to fetch role from database
    // since RegisterResp doesn't include role info
    let user_id_str = resp.user_id.to_string();
    let tenant_id_str = resp.tenant_id.to_string();

    // Fetch user to get role for Casbin
    // Use auth_service to get user info (it has the role)
    let user_info = state
        .auth_service
        .get_user(resp.user_id, resp.tenant_id)
        .await?;
    let role = &user_info.role;

    // If this is a new tenant (user is owner), copy default policies to the new tenant
    // This ensures the new tenant has the same permission structure as default_tenant
    if role == "owner" {
        if let Err(e) =
            copy_policies_for_tenant(&state.enforcer, "default_tenant", &tenant_id_str).await
        {
            // Policies not copied - user won't have proper permissions.
            // Log loudly so this can be detected and remediated.
            tracing::error!(
                user_id = %user_id_str,
                tenant_id = %tenant_id_str,
                error = %e,
                "Failed to copy Casbin policies for new tenant; tenant has no policies"
            );
        }
    }

    // Add grouping: (user_id, role, tenant_id)
    // This ensures Casbin policies for the role apply to this user.
    // Log error but don't fail registration - Casbin grouping can be fixed later via admin APIs.
    if let Err(e) = add_role_for_user(&state.enforcer, &user_id_str, role, &tenant_id_str).await {
        // User is created but Casbin grouping failed.
        // Log loudly so partial-provisioning can be detected and remediated.
        tracing::error!(
            user_id = %user_id_str,
            tenant_id = %tenant_id_str,
            role = %role,
            error = %e,
            "Failed to add Casbin grouping for registered user; user is partially provisioned"
        );
    }

    // Send verification email
    // Log error but don't fail registration - user can request resend later
    if let Some(ref verification_service) = state.email_verification_service {
        if let Err(e) = verification_service
            .send_verification_email(resp.user_id, resp.tenant_id, &resp.email)
            .await
        {
            tracing::error!(
                user_id = %user_id_str,
                email = %resp.email,
                error = %e,
                "Failed to send verification email; user can request resend"
            );
        }
    } else {
        tracing::warn!(
            user_id = %user_id_str,
            "Email verification service not configured; skipping verification email"
        );
    }

    // No tokens returned - user must verify email before login
    Ok((StatusCode::CREATED, Json(resp)))
}

/// Check tenant slug availability
///
/// This is a public endpoint that allows checking if a tenant slug is available
/// before registration. This helps users understand if they will be creating a
/// new tenant (as owner) or joining an existing one.
#[utoipa::path(
    get,
    path = "/api/v1/auth/check-tenant-slug",
    tag = "auth",
    operation_id = "check_tenant_slug",
    params(
        ("slug" = String, Query, description = "The tenant slug to check availability for"),
    ),
    responses(
        (status = 200, description = "Slug availability status", body = CheckTenantSlugResp),
        (status = 400, description = "Invalid slug format", body = ErrorResp),
    )
)]
pub async fn check_tenant_slug<S: AuthService>(
    Extension(state): Extension<AppState<S>>,
    Query(query): Query<CheckTenantSlugQuery>,
) -> Result<Json<CheckTenantSlugResp>, AppError> {
    // Normalize the slug (lowercase, replace spaces with hyphens)
    let slug = query
        .slug
        .trim()
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    if slug.is_empty() {
        return Err(AppError::ValidationError("Slug cannot be empty".to_string()));
    }

    // Check if tenant repository is available
    let tenant_repo = state
        .tenant_repo
        .as_ref()
        .ok_or_else(|| AppError::InternalError("Tenant repository not configured".to_string()))?;

    // Check if tenant with this slug exists
    let existing_tenant = tenant_repo.find_by_slug(&slug).await?;

    let resp = match existing_tenant {
        Some(tenant) => CheckTenantSlugResp {
            slug,
            available: false,
            existing_tenant_name: Some(tenant.name),
        },
        None => CheckTenantSlugResp {
            slug,
            available: true,
            existing_tenant_name: None,
        },
    };

    Ok(Json(resp))
}

/// Login user
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "auth",
    operation_id = "user_login",
    request_body = LoginReq,
    responses(
        (status = 200, description = "Login successful", body = AuthResp),
        (status = 401, description = "Invalid credentials", body = ErrorResp),
        (status = 429, description = "Too many login attempts", body = ErrorResp),
    )
)]
pub async fn login<S: AuthService>(
    Extension(state): Extension<AppState<S>>,
    client_info: crate::extractors::ClientInfo,
    req_headers: axum::http::HeaderMap,
    Json(payload): Json<LoginReq>,
) -> Result<impl IntoResponse, AppError> {
    // Validate request
    use validator::Validate;
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Determine tenant from headers or host
    // Priority:
    // 1. X-Tenant-ID header (for API clients/testing)
    // 2. Host header (for browser clients)
    let tenant_identifier = if let Some(tenant_id) = req_headers.get("X-Tenant-ID") {
        tenant_id.to_str().ok().map(|s| s.to_string())
    } else if let Some(host) = req_headers.get("Host") {
        // Use .ok() to handle invalid headers instead of unwrap_or("") which masks errors
        host.to_str().ok().and_then(|host_str| {
            // Simple subdomain extraction (naive)
            // host: tenant.domain.com -> tenant
            // host: localhost:8000 -> None (or default?)
            if host_str.contains("localhost") || host_str.contains("127.0.0.1") {
                // For local development without subdomain, require header
                None
            } else {
                host_str.split('.').next().map(|s| s.to_string())
            }
        })
    } else {
        None
    };

    let resp = state
        .auth_service
        .login(payload, tenant_identifier, client_info.ip_address, client_info.user_agent)
        .await?;

    // Set httpOnly cookies for authentication tokens
    let cookie_config = CookieConfig::new(&state.config);
    let mut headers = HeaderMap::new();
    set_auth_cookies(&mut headers, &resp.access_token, &resp.refresh_token, &cookie_config)
        .map_err(|e| AppError::InternalError(format!("Failed to set auth cookies: {}", e)))?;

    Ok((StatusCode::OK, headers, Json(resp)))
}

/// Refresh access token
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    tag = "auth",
    operation_id = "user_refresh_token",
    request_body = RefreshReq,
    responses(
        (status = 200, description = "Token refreshed", body = AuthResp),
        (status = 401, description = "Invalid refresh token", body = ErrorResp),
    )
)]
pub async fn refresh_token<S: AuthService>(
    Extension(state): Extension<AppState<S>>,
    client_info: crate::extractors::ClientInfo,
    req_headers: axum::http::HeaderMap,
    payload: Option<Json<OptionalRefreshReq>>,
) -> Result<impl IntoResponse, AppError> {
    // Get refresh token from cookie or request body
    // Priority: 1. httpOnly cookie (secure), 2. Request body (backwards compatibility)
    let refresh_token_value = if let Some(cookie_token) =
        get_cookie_value(&req_headers, "refresh_token")
    {
        cookie_token
    } else if let Some(Json(body)) = payload {
        // Check if body has a refresh token
        if let Some(token) = body.refresh_token {
            if token.is_empty() {
                return Err(AppError::ValidationError("refresh_token cannot be empty".to_string()));
            }
            token
        } else {
            return Err(AppError::Unauthorized("No refresh token provided".to_string()));
        }
    } else {
        return Err(AppError::Unauthorized("No refresh token provided".to_string()));
    };

    let refresh_req = RefreshReq {
        refresh_token: refresh_token_value,
    };

    let resp = state
        .auth_service
        .refresh_token(refresh_req, client_info.ip_address, client_info.user_agent)
        .await?;

    // Set httpOnly cookies for new authentication tokens
    let cookie_config = CookieConfig::new(&state.config);
    let mut headers = HeaderMap::new();
    set_auth_cookies(&mut headers, &resp.access_token, &resp.refresh_token, &cookie_config)
        .map_err(|e| AppError::InternalError(format!("Failed to set auth cookies: {}", e)))?;

    Ok((StatusCode::OK, headers, Json(resp)))
}

/// Logout user by revoking refresh token session
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "auth",
    operation_id = "user_logout",
    request_body = OptionalRefreshReq,
    responses(
        (status = 200, description = "Logout successful"),
        (status = 401, description = "Invalid refresh token", body = ErrorResp),
    )
)]
pub async fn logout<S: AuthService>(
    Extension(state): Extension<AppState<S>>,
    req_headers: axum::http::HeaderMap,
    payload: Option<Json<OptionalRefreshReq>>,
) -> Result<impl IntoResponse, AppError> {
    // Get refresh token from cookie or request body
    // Priority: 1. httpOnly cookie (secure), 2. Request body (backwards compatibility)
    let refresh_token_value =
        if let Some(cookie_token) = get_cookie_value(&req_headers, "refresh_token") {
            Some(cookie_token)
        } else if let Some(Json(body)) = payload {
            // Check if body has a refresh token
            body.refresh_token.filter(|t| !t.is_empty())
        } else {
            None
        };

    // If we have a refresh token, revoke it
    if let Some(token) = refresh_token_value {
        state.auth_service.logout(&token).await?;
    }

    // Always clear httpOnly cookies
    let mut headers = HeaderMap::new();
    clear_auth_cookies(&mut headers, &state.config)
        .map_err(|e| AppError::InternalError(format!("Failed to clear auth cookies: {}", e)))?;

    Ok((StatusCode::OK, headers, ()))
}

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_page_size")]
    pub page_size: i32,
    /// Filter by user role (optional)
    pub role: Option<String>,
    /// Filter by user status (optional)
    pub status: Option<String>,
}

fn default_page() -> i32 {
    1
}
fn default_page_size() -> i32 {
    20
}

/// List users (protected endpoint - requires authentication)
#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "users",
    operation_id = "user_list_users",
    params(
        ("page" = Option<i32>, Query, description = "Page number (default: 1)"),
        ("page_size" = Option<i32>, Query, description = "Page size (default: 20)"),
        ("role" = Option<String>, Query, description = "Filter by user role (e.g., admin, manager, user)"),
        ("status" = Option<String>, Query, description = "Filter by user status (e.g., active, inactive, suspended)"),
    ),
    responses(
        (status = 200, description = "List of users", body = UserListResp),
        (status = 401, description = "Unauthorized", body = ErrorResp),
        (status = 403, description = "Forbidden", body = ErrorResp),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_users<S: AuthService>(
    auth_user: AuthUser,
    Extension(state): Extension<AppState<S>>,
    Query(query): Query<ListUsersQuery>,
) -> Result<Json<UserListResp>, AppError> {
    // Extract tenant_id from authenticated user
    let tenant_id = auth_user.tenant_id;

    let mut resp = state
        .auth_service
        .list_users(tenant_id, query.page, query.page_size, query.role, query.status)
        .await?;

    // Populate roles from Casbin for each user
    let enforcer = state.enforcer.read().await;
    let tenant_id_str = tenant_id.to_string();

    for user in &mut resp.users {
        let user_roles = enforcer.get_filtered_grouping_policy(0, vec![user.id.to_string()]);
        user.roles = user_roles
            .into_iter()
            .filter(|g| g.len() >= 3 && g[2] == tenant_id_str)
            .map(|g| g[1].clone())
            .collect();
    }

    drop(enforcer);

    Ok(Json(resp))
}

/// Get user by ID (admin only)
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}",
    tag = "users",
    operation_id = "user_get_user",
    params(
        ("user_id" = uuid::Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "User details", body = UserInfo),
        (status = 401, description = "Unauthorized", body = ErrorResp),
        (status = 403, description = "Forbidden - Admin only", body = ErrorResp),
        (status = 404, description = "User not found", body = ErrorResp),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user<S: AuthService>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<UserInfo>, AppError> {
    // Admin can view any user in their tenant
    let tenant_id = admin_user.tenant_id;

    let user_info = state.auth_service.get_user(user_id, tenant_id).await?;

    Ok(Json(user_info))
}

// DTOs for direct policy manipulation (low-level Casbin operations)

#[derive(Debug, Deserialize, utoipa::ToSchema, validator::Validate)]
pub struct CreatePolicyReq {
    #[validate(length(min = 1, max = 255))]
    pub role: String,
    #[validate(length(min = 1, max = 255))]
    pub resource: String,
    #[validate(length(min = 1, max = 255))]
    pub action: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, validator::Validate)]
pub struct DeletePolicyReq {
    #[validate(length(min = 1))]
    pub role: String,
    #[validate(length(min = 1))]
    pub resource: String,
    #[validate(length(min = 1))]
    pub action: String,
}

/// Protected role that cannot be assigned or modified via admin APIs.
/// Owner role is assigned only during tenant bootstrap (registration).
const OWNER_ROLE: &str = "owner";

// Note: AssignRoleReq and RevokeRoleReq DTOs have been moved to admin_dto.rs
// as AssignUserRoleReq for better consistency and enhanced validation.

// Role management handlers

/// Add a policy to a role (admin only)
#[utoipa::path(
    post,
    path = "/api/v1/admin/policies",
    tag = "admin",
    operation_id = "admin_add_policy",
    request_body = CreatePolicyReq,
    responses(
        (status = 200, description = "Policy added successfully"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn add_policy<S: AuthService>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
    Json(payload): Json<CreatePolicyReq>,
) -> Result<StatusCode, AppError> {
    // Validate request
    use validator::Validate;
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Prevent privilege escalation: cannot add policies for owner role
    // Owner policies are seeded during tenant bootstrap only
    if payload.role.to_lowercase() == OWNER_ROLE {
        return Err(AppError::Forbidden(
            "Cannot add policies for 'owner' role. Owner policies are managed by the system only."
                .to_string(),
        ));
    }

    let mut enforcer = state.enforcer.write().await;
    let added = enforcer
        .add_policy(vec![
            payload.role.clone(),
            admin_user.tenant_id.to_string(),
            payload.resource.clone(),
            payload.action.clone(),
        ])
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to add policy: {}", e)))?;
    if added {
        enforcer
            .save_policy()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to save policy: {}", e)))?;
        drop(enforcer);

        // Bump tenant authz version to invalidate existing tokens
        bump_tenant_authz_version(&state, admin_user.tenant_id, "add_policy").await;

        Ok(StatusCode::OK)
    } else {
        Err(AppError::ValidationError("Policy already exists".to_string()))
    }
}

/// Remove a policy from a role (admin only)
#[utoipa::path(
    delete,
    path = "/api/v1/admin/policies",
    tag = "admin",
    operation_id = "admin_remove_policy",
    request_body = DeletePolicyReq,
    responses(
        (status = 200, description = "Policy removed successfully"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn remove_policy<S: AuthService>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
    Json(payload): Json<DeletePolicyReq>,
) -> Result<StatusCode, AppError> {
    // Validate request
    use validator::Validate;
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Prevent privilege escalation: cannot remove policies from owner role
    // Owner policies are managed by the system only
    if payload.role.to_lowercase() == OWNER_ROLE {
        return Err(AppError::Forbidden(
            "Cannot remove policies from 'owner' role. Owner policies are managed by the system only."
                .to_string(),
        ));
    }

    let mut enforcer = state.enforcer.write().await;
    let removed = enforcer
        .remove_policy(vec![
            payload.role.clone(),
            admin_user.tenant_id.to_string(),
            payload.resource.clone(),
            payload.action.clone(),
        ])
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to remove policy: {}", e)))?;

    if removed {
        enforcer
            .save_policy()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to save policy: {}", e)))?;
        drop(enforcer);

        // Bump tenant authz version to invalidate existing tokens
        bump_tenant_authz_version(&state, admin_user.tenant_id, "remove_policy").await;

        Ok(StatusCode::OK)
    } else {
        Err(AppError::ValidationError("Policy does not exist".to_string()))
    }
}

// Note: Legacy assign_role_to_user and revoke_role_from_user handlers have been moved
// to admin_handlers.rs with enhanced validation and error handling.
// The new implementations include:
// - Role existence verification
// - Prevention of removing user's last role
// - Better error messages (404 vs 400)
// See admin_handlers.rs for the current implementations.
