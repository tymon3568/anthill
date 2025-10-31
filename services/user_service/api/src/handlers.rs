use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use shared_auth::casbin::{CoreApi, MgmtApi};
use shared_auth::enforcer::SharedEnforcer;
use shared_auth::extractors::{AuthUser, JwtSecretProvider, RequireAdmin};
use shared_error::AppError;
use std::sync::Arc;
use user_service_core::domains::auth::domain::service::AuthService;
use user_service_core::domains::auth::dto::auth_dto::{
    AuthResp, ErrorResp, HealthResp, LoginReq, RefreshReq, RegisterReq, UserInfo, UserListResp,
};

/// Application state containing service dependencies
pub struct AppState<S: AuthService> {
    pub auth_service: Arc<S>,
    pub enforcer: SharedEnforcer,
    pub jwt_secret: String,
}

impl<S: AuthService> Clone for AppState<S> {
    fn clone(&self) -> Self {
        Self {
            auth_service: Arc::clone(&self.auth_service),
            enforcer: self.enforcer.clone(),
            jwt_secret: self.jwt_secret.clone(),
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
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "auth",
    operation_id = "user_register",
    request_body = RegisterReq,
    responses(
        (status = 201, description = "User registered successfully", body = AuthResp),
        (status = 400, description = "Invalid request", body = ErrorResp),
        (status = 409, description = "User already exists", body = ErrorResp),
    )
)]
pub async fn register<S: AuthService>(
    State(state): State<AppState<S>>,
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
    State(state): State<AppState<S>>,
    client_info: crate::extractors::ClientInfo,
    Json(payload): Json<LoginReq>,
) -> Result<Json<AuthResp>, AppError> {
    // Validate request
    use validator::Validate;
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let resp = state
        .auth_service
        .login(payload, client_info.ip_address, client_info.user_agent)
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
    State(state): State<AppState<S>>,
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
    State(state): State<AppState<S>>,
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
    State(state): State<AppState<S>>,
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
    State(state): State<AppState<S>>,
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
    State(state): State<AppState<S>>,
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
        enforcer.save_policy().await.map_err(|e| AppError::InternalError(format!("Failed to save policy: {}", e)))?;
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
    State(state): State<AppState<S>>,
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
        enforcer.save_policy().await.map_err(|e| AppError::InternalError(format!("Failed to save policy: {}", e)))?;
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




