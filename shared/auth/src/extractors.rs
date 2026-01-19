use axum::{
    extract::{Extension, FromRequestParts},
    http::{header, request::Parts, StatusCode},
};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use tracing::{debug, warn};
use uuid::Uuid;

use casbin::CoreApi;
use shared_jwt::Claims;

use crate::{enforcer::SharedEnforcer, middleware::AuthzState};

pub trait JwtSecretProvider {
    fn get_jwt_secret(&self) -> &str;
}

/// Authenticated user information extracted from JWT
///
/// This extractor validates the JWT token and extracts user information.
/// It does NOT check permissions - use `RequirePermission` or `RequireRole` for that.
///
/// # Usage
/// ```no_run
/// use axum::{routing::get, Router};
/// use shared_auth::AuthUser;
///
/// async fn handler(user: AuthUser) -> String {
///     format!("Hello, user {}!", user.user_id)
/// }
///
/// let app = Router::new().route("/profile", get(handler));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub role: String,
    /// User email from JWT claims
    pub email: Option<String>,
}

impl AuthUser {
    /// Create AuthUser from JWT claims
    pub fn from_claims(claims: Claims) -> Self {
        Self {
            user_id: claims.sub,
            tenant_id: claims.tenant_id,
            role: claims.role,
            email: None,
        }
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.role == role
    }

    /// Check if user is admin (includes owner and super_admin)
    ///
    /// Per AUTHORIZATION_RBAC_STRATEGY.md, owner has all admin privileges plus
    /// tenant-level management capabilities. Therefore owner should pass admin checks.
    pub fn is_admin(&self) -> bool {
        self.role == "admin" || self.role == "super_admin" || self.role == "owner"
    }

    /// Check if user is the tenant owner
    ///
    /// Owner is a special protected role assigned only during tenant bootstrap.
    /// Owners have full control over their tenant including billing and settings.
    pub fn is_owner(&self) -> bool {
        self.role == "owner"
    }
}

/// Validate JWT token and return AuthUser
///
/// Only accepts "access" tokens - refresh tokens are rejected to prevent
/// using long-lived refresh tokens for API authentication.
fn validate_token(token: &str, authz_state: &AuthzState) -> Result<AuthUser, StatusCode> {
    match shared_jwt::decode_jwt(token, &authz_state.jwt_secret) {
        Ok(claims) => {
            // Security: Ensure only access tokens are accepted for API authentication
            // Refresh tokens should only be used at the /auth/refresh endpoint
            if claims.token_type != "access" {
                warn!(
                    "JWT validation failed for user {}: expected 'access' token, got '{}'",
                    claims.sub, claims.token_type
                );
                return Err(StatusCode::UNAUTHORIZED);
            }
            debug!("Validated JWT for user {}", claims.sub);
            Ok(AuthUser::from_claims(claims))
        },
        Err(e) => {
            warn!("JWT validation failed: {}", e);
            Err(StatusCode::UNAUTHORIZED)
        },
    }
}

impl FromRequestParts<()> for AuthUser {
    type Rejection = StatusCode;

    /// Extracts an AuthUser from request parts by validating a Bearer JWT in the `Authorization` header.
    ///
    /// Returns `Ok(AuthUser)` when a valid Bearer token is present and validates successfully.
    /// Returns `StatusCode::UNAUTHORIZED` when the header is missing, not a Bearer token,
    /// or the JWT fails to validate.
    async fn from_request_parts(parts: &mut Parts, _state: &()) -> Result<Self, Self::Rejection> {
        // Extract AuthzState from extensions
        let Extension(authz_state): Extension<AuthzState> =
            Extension::from_request_parts(parts, &())
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Extract Authorization header
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // Extract token from "Bearer <token>"
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // Validate JWT token
        validate_token(token, &authz_state)
    }
}

/// Trait to define a role for the extractor
pub trait Role {
    fn name() -> &'static str;
}

/// Generic extractor for role checking
///
/// Note: checks for the 'admin' role also accept 'super_admin'; all
/// other roles require exact matches.
#[derive(Debug, Clone)]
pub struct RequireRole<R: Role> {
    pub user: AuthUser,
    _phantom: PhantomData<R>,
}

impl<R> FromRequestParts<()> for RequireRole<R>
where
    R: Role + Send + Sync,
{
    type Rejection = StatusCode;

    /// Extracts the authenticated user from the request and enforces that the user has role `R`.
    ///
    /// If the user's role matches `R::name()` (with `"admin"` treated as matching both `admin` and `super_admin`),
    /// the extractor returns a `RequireRole<R>` containing the authenticated `AuthUser`.
    ///
    /// # Returns
    ///
    /// `Ok(RequireRole<R>)` when the user is authenticated and authorized for the role, `Err(StatusCode::FORBIDDEN)`
    /// when the user is authenticated but does not have the required role. Other rejections from authentication are
    /// forwarded as-is.
    async fn from_request_parts(parts: &mut Parts, _state: &()) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, &()).await?;
        let required_role = R::name();

        let authorized = if required_role == "admin" {
            user.is_admin()
        } else {
            user.has_role(required_role)
        };

        if !authorized {
            warn!(
                "Role check failed for role '{}': user {} has role '{}'",
                required_role, user.user_id, user.role
            );
            return Err(StatusCode::FORBIDDEN);
        }

        debug!("Role check passed for role '{}': user {}", required_role, user.user_id);
        Ok(RequireRole {
            user,
            _phantom: PhantomData,
        })
    }
}

/// Define an AdminRole struct implementing the Role trait
pub struct AdminRole;
impl Role for AdminRole {
    /// Provides the canonical name for the admin role.
    fn name() -> &'static str {
        "admin"
    }
}

/// Extractor that requires admin role
///
/// Convenience extractor for admin-only endpoints.
/// It is implemented using the generic `RequireRole` extractor.
///
/// # Usage
/// ```no_run
/// use axum::{routing::get, Router};
/// use shared_auth::RequireAdmin;
///
/// async fn admin_handler(RequireAdmin(user): RequireAdmin) -> String {
///     format!("Hello, admin {}!", user.user_id)
/// }
///
/// let app = Router::new().route("/admin", get(admin_handler));
/// ```
#[derive(Debug, Clone)]
pub struct RequireAdmin(pub AuthUser);

impl FromRequestParts<()> for RequireAdmin {
    type Rejection = StatusCode;

    /// Extracts an authenticated admin user from request parts and returns it wrapped in `RequireAdmin`.
    ///
    /// This will perform JWT authentication and enforce that the authenticated user has an admin role; on failure it yields an appropriate `StatusCode` rejection.
    async fn from_request_parts(parts: &mut Parts, _state: &()) -> Result<Self, Self::Rejection> {
        let role_extractor = RequireRole::<AdminRole>::from_request_parts(parts, &()).await?;
        Ok(RequireAdmin(role_extractor.user))
    }
}

/// Extractor that checks Casbin permission
///
/// This extractor validates JWT and checks if the user has permission
/// to perform the action on the resource using Casbin.
///
/// Note: This requires the SharedEnforcer to be in the request extensions.
/// Use the `casbin_middleware` to add it automatically.
///
/// # Usage
/// ```no_run
/// use axum::{routing::get, Router, middleware};
/// use shared_auth::{RequirePermission, casbin_middleware, create_enforcer};
///
/// async fn handler(
///     RequirePermission(user): RequirePermission,
/// ) -> String {
///     format!("Hello, authorized user {}!", user.user_id)
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let enforcer = create_enforcer("postgres://localhost/db", None).await.unwrap();
///
///     let app = Router::new()
///         .route("/api/products", get(handler))
///         .layer(middleware::from_fn_with_state(enforcer, casbin_middleware));
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RequirePermission {
    pub user: AuthUser,
    pub resource: String,
    pub action: String,
}

impl RequirePermission {
    /// Create a new permission requirement
    pub fn new(user: AuthUser, resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            user,
            resource: resource.into(),
            action: action.into(),
        }
    }
}

impl FromRequestParts<()> for RequirePermission {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &()) -> Result<Self, Self::Rejection> {
        // First, extract the authenticated user
        let user = AuthUser::from_request_parts(parts, &()).await?;

        // Extract the enforcer from extensions
        let enforcer = parts
            .extensions
            .get::<SharedEnforcer>()
            .ok_or_else(|| {
                warn!("SharedEnforcer not found in request extensions. Did you forget to add casbin_middleware?");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .clone();

        // Get resource and action from request
        let resource = parts.uri.path().to_string();
        let action = parts.method.as_str().to_string();

        // Check permission with Casbin
        let e = enforcer.read().await;
        let allowed = e
            .enforce((user.user_id.to_string(), user.tenant_id.to_string(), &resource, &action))
            .map_err(|e| {
                warn!("Casbin error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        if !allowed {
            warn!(
                "Permission denied: user={}, resource={}, action={}",
                user.user_id, resource, action
            );
            return Err(StatusCode::FORBIDDEN);
        }

        debug!(
            "Permission granted: user={}, resource={}, action={}",
            user.user_id, resource, action
        );

        Ok(RequirePermission {
            user,
            resource,
            action,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_user_has_role() {
        let user = AuthUser {
            user_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            role: "admin".to_string(),
            email: None,
        };

        assert!(user.has_role("admin"));
        assert!(!user.has_role("user"));
        assert!(user.is_admin());
    }

    #[test]
    fn test_auth_user_is_admin() {
        let admin = AuthUser {
            user_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            role: "admin".to_string(),
            email: None,
        };

        let super_admin = AuthUser {
            user_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            role: "super_admin".to_string(),
            email: None,
        };

        let owner = AuthUser {
            user_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            role: "owner".to_string(),
            email: None,
        };

        let user = AuthUser {
            user_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            role: "user".to_string(),
            email: None,
        };

        assert!(admin.is_admin());
        assert!(super_admin.is_admin());
        assert!(owner.is_admin()); // Owner should have admin privileges
        assert!(!user.is_admin());
    }

    #[test]
    fn test_auth_user_is_owner() {
        let owner = AuthUser {
            user_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            role: "owner".to_string(),
            email: None,
        };

        let admin = AuthUser {
            user_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            role: "admin".to_string(),
            email: None,
        };

        assert!(owner.is_owner());
        assert!(!admin.is_owner());
    }
}
