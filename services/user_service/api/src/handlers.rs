use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use shared_auth::casbin::{CoreApi, MgmtApi};
use shared_auth::enforcer::{add_role_for_user, SharedEnforcer};
use shared_auth::extractors::{AuthUser, JwtSecretProvider, RequireAdmin};
use shared_error::AppError;
use std::sync::Arc;
use user_service_core::domains::auth::domain::repository::{TenantRepository, UserRepository};
use user_service_core::domains::auth::domain::service::AuthService;
use user_service_core::domains::auth::dto::auth_dto::{
    AuthResp, ErrorResp, HealthResp, LoginReq, RefreshReq, RegisterReq, UserInfo, UserListResp,
};

/// Application state containing service dependencies
pub struct AppState<S: AuthService> {
    pub auth_service: Arc<S>,
    pub enforcer: SharedEnforcer,
    pub jwt_secret: String,
    // Repositories for user/tenant management
    pub user_repo: Option<Arc<dyn UserRepository>>,
    pub tenant_repo: Option<Arc<dyn TenantRepository>>,
}

impl<S: AuthService> Clone for AppState<S> {
    fn clone(&self) -> Self {
        Self {
            auth_service: Arc::clone(&self.auth_service),
            enforcer: self.enforcer.clone(),
            jwt_secret: self.jwt_secret.clone(),
            user_repo: self.user_repo.clone(),
            tenant_repo: self.tenant_repo.clone(),
        }
    }
}

impl<S: AuthService> JwtSecretProvider for AppState<S> {
    fn get_jwt_secret(&self) -> &str {
        &self.jwt_secret
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
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "auth",
    operation_id = "user_register",
    request_body = RegisterReq,
    responses(
        (status = 201, description = "User registered successfully. Role is 'owner' for new tenant, 'user' for existing tenant.", body = AuthResp),
        (status = 400, description = "Invalid request (validation error)", body = ErrorResp),
        (status = 409, description = "User already exists in the tenant", body = ErrorResp),
    )
)]
pub async fn register<S: AuthService>(
    Extension(state): Extension<AppState<S>>,
    client_info: crate::extractors::ClientInfo,
    Json(payload): Json<RegisterReq>,
) -> Result<(StatusCode, Json<AuthResp>), AppError> {
    // Validate request
    use validator::Validate;
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let resp = state
        .auth_service
        .register(payload, client_info.ip_address, client_info.user_agent)
        .await?;

    // Add Casbin grouping policy for the new user
    // The user's role (owner/user) is determined by the service based on:
    // - 'owner' if user created a new tenant
    // - 'user' if user joined an existing tenant
    let user_id_str = resp.user.id.to_string();
    let tenant_id_str = resp.user.tenant_id.to_string();
    let role = &resp.user.role;

    // Add grouping: (user_id, role, tenant_id)
    // This ensures Casbin policies for the role apply to this user
    if let Err(e) = add_role_for_user(&state.enforcer, &user_id_str, role, &tenant_id_str).await {
        // Log error but don't fail registration - Casbin grouping can be fixed later
        tracing::error!(
            user_id = %user_id_str,
            tenant_id = %tenant_id_str,
            role = %role,
            error = %e,
            "Failed to add Casbin grouping for registered user"
        );
    }

    Ok((StatusCode::CREATED, Json(resp)))
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
    headers: axum::http::HeaderMap,
    Json(payload): Json<LoginReq>,
) -> Result<Json<AuthResp>, AppError> {
    // Validate request
    use validator::Validate;
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Determine tenant from headers or host
    // Priority:
    // 1. X-Tenant-ID header (for API clients/testing)
    // 2. Host header (for browser clients)
    let tenant_identifier = if let Some(tenant_id) = headers.get("X-Tenant-ID") {
        tenant_id.to_str().ok().map(|s| s.to_string())
    } else if let Some(host) = headers.get("Host") {
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
    Ok(Json(resp))
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
    Json(payload): Json<RefreshReq>,
) -> Result<Json<AuthResp>, AppError> {
    // Validate request
    use validator::Validate;
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let resp = state
        .auth_service
        .refresh_token(payload, client_info.ip_address, client_info.user_agent)
        .await?;
    Ok(Json(resp))
}

/// Logout user by revoking refresh token session
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "auth",
    operation_id = "user_logout",
    request_body = RefreshReq,
    responses(
        (status = 200, description = "Logout successful"),
        (status = 401, description = "Invalid refresh token", body = ErrorResp),
    )
)]
pub async fn logout<S: AuthService>(
    Extension(state): Extension<AppState<S>>,
    Json(payload): Json<RefreshReq>,
) -> Result<StatusCode, AppError> {
    // Validate request
    use validator::Validate;
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    state.auth_service.logout(&payload.refresh_token).await?;
    Ok(StatusCode::OK)
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

    let resp = state
        .auth_service
        .list_users(tenant_id, query.page, query.page_size, query.role, query.status)
        .await?;

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
