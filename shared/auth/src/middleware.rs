use crate::enforcer::SharedEnforcer;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use http::{header, StatusCode};
use tracing::{debug, warn};
use casbin::CoreApi;

#[derive(Clone)]
pub struct AuthzState {
    pub enforcer: SharedEnforcer,
    pub jwt_secret: String,
}

/// Casbin authorization middleware
///
/// This middleware:
/// 1. Extracts JWT from Authorization header
/// 2. Validates JWT and extracts claims (user_id, tenant_id, role)
/// 3. Checks permissions using Casbin enforcer
/// 4. Returns 403 Forbidden if permission denied
///
/// # Usage
/// ```no_run
/// use axum::{Router, routing::get, middleware};
/// use shared_auth::{create_enforcer, casbin_middleware};
///
/// async fn handler() -> &'static str {
///     "Hello, authorized user!"
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let enforcer = create_enforcer("postgres://localhost/db", None).await.unwrap();
///
///     let app = Router::new()
///         .route("/api/v1/products", get(handler))
///         .layer(middleware::from_fn_with_state(enforcer, casbin_middleware));
/// }
/// ```
pub async fn casbin_middleware(
    State(state): State<AuthzState>,
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthError::MissingToken)?;

    // Extract token from "Bearer <token>"
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidToken)?;

    // Decode and validate JWT
    let claims =
        shared_jwt::decode_jwt(token, &state.jwt_secret).map_err(|_| AuthError::InvalidToken)?;

    debug!(
        "JWT validated: user_id={}, tenant_id={}, role={}",
        claims.sub, claims.tenant_id, claims.role
    );

    // Get resource and action from request
    let resource = request.uri().path();
    let action = request.method().as_str();

    // Check permission with Casbin
    let allowed = check_permission(
        &state.enforcer,
        &claims.sub.to_string(),
        &claims.tenant_id.to_string(),
        resource,
        action,
    )
    .await?;

    if !allowed {
        warn!(
            "Permission denied: user={}, tenant={}, resource={}, action={}",
            claims.sub, claims.tenant_id, resource, action
        );
        return Err(AuthError::PermissionDenied);
    }

    debug!(
        "Permission granted: user={}, resource={}, action={}",
        claims.sub, resource, action
    );

    // Permission granted, continue to next middleware/handler
    Ok(next.run(request).await)
}

/// Check permission using Casbin enforcer
pub async fn check_permission(
    enforcer: &SharedEnforcer,
    user_id: &str,
    tenant_id: &str,
    resource: &str,
    action: &str,
) -> Result<bool, AuthError> {
    let e = enforcer.read().await;

    // Try to enforce with user_id first
    let allowed = e
        .enforce((user_id, tenant_id, resource, action))
        .map_err(|e| AuthError::CasbinError(e.to_string()))?;

    Ok(allowed)
}

/// Authentication/Authorization errors
#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    PermissionDenied,
    CasbinError(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid authorization token"),
            AuthError::PermissionDenied => (StatusCode::FORBIDDEN, "Permission denied"),
            AuthError::CasbinError(ref e) => {
                warn!("Casbin error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Authorization check failed",
                )
            }
        };

        let body = serde_json::json!({
            "error": message,
        });

        (status, axum::Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auth_error_responses() {
        let err = AuthError::MissingToken;
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let err = AuthError::PermissionDenied;
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
