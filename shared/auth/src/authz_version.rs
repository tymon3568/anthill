//! AuthZ Version Middleware
//!
//! Global middleware that validates authorization versions in JWT tokens
//! against current versions in the version store (Redis/DB).
//!
//! This enables immediate-effect permission invalidation:
//! - When tenant policies change, bump tenant_v -> all tokens become stale
//! - When user role changes, bump user_v -> that user's tokens become stale

use async_trait::async_trait;
use axum::extract::{Extension, Request};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use http::{header, StatusCode};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Trait for AuthZ version lookups
///
/// This trait allows the middleware to be decoupled from the concrete
/// implementation (Redis/PostgreSQL). The user_service_infra crate
/// provides the actual implementation.
#[async_trait]
pub trait AuthzVersionProvider: Send + Sync {
    /// Get both tenant and user versions concurrently
    async fn get_versions(&self, tenant_id: Uuid, user_id: Uuid) -> Result<(i64, i64), String>;
}

/// State for AuthZ version middleware
#[derive(Clone)]
pub struct AuthzVersionState {
    /// JWT secret for token validation
    pub jwt_secret: String,
    /// Version provider (Redis + DB fallback)
    pub version_provider: Arc<dyn AuthzVersionProvider>,
    /// Whether to enforce version checks (can be disabled for gradual rollout)
    pub enforce: bool,
}

impl AuthzVersionState {
    /// Create new state with version enforcement enabled
    pub fn new(jwt_secret: String, version_provider: Arc<dyn AuthzVersionProvider>) -> Self {
        Self {
            jwt_secret,
            version_provider,
            enforce: true,
        }
    }

    /// Create new state with configurable enforcement
    pub fn new_with_enforcement(
        jwt_secret: String,
        version_provider: Arc<dyn AuthzVersionProvider>,
        enforce: bool,
    ) -> Self {
        Self {
            jwt_secret,
            version_provider,
            enforce,
        }
    }
}

/// AuthZ version validation errors
#[derive(Debug)]
pub enum AuthzVersionError {
    /// Missing Authorization header
    MissingToken,
    /// Invalid or malformed token
    InvalidToken,
    /// Token's tenant version is stale (permissions changed)
    StaleTenantVersion { token_v: i64, current_v: i64 },
    /// Token's user version is stale (role/status changed)
    StaleUserVersion { token_v: i64, current_v: i64 },
    /// Version lookup failed
    VersionLookupFailed(String),
}

impl IntoResponse for AuthzVersionError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AuthzVersionError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "MISSING_TOKEN",
                "Missing authorization token".to_string(),
            ),
            AuthzVersionError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "INVALID_TOKEN",
                "Invalid authorization token".to_string(),
            ),
            AuthzVersionError::StaleTenantVersion { token_v, current_v } => {
                info!(
                    "Rejected stale tenant version: token_v={}, current_v={}",
                    token_v, current_v
                );
                (
                    StatusCode::UNAUTHORIZED,
                    "STALE_TOKEN",
                    "Token is stale due to permission changes. Please re-authenticate.".to_string(),
                )
            },
            AuthzVersionError::StaleUserVersion { token_v, current_v } => {
                info!("Rejected stale user version: token_v={}, current_v={}", token_v, current_v);
                (
                    StatusCode::UNAUTHORIZED,
                    "STALE_TOKEN",
                    "Token is stale due to account changes. Please re-authenticate.".to_string(),
                )
            },
            AuthzVersionError::VersionLookupFailed(ref e) => {
                warn!("Version lookup failed: {}", e);
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "VERSION_CHECK_FAILED",
                    "Authorization version check failed. Please try again.".to_string(),
                )
            },
        };

        let body = serde_json::json!({
            "error": message,
            "code": code,
        });

        (status, axum::Json(body)).into_response()
    }
}

/// AuthZ version validation middleware
///
/// This middleware should be placed BEFORE Casbin middleware in the chain.
/// It validates that the JWT token's authorization versions match the current
/// versions in the store. If they don't match, the request is rejected immediately.
///
/// # Version Semantics
/// - `tenant_v`: Bumped when tenant-wide policies or role definitions change
/// - `user_v`: Bumped when user's role assignment or security state changes
///
/// # Backward Compatibility
/// Tokens without version claims (tenant_v=0, user_v=0) skip version validation
/// to support gradual rollout and existing tokens.
///
/// # Usage
/// ```no_run
/// use axum::{Router, routing::get, middleware};
/// use shared_auth::authz_version_middleware;
///
/// let app = Router::new()
///     .route("/api/v1/resource", get(handler))
///     .layer(middleware::from_fn_with_state(authz_state, authz_version_middleware));
/// ```
pub async fn authz_version_middleware(
    Extension(state): Extension<AuthzVersionState>,
    request: Request,
    next: Next,
) -> Result<Response, AuthzVersionError> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthzVersionError::MissingToken)?;

    // Extract token from "Bearer <token>"
    let token = auth_header
        .trim()
        .split_once(' ')
        .and_then(|(scheme, token)| scheme.eq_ignore_ascii_case("Bearer").then_some(token))
        .filter(|token| !token.is_empty())
        .ok_or(AuthzVersionError::InvalidToken)?;

    // Decode JWT (validation including expiry is done by jsonwebtoken)
    let claims = shared_jwt::decode_jwt(token, &state.jwt_secret)
        .map_err(|_| AuthzVersionError::InvalidToken)?;

    // Skip version check for legacy tokens (without version claims)
    // This allows gradual rollout without breaking existing sessions
    if !claims.has_authz_versions() {
        debug!(
            "Skipping version check for legacy token: user={}, tenant={}",
            claims.sub, claims.tenant_id
        );
        return Ok(next.run(request).await);
    }

    // Skip version check if enforcement is disabled
    if !state.enforce {
        debug!("AuthZ version enforcement disabled, skipping check");
        return Ok(next.run(request).await);
    }

    // Get current versions from store
    let (current_tenant_v, current_user_v) = state
        .version_provider
        .get_versions(claims.tenant_id, claims.sub)
        .await
        .map_err(AuthzVersionError::VersionLookupFailed)?;

    // Check tenant version
    if claims.tenant_v < current_tenant_v {
        return Err(AuthzVersionError::StaleTenantVersion {
            token_v: claims.tenant_v,
            current_v: current_tenant_v,
        });
    }

    // Check user version
    if claims.user_v < current_user_v {
        return Err(AuthzVersionError::StaleUserVersion {
            token_v: claims.user_v,
            current_v: current_user_v,
        });
    }

    debug!(
        "AuthZ version check passed: user={}, tenant_v={}/{}, user_v={}/{}",
        claims.sub, claims.tenant_v, current_tenant_v, claims.user_v, current_user_v
    );

    // Version check passed, continue to next middleware
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authz_version_error_response_codes() {
        let err = AuthzVersionError::MissingToken;
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let err = AuthzVersionError::InvalidToken;
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let err = AuthzVersionError::StaleTenantVersion {
            token_v: 1,
            current_v: 2,
        };
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let err = AuthzVersionError::StaleUserVersion {
            token_v: 1,
            current_v: 2,
        };
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let err = AuthzVersionError::VersionLookupFailed("test error".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
