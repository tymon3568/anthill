use axum::{extract::Extension, Json};
use shared_error::AppError;
use shared_kanidm_client::{KanidmOAuth2Client, PkceState};
use tracing::{debug, warn};
use user_service_core::domains::auth::{
    domain::service::AuthService,
    dto::{
        auth_dto::ErrorResp,
        kanidm_dto::{
            KanidmUserInfo, OAuth2AuthorizeReq, OAuth2AuthorizeResp, OAuth2CallbackReq,
            OAuth2CallbackResp, OAuth2RefreshReq, OAuth2RefreshResp, TenantInfo,
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
    Extension(state): Extension<AppState<S>>,
    Json(payload): Json<OAuth2AuthorizeReq>,
) -> Result<Json<OAuth2AuthorizeResp>, AppError> {
    debug!("OAuth2 authorize request received");

    // Generate PKCE state, using provided state if available
    let mut pkce = PkceState::generate();
    if let Some(custom_state) = payload.state {
        pkce.state = custom_state;
    }

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
    Extension(state): Extension<AppState<S>>,
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

    debug!("Token validated - user: {}, email: {:?}", claims.sub, claims.email);

    // Extract user info from claims
    let user_info = KanidmUserInfo {
        kanidm_user_id: claims.sub.clone(),
        email: claims.email.clone(),
        preferred_username: claims.preferred_username.clone(),
        groups: claims.groups.clone(),
    };

    // Map Kanidm user to tenant and role
    let tenant_info = map_tenant_from_groups(&state, &claims.groups).await?;

    // Upsert user in database
    if let Some((ref tenant, ref role)) = tenant_info {
        debug!("Mapping user to tenant: {} with role: {}", tenant.name, role);

        // Get user repository from state
        if let Some(user_repo) = &state.user_repo {
            let (user, is_new) = user_repo
                .upsert_from_kanidm(
                    &claims.sub,
                    claims.email.as_deref(),
                    claims.preferred_username.as_deref(),
                    tenant.tenant_id,
                )
                .await?;

            if is_new {
                debug!("Created new user from Kanidm authentication: {}", user.user_id);
            } else {
                debug!("Updated existing user from Kanidm: {}", user.user_id);
            }
        } else {
            warn!("User repository not available - skipping user sync");
        }
    }

    Ok(Json(OAuth2CallbackResp {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: Some(token_response.expires_in as i64),
        user: user_info,
        tenant: tenant_info.map(|(tenant, role)| TenantInfo {
            tenant_id: tenant.tenant_id.to_string(),
            name: tenant.name,
            slug: tenant.slug,
            role,
        }),
    }))
}

/// Map Kanidm groups to tenant and role
///
/// Expects groups in format: tenant_{slug}_admins, tenant_{slug}_users
/// Returns first matching tenant with role
async fn map_tenant_from_groups<S: AuthService>(
    state: &AppState<S>,
    groups: &[String],
) -> Result<Option<(user_service_core::domains::auth::domain::model::Tenant, String)>, AppError> {
    // Filter tenant-related groups
    let tenant_groups: Vec<&String> = groups.iter().filter(|g| g.starts_with("tenant_")).collect();

    if tenant_groups.is_empty() {
        warn!("User has no tenant groups in Kanidm");
        return Ok(None);
    }

    // Get tenant repository from state
    let tenant_repo = match &state.tenant_repo {
        Some(repo) => repo,
        None => {
            warn!("Tenant repository not available in AppState");
            return Ok(None);
        },
    };

    // Try to find matching tenant for each group
    for group_name in &tenant_groups {
        debug!("Checking group: {}", group_name);

        if let Some((tenant, role)) = tenant_repo.find_by_kanidm_group(group_name).await? {
            debug!("Found tenant: {} with role: {}", tenant.name, role);
            return Ok(Some((tenant, role)));
        }
    }

    warn!("No matching tenant found for groups: {:?}", tenant_groups);
    Ok(None)
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
    Extension(state): Extension<AppState<S>>,
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
