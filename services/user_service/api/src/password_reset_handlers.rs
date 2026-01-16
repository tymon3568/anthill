use axum::{extract::ConnectInfo, http::HeaderMap, Extension, Json};
use shared_error::AppError;
use std::{net::SocketAddr, sync::Arc};
use user_service_core::domains::auth::{
    domain::password_reset_service::PasswordResetService,
    dto::password_reset_dto::{
        ForgotPasswordReq, ForgotPasswordResp, ResetPasswordReq, ResetPasswordResp,
        ValidateResetTokenReq, ValidateResetTokenResp,
    },
};
use uuid::Uuid;
use validator::Validate;

/// Request password reset (forgot-password)
///
/// Sends a password reset email if the email exists in the system.
/// ALWAYS returns success to prevent email enumeration attacks.
/// Rate limited: max 3 requests per hour per email.
#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot-password",
    tag = "auth",
    request_body = ForgotPasswordReq,
    responses(
        (status = 200, description = "Reset email sent (or would be sent if email exists)", body = ForgotPasswordResp),
        (status = 400, description = "Invalid email format", body = String),
    )
)]
pub async fn forgot_password<PRS>(
    Extension(reset_service): Extension<Arc<PRS>>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<ForgotPasswordReq>,
) -> Result<Json<ForgotPasswordResp>, AppError>
where
    PRS: PasswordResetService,
{
    // Validate request
    req.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Extract client info for audit
    let ip_address = Some(addr.ip().to_string());
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Get tenant_id from header (optional for password reset)
    let tenant_id = headers
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok());

    let response = reset_service
        .request_password_reset(&req.email, tenant_id, ip_address, user_agent)
        .await?;

    Ok(Json(response))
}

/// Reset password using token from email
///
/// Validates the reset token, updates the password, and invalidates all sessions.
/// Token is single-use and expires after 1 hour.
#[utoipa::path(
    post,
    path = "/api/v1/auth/reset-password",
    tag = "auth",
    request_body = ResetPasswordReq,
    responses(
        (status = 200, description = "Password reset successfully", body = ResetPasswordResp),
        (status = 400, description = "Invalid token, password mismatch, or weak password", body = String),
        (status = 404, description = "Token not found or expired", body = String),
    )
)]
pub async fn reset_password<PRS>(
    Extension(reset_service): Extension<Arc<PRS>>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<ResetPasswordReq>,
) -> Result<Json<ResetPasswordResp>, AppError>
where
    PRS: PasswordResetService,
{
    // Validate request
    req.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Validate password match
    if req.new_password != req.confirm_password {
        return Err(AppError::ValidationError("Passwords do not match".to_string()));
    }

    // Extract client info for audit
    let ip_address = Some(addr.ip().to_string());
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let response = reset_service
        .reset_password(&req.token, &req.new_password, ip_address, user_agent)
        .await?;

    Ok(Json(response))
}

/// Validate reset token without using it
///
/// Checks if a reset token is valid before showing the password reset form.
/// Does not consume the token.
#[utoipa::path(
    post,
    path = "/api/v1/auth/validate-reset-token",
    tag = "auth",
    request_body = ValidateResetTokenReq,
    responses(
        (status = 200, description = "Token validation result", body = ValidateResetTokenResp),
        (status = 400, description = "Invalid request", body = String),
    )
)]
pub async fn validate_reset_token<PRS>(
    Extension(reset_service): Extension<Arc<PRS>>,
    Json(req): Json<ValidateResetTokenReq>,
) -> Result<Json<ValidateResetTokenResp>, AppError>
where
    PRS: PasswordResetService,
{
    // Validate request
    req.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let response = reset_service.validate_reset_token(&req.token).await?;

    Ok(Json(response))
}
