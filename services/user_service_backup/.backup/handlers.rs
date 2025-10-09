use axum::{
    Json,
    http::StatusCode,
};
use uuid::Uuid;
use chrono::Utc;
use crate::models::*;

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
pub async fn register(
    Json(_payload): Json<RegisterReq>,
) -> Result<(StatusCode, Json<AuthResp>), (StatusCode, Json<ErrorResp>)> {
    // Mock response
    Ok((
        StatusCode::CREATED,
        Json(AuthResp {
            access_token: "mock_access_token".to_string(),
            refresh_token: "mock_refresh_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 900,
            user: UserInfo {
                id: Uuid::new_v4(),
                email: "user@example.com".to_string(),
                full_name: "John Doe".to_string(),
                tenant_id: Uuid::new_v4(),
                role: "user".to_string(),
                created_at: Utc::now(),
            },
        }),
    ))
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
pub async fn login(
    Json(_payload): Json<LoginReq>,
) -> Result<Json<AuthResp>, (StatusCode, Json<ErrorResp>)> {
    // Mock response
    Ok(Json(AuthResp {
        access_token: "mock_access_token".to_string(),
        refresh_token: "mock_refresh_token".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: 900,
        user: UserInfo {
            id: Uuid::new_v4(),
            email: "user@example.com".to_string(),
            full_name: "John Doe".to_string(),
            tenant_id: Uuid::new_v4(),
            role: "user".to_string(),
            created_at: Utc::now(),
        },
    }))
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
pub async fn refresh_token(
    Json(_payload): Json<RefreshReq>,
) -> Result<Json<AuthResp>, (StatusCode, Json<ErrorResp>)> {
    // Mock response
    Ok(Json(AuthResp {
        access_token: "new_mock_access_token".to_string(),
        refresh_token: "new_mock_refresh_token".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: 900,
        user: UserInfo {
            id: Uuid::new_v4(),
            email: "user@example.com".to_string(),
            full_name: "John Doe".to_string(),
            tenant_id: Uuid::new_v4(),
            role: "user".to_string(),
            created_at: Utc::now(),
        },
    }))
}

/// List users (protected endpoint)
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
pub async fn list_users() -> Json<UserListResp> {
    // Mock response
    Json(UserListResp {
        users: vec![
            UserInfo {
                id: Uuid::new_v4(),
                email: "user1@example.com".to_string(),
                full_name: "User One".to_string(),
                tenant_id: Uuid::new_v4(),
                role: "admin".to_string(),
                created_at: Utc::now(),
            },
            UserInfo {
                id: Uuid::new_v4(),
                email: "user2@example.com".to_string(),
                full_name: "User Two".to_string(),
                tenant_id: Uuid::new_v4(),
                role: "user".to_string(),
                created_at: Utc::now(),
            },
        ],
        total: 2,
        page: 1,
        page_size: 20,
    })
}
