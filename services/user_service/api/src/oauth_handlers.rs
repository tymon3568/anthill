use axum::{extract::State, Json};
use shared_error::AppError;
use shared_kanidm_client::{KanidmOAuth2Client, PkceState};
use tracing::{debug, warn};
use user_service_core::domains::auth::{
    domain::service::AuthService,
    dto::{
        auth_dto::ErrorResp,
        kanidm_dto::{
            KanidmUserInfo, OAuth2AuthorizeReq, OAuth2AuthorizeResp, OAuth2CallbackReq,
            OAuth2CallbackResp, OAuth2RefreshReq, OAuth2RefreshResp,
        },
    },
};

use crate::handlers::AppState;

/// Initiate OAuth2 authorization flow with Kanidm
///
/// This endpoint generates a PKCE challenge and returns the authorization URL
/// where the user should be redirected to authenticate with Kanidm.
#[utoipa::path(
    post,
    path = "/api/v1/auth/oauth/authorize",
    tag = "oauth",
    operation_id = "oauth_authorize",
    request_body = OAuth2AuthorizeReq,
    responses(
        (status = 200, description = "Authorization URL generated", body = OAuth2AuthorizeResp),
        (status = 500, description = "Server error", body = ErrorResp),
    )
)]
pub async fn oauth_authorize<S: AuthService>(
    State(state): State<AppState<S>>,
    Json(_payload): Json<OAuth2AuthorizeReq>,
) -> Result<Json<OAuth2AuthorizeResp>, AppError> {
    debug!("OAuth2 authorize request received");

    // Generate PKCE state
    let pkce = PkceState::generate();

    // Generate authorization URL
    let auth_url = state
        .kanidm_client
        .authorization_url(&pkce)
        .map_err(|e| AppError::InternalError(format!("Failed to generate auth URL: {}", e)))?;

    debug!("Authorization URL generated: {}", auth_url);

    Ok(Json(OAuth2AuthorizeResp {
        authorization_url: auth_url.to_string(),
        state: pkce.state.clone(),
        code_verifier: pkce.code_verifier,
    }))
}

/// Handle OAuth2 callback from Kanidm
///
/// After user authenticates with Kanidm, they are redirected back with an
/// authorization code. This endpoint exchanges the code for tokens and
/// maps the Kanidm user to a tenant.
#[utoipa::path(
    post,
    path = "/api/v1/auth/oauth/callback",
    tag = "oauth",
    operation_id = "oauth_callback",
    request_body = OAuth2CallbackReq,
    responses(
        (status = 200, description = "Authentication successful", body = OAuth2CallbackResp),
        (status = 400, description = "Invalid request", body = ErrorResp),
        (status = 401, description = "Authentication failed", body = ErrorResp),
        (status = 500, description = "Server error", body = ErrorResp),
    )
)]
pub async fn oauth_callback<S: AuthService>(
    State(state): State<AppState<S>>,
    Json(payload): Json<OAuth2CallbackReq>,
) -> Result<Json<OAuth2CallbackResp>, AppError> {
    debug!("OAuth2 callback received with code");

    // Reconstruct PKCE state
    let pkce = PkceState {
        code_verifier: payload.code_verifier.clone(),
        code_challenge: String::new(), // Not needed for token exchange
        state: payload.state.clone(),
    };

    // Exchange authorization code for tokens
    let token_response = state
        .kanidm_client
        .exchange_code(&payload.code, &pkce)
        .await
        .map_err(|e| {
            warn!("Token exchange failed: {}", e);
            AppError::Unauthorized(format!("Failed to exchange code: {}", e))
        })?;

    debug!("Token exchange successful");

    // Validate and extract claims from JWT
    let claims = state
        .kanidm_client
        .validate_token(&token_response.access_token)
        .await
        .map_err(|e| {
            warn!("Token validation failed: {}", e);
            AppError::InvalidToken
        })?;

    debug!(
        "Token validated - user: {}, email: {:?}",
        claims.sub, claims.email
    );

    // Extract user info from claims
    let user_info = KanidmUserInfo {
        kanidm_user_id: claims.sub.clone(),
        email: claims.email.clone(),
        preferred_username: claims.preferred_username.clone(),
        groups: claims.groups.clone(),
    };

    // TODO: Map Kanidm user to tenant and role
    // For now, return None for tenant (will be implemented in Phase 3.5)
    let tenant_info = None;

    warn!("⚠️ Tenant mapping not implemented yet - user authenticated but no tenant assigned");

    Ok(Json(OAuth2CallbackResp {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: Some(token_response.expires_in as i64),
        user: user_info,
        tenant: tenant_info,
    }))
}

/// Refresh access token using refresh token
#[utoipa::path(
    post,
    path = "/api/v1/auth/oauth/refresh",
    tag = "oauth",
    operation_id = "oauth_refresh",
    request_body = OAuth2RefreshReq,
    responses(
        (status = 200, description = "Token refreshed successfully", body = OAuth2RefreshResp),
        (status = 401, description = "Invalid refresh token", body = ErrorResp),
        (status = 500, description = "Server error", body = ErrorResp),
    )
)]
pub async fn oauth_refresh<S: AuthService>(
    State(state): State<AppState<S>>,
    Json(payload): Json<OAuth2RefreshReq>,
) -> Result<Json<OAuth2RefreshResp>, AppError> {
    debug!("OAuth2 refresh token request received");

    // Refresh access token
    let token_response = state
        .kanidm_client
        .refresh_token(&payload.refresh_token)
        .await
        .map_err(|e| {
            warn!("Token refresh failed: {}", e);
            AppError::Unauthorized(format!("Failed to refresh token: {}", e))
        })?;

    debug!("Token refreshed successfully");

    Ok(Json(OAuth2RefreshResp {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: Some(token_response.expires_in as i64),
    }))
}
