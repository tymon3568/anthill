use axum::{
    Json,
    http::StatusCode,
    extract::State,
};
use chrono::Utc;
use user_service_core::domains::auth::{
    domain::service::AuthService,
    dto::auth_dto::*,
};
use shared_error::AppError;
use std::sync::Arc;

/// Application state containing service dependencies
pub struct AppState<S: AuthService> {
    pub auth_service: Arc<S>,
}

impl<S: AuthService> Clone for AppState<S> {
    fn clone(&self) -> Self {
        Self {
            auth_service: Arc::clone(&self.auth_service),
        }
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
    Json(payload): Json<RegisterReq>,
) -> Result<(StatusCode, Json<AuthResp>), AppError> {
    // Validate request
    use validator::Validate;
    payload.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    
    let resp = state.auth_service.register(payload).await?;
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
    Json(payload): Json<LoginReq>,
) -> Result<Json<AuthResp>, AppError> {
    // Validate request
    use validator::Validate;
    payload.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    
    let resp = state.auth_service.login(payload).await?;
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
    Json(payload): Json<RefreshReq>,
) -> Result<Json<AuthResp>, AppError> {
    // Validate request
    use validator::Validate;
    payload.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    
    let resp = state.auth_service.refresh_token(payload).await?;
    Ok(Json(resp))
}

/// List users (protected endpoint - TODO: add auth middleware)
#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "users",
    operation_id = "user_list_users",
    params(
        ("page" = Option<i32>, Query, description = "Page number (default: 1)"),
        ("page_size" = Option<i32>, Query, description = "Page size (default: 20)"),
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
    State(_state): State<AppState<S>>,
) -> Result<Json<UserListResp>, AppError> {
    // TODO: Extract tenant_id from JWT token via auth middleware
    // For now, return empty list
    Ok(Json(UserListResp {
        users: vec![],
        total: 0,
        page: 1,
        page_size: 20,
    }))
}
