use axum::{extract::ConnectInfo, http::HeaderMap, Extension, Json};
use shared_error::AppError;
use std::{net::SocketAddr, sync::Arc};
use user_service_core::domains::auth::{
    domain::email_verification_service::EmailVerificationService,
    dto::email_verification_dto::{
        ResendVerificationReq, ResendVerificationResp, VerifyEmailReq, VerifyEmailResp,
    },
};
use uuid::Uuid;
use validator::Validate;

/// Verify email using token from verification link
///
/// Validates the verification token and marks the user's email as verified.
/// Token is single-use and expires after 24 hours.
#[utoipa::path(
    post,
    path = "/api/v1/auth/verify-email",
    tag = "auth",
    request_body = VerifyEmailReq,
    responses(
        (status = 200, description = "Email verified successfully", body = VerifyEmailResp),
        (status = 400, description = "Invalid token or already verified", body = String),
        (status = 404, description = "Token not found or expired", body = String),
    )
)]
pub async fn verify_email<EVS>(
    Extension(verification_service): Extension<Arc<EVS>>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<VerifyEmailReq>,
) -> Result<Json<VerifyEmailResp>, AppError>
where
    EVS: EmailVerificationService,
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

    let response = verification_service
        .verify_email(&req.token, ip_address, user_agent)
        .await?;

    Ok(Json(response))
}

/// Resend verification email
///
/// Sends a new verification email to the user. Rate-limited to prevent spam.
/// Maximum 3 resends per hour.
#[utoipa::path(
    post,
    path = "/api/v1/auth/resend-verification",
    tag = "auth",
    request_body = ResendVerificationReq,
    responses(
        (status = 200, description = "Verification email sent or rate limit info", body = ResendVerificationResp),
        (status = 400, description = "Email already verified or invalid", body = String),
        (status = 404, description = "User not found", body = String),
    )
)]
pub async fn resend_verification<EVS>(
    Extension(verification_service): Extension<Arc<EVS>>,
    headers: HeaderMap,
    Json(req): Json<ResendVerificationReq>,
) -> Result<Json<ResendVerificationResp>, AppError>
where
    EVS: EmailVerificationService,
{
    // Validate request
    req.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Get tenant_id from header (required for multi-tenant context)
    let tenant_id = headers
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok());

    let response = verification_service
        .resend_verification_email(&req.email, tenant_id)
        .await?;

    Ok(Json(response))
}
