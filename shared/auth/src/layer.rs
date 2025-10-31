use crate::middleware::{AuthError, AuthzState};
use axum::body::Body;
use axum::response::{IntoResponse, Response};
use futures_util::future::BoxFuture;
use http::{header, Request};
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::{debug, warn};

#[derive(Clone)]
pub struct CasbinAuthLayer {
    state: AuthzState,
}

impl CasbinAuthLayer {
    pub fn new(state: AuthzState) -> Self {
        Self { state }
    }
}

impl<S> Layer<S> for CasbinAuthLayer {
    type Service = CasbinAuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CasbinAuthMiddleware {
            inner,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
pub struct CasbinAuthMiddleware<S> {
    inner: S,
    state: AuthzState,
}

impl<S> Service<Request<Body>> for CasbinAuthMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let state = self.state.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let auth_header = req
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok());

            if auth_header.is_none() {
                let error = AuthError::MissingToken;
                return Ok(error.into_response());
            }

            // Parse Bearer token with case-insensitive scheme matching (OAuth spec compliant)
            let token = auth_header
                .and_then(|value| value.trim().split_once(' '))
                .and_then(|(scheme, token)| scheme.eq_ignore_ascii_case("Bearer").then_some(token))
                .filter(|token| !token.is_empty());

            if token.is_none() {
                let error = AuthError::InvalidToken;
                return Ok(error.into_response());
            }

            let claims = match shared_jwt::decode_jwt(token.unwrap(), &state.jwt_secret) {
                Ok(claims) => claims,
                Err(_) => {
                    let error = AuthError::InvalidToken;
                    return Ok(error.into_response());
                },
            };

            debug!(
                "JWT validated: user_id={}, tenant_id={}, role={}",
                claims.sub, claims.tenant_id, claims.role
            );

            let resource = req.uri().path();
            let action = req.method().as_str();

            let allowed = match crate::middleware::check_permission(
                &state.enforcer,
                &claims.sub.to_string(),
                &claims.tenant_id.to_string(),
                resource,
                action,
            )
            .await
            {
                Ok(allowed) => allowed,
                Err(e) => {
                    return Ok(e.into_response());
                },
            };

            if !allowed {
                warn!(
                    "Permission denied: user={}, tenant={}, resource={}, action={}",
                    claims.sub, claims.tenant_id, resource, action
                );
                let error = AuthError::PermissionDenied;
                return Ok(error.into_response());
            }

            debug!(
                "Permission granted: user={}, resource={}, action={}",
                claims.sub, resource, action
            );

            inner.call(req).await
        })
    }
}
