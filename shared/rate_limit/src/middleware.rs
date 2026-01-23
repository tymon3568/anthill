//! Axum middleware for rate limiting

use crate::config::RateLimitConfig;
use crate::limiter::{KeyGenerator, RateLimitError, RateLimitResult, RateLimiter};
use crate::memory_limiter::InMemoryRateLimiter;
use crate::redis_limiter::RedisRateLimiter;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{header, HeaderValue, Request, Response, StatusCode};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use tower::{Layer, Service};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Rate limit endpoint type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitEndpoint {
    Login,
    Register,
    ForgotPassword,
    ResendVerification,
    Refresh,
    AcceptInvite,
    FileUpload,
    Global,
}

impl RateLimitEndpoint {
    /// Get the key prefix for this endpoint
    pub fn key_prefix(&self) -> &'static str {
        match self {
            Self::Login => "rate_limit:login:ip",
            Self::Register => "rate_limit:register:ip",
            Self::ForgotPassword => "rate_limit:forgot:email",
            Self::ResendVerification => "rate_limit:resend:user",
            Self::Refresh => "rate_limit:refresh:user",
            Self::AcceptInvite => "rate_limit:accept_invite:ip",
            Self::FileUpload => "rate_limit:file_upload:user",
            Self::Global => "rate_limit:global:ip",
        }
    }

    /// Check if this endpoint uses IP-based rate limiting
    pub fn is_ip_based(&self) -> bool {
        matches!(self, Self::Login | Self::Register | Self::AcceptInvite | Self::Global)
    }

    /// Check if this endpoint uses email-based rate limiting
    pub fn is_email_based(&self) -> bool {
        matches!(self, Self::ForgotPassword)
    }

    /// Check if this endpoint uses user-based rate limiting
    pub fn is_user_based(&self) -> bool {
        matches!(self, Self::ResendVerification | Self::Refresh | Self::FileUpload)
    }
}

/// Shared rate limiter that can use either Redis or in-memory storage
#[derive(Clone)]
pub enum SharedRateLimiter {
    Redis(RedisRateLimiter),
    InMemory(InMemoryRateLimiter),
}

impl SharedRateLimiter {
    /// Create a new shared rate limiter based on configuration
    pub async fn from_config(config: &RateLimitConfig) -> Self {
        if let Some(redis_url) = &config.redis_url {
            match RedisRateLimiter::new(redis_url).await {
                Ok(limiter) => {
                    info!("Rate limiter using Redis backend");
                    return Self::Redis(limiter);
                },
                Err(e) => {
                    warn!(
                        "Failed to connect to Redis for rate limiting: {}. Falling back to in-memory.",
                        e
                    );
                },
            }
        }

        info!("Rate limiter using in-memory backend");
        Self::InMemory(InMemoryRateLimiter::new())
    }

    /// Check rate limit
    pub async fn check(
        &self,
        key: &str,
        max_requests: u32,
        window: Duration,
    ) -> Result<RateLimitResult, RateLimitError> {
        match self {
            Self::Redis(limiter) => limiter.check_rate_limit(key, max_requests, window).await,
            Self::InMemory(limiter) => limiter.check_rate_limit(key, max_requests, window).await,
        }
    }

    /// Reset rate limit for a key
    pub async fn reset(&self, key: &str) -> Result<(), RateLimitError> {
        match self {
            Self::Redis(limiter) => limiter.reset(key).await,
            Self::InMemory(limiter) => limiter.reset(key).await,
        }
    }

    /// Get current count for a key
    pub async fn get_count(&self, key: &str) -> Result<u32, RateLimitError> {
        match self {
            Self::Redis(limiter) => limiter.get_count(key).await,
            Self::InMemory(limiter) => limiter.get_count(key).await,
        }
    }

    /// Get TTL for a key
    pub async fn get_ttl(&self, key: &str) -> Result<u64, RateLimitError> {
        match self {
            Self::Redis(limiter) => limiter.get_ttl(key).await,
            Self::InMemory(limiter) => limiter.get_ttl(key).await,
        }
    }

    /// Check if healthy
    pub async fn is_healthy(&self) -> bool {
        match self {
            Self::Redis(limiter) => limiter.is_healthy().await,
            Self::InMemory(limiter) => limiter.is_healthy().await,
        }
    }
}

/// Rate limit state for the middleware
#[derive(Clone)]
pub struct RateLimitState {
    /// The rate limiter implementation
    pub limiter: Arc<SharedRateLimiter>,
    /// Configuration
    pub config: RateLimitConfig,
}

impl RateLimitState {
    /// Create a new rate limit state
    pub async fn from_config(config: RateLimitConfig) -> Self {
        let limiter = SharedRateLimiter::from_config(&config).await;
        Self {
            limiter: Arc::new(limiter),
            config,
        }
    }

    /// Check rate limit for an endpoint
    /// The identifier should be:
    /// - IP address for IP-based endpoints (Login, Register, AcceptInvite, Global)
    /// - Email for email-based endpoints (ForgotPassword)
    /// - User ID for user-based endpoints (ResendVerification, Refresh)
    pub async fn check_endpoint(
        &self,
        endpoint: RateLimitEndpoint,
        identifier: &str,
    ) -> Result<RateLimitResult, RateLimitError> {
        let (max_requests, window_seconds) = match endpoint {
            RateLimitEndpoint::Login => {
                (self.config.login_max_attempts, self.config.login_window_seconds)
            },
            RateLimitEndpoint::Register => {
                (self.config.register_max_attempts, self.config.register_window_seconds)
            },
            RateLimitEndpoint::ForgotPassword => {
                (self.config.forgot_password_max, self.config.forgot_password_window)
            },
            RateLimitEndpoint::ResendVerification => {
                (self.config.resend_verification_max, self.config.resend_verification_window)
            },
            RateLimitEndpoint::Refresh => (self.config.refresh_max, self.config.refresh_window),
            RateLimitEndpoint::AcceptInvite => {
                (self.config.accept_invite_max, self.config.accept_invite_window)
            },
            RateLimitEndpoint::FileUpload => {
                (self.config.file_upload_max, self.config.file_upload_window)
            },
            RateLimitEndpoint::Global => (self.config.global_requests_per_second * 60, 60),
        };

        // Use appropriate key generator based on endpoint type
        let key = match endpoint {
            RateLimitEndpoint::Login
            | RateLimitEndpoint::Register
            | RateLimitEndpoint::AcceptInvite
            | RateLimitEndpoint::Global => KeyGenerator::ip_key(endpoint.key_prefix(), identifier),
            RateLimitEndpoint::ForgotPassword => {
                KeyGenerator::email_key(endpoint.key_prefix(), identifier)
            },
            RateLimitEndpoint::ResendVerification
            | RateLimitEndpoint::Refresh
            | RateLimitEndpoint::FileUpload => {
                KeyGenerator::user_key(endpoint.key_prefix(), identifier)
            },
        };

        self.limiter
            .check(&key, max_requests, Duration::from_secs(window_seconds))
            .await
    }
}

/// Layer for applying rate limiting to routes
#[derive(Clone)]
pub struct RateLimitLayer {
    state: RateLimitState,
    endpoint: RateLimitEndpoint,
    /// JWT secret for user-based rate limiting
    jwt_secret: Option<String>,
}

impl RateLimitLayer {
    /// Create a new rate limit layer for a specific endpoint
    pub fn new(state: RateLimitState, endpoint: RateLimitEndpoint) -> Self {
        Self {
            state,
            endpoint,
            jwt_secret: None,
        }
    }

    /// Create a new rate limit layer with JWT secret for user-based endpoints
    pub fn with_jwt_secret(
        state: RateLimitState,
        endpoint: RateLimitEndpoint,
        jwt_secret: String,
    ) -> Self {
        Self {
            state,
            endpoint,
            jwt_secret: Some(jwt_secret),
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitMiddleware {
            inner,
            state: self.state.clone(),
            endpoint: self.endpoint,
            jwt_secret: self.jwt_secret.clone(),
        }
    }
}

/// Rate limiting middleware
#[derive(Clone)]
pub struct RateLimitMiddleware<S> {
    inner: S,
    state: RateLimitState,
    endpoint: RateLimitEndpoint,
    jwt_secret: Option<String>,
}

impl<S> Service<Request<Body>> for RateLimitMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let state = self.state.clone();
        let endpoint = self.endpoint;
        let jwt_secret = self.jwt_secret.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Skip rate limiting if disabled
            if !state.config.enabled {
                return inner.call(req).await;
            }

            // Extract client IP using configuration
            let ip = extract_client_ip(&req, &state.config);

            // Check if IP is trusted
            if state.config.is_trusted_ip(&ip) {
                debug!("Trusted IP {} bypassing rate limit", ip);
                return inner.call(req).await;
            }

            // Determine the identifier based on endpoint type
            let identifier = if endpoint.is_user_based() {
                // For user-based endpoints, extract user ID from JWT
                match extract_user_id_from_jwt(&req, jwt_secret.as_deref()) {
                    Some(user_id) => user_id.to_string(),
                    None => {
                        // If no valid JWT, fall back to IP-based rate limiting
                        // This provides some protection even for unauthenticated requests
                        debug!(
                            "No valid JWT for user-based rate limit on {:?}, falling back to IP",
                            endpoint
                        );
                        ip.clone()
                    },
                }
            } else {
                ip.clone()
            };

            // Check rate limit
            match state.check_endpoint(endpoint, &identifier).await {
                Ok(result) if result.allowed => {
                    // Add rate limit headers to response
                    let response = inner.call(req).await?;
                    Ok(add_rate_limit_headers(response, &result))
                },
                Ok(result) => {
                    // Rate limit exceeded
                    info!(
                        "Rate limit exceeded for {} on {:?}: {}/{} requests",
                        identifier, endpoint, result.limit, result.limit
                    );
                    Ok(rate_limit_exceeded_response(&result))
                },
                Err(e) => {
                    // Log error but allow request (fail open for availability)
                    warn!("Rate limit check failed: {}. Allowing request.", e);
                    inner.call(req).await
                },
            }
        })
    }
}

/// Extract user ID from JWT token in Authorization header
fn extract_user_id_from_jwt<B>(req: &Request<B>, jwt_secret: Option<&str>) -> Option<Uuid> {
    let jwt_secret = jwt_secret?;

    // Extract Authorization header
    let auth_header = req.headers().get(header::AUTHORIZATION)?;
    let auth_str = auth_header.to_str().ok()?;

    // Extract token from "Bearer <token>"
    let token = auth_str.strip_prefix("Bearer ")?;

    // Decode JWT to get user ID
    match shared_jwt::decode_jwt(token, jwt_secret) {
        Ok(claims) => Some(claims.sub),
        Err(e) => {
            debug!("Failed to decode JWT for rate limiting: {}", e);
            None
        },
    }
}

/// Extract client IP from request
///
/// Security considerations:
/// - Only trust proxy headers (X-Forwarded-For, X-Real-IP) when `trust_proxy_headers` is enabled
/// - When behind a trusted proxy, use rightmost-trusted strategy based on `proxy_count`
/// - Falls back to connection info when headers are disabled or unavailable
fn extract_client_ip<B>(req: &Request<B>, config: &RateLimitConfig) -> String {
    // Only trust proxy headers if explicitly configured
    if config.trust_proxy_headers {
        // Try X-Forwarded-For header (for proxied requests)
        if let Some(xff) = req.headers().get("x-forwarded-for") {
            if let Ok(xff_str) = xff.to_str() {
                // X-Forwarded-For contains: client, proxy1, proxy2, ...
                // Use rightmost-trusted strategy: trust the IP at position (len - proxy_count)
                let ips: Vec<&str> = xff_str.split(',').map(|s| s.trim()).collect();
                if !ips.is_empty() {
                    // Calculate index: we trust `proxy_count` proxies from the right
                    // So the client IP is at position (len - proxy_count - 1), but we want
                    // the rightmost IP added by a trusted proxy, which is (len - proxy_count)
                    let index = ips.len().saturating_sub(config.proxy_count as usize + 1);
                    let ip = ips.get(index).unwrap_or(&ips[0]).trim();
                    if !ip.is_empty() {
                        return ip.to_string();
                    }
                }
            }
        }

        // Try X-Real-IP header (simpler, single IP from reverse proxy)
        if let Some(real_ip) = req.headers().get("x-real-ip") {
            if let Ok(ip) = real_ip.to_str() {
                let ip = ip.trim();
                if !ip.is_empty() {
                    return ip.to_string();
                }
            }
        }
    }

    // Fall back to connection info (direct connection IP)
    if let Some(connect_info) = req.extensions().get::<ConnectInfo<SocketAddr>>() {
        return connect_info.0.ip().to_string();
    }

    // Ultimate fallback
    "unknown".to_string()
}

/// Add rate limit headers to response
fn add_rate_limit_headers<B>(mut response: Response<B>, result: &RateLimitResult) -> Response<B> {
    let headers = response.headers_mut();

    if let Ok(v) = HeaderValue::from_str(&result.limit.to_string()) {
        headers.insert("x-ratelimit-limit", v);
    }
    if let Ok(v) = HeaderValue::from_str(&result.remaining.to_string()) {
        headers.insert("x-ratelimit-remaining", v);
    }
    if let Ok(v) = HeaderValue::from_str(&result.reset_at.to_string()) {
        headers.insert("x-ratelimit-reset", v);
    }

    response
}

/// Create a 429 Too Many Requests response
fn rate_limit_exceeded_response(result: &RateLimitResult) -> Response<Body> {
    let body = serde_json::json!({
        "error": "rate_limit_exceeded",
        "message": "Too many requests. Please try again later.",
        "retry_after_seconds": result.retry_after
    });

    let mut response = Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .header(header::CONTENT_TYPE, "application/json")
        .header("retry-after", result.retry_after.to_string())
        .body(Body::from(body.to_string()))
        .unwrap();

    // Add rate limit headers
    let headers = response.headers_mut();
    if let Ok(v) = HeaderValue::from_str(&result.limit.to_string()) {
        headers.insert("x-ratelimit-limit", v);
    }
    headers.insert("x-ratelimit-remaining", HeaderValue::from_static("0"));
    if let Ok(v) = HeaderValue::from_str(&result.reset_at.to_string()) {
        headers.insert("x-ratelimit-reset", v);
    }

    response
}

/// Extension trait for adding rate limiting to axum routers
pub trait RateLimitExt {
    /// Apply rate limiting to this router for a specific endpoint
    fn rate_limit(self, state: RateLimitState, endpoint: RateLimitEndpoint) -> Self;
}

impl<S> RateLimitExt for axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn rate_limit(self, state: RateLimitState, endpoint: RateLimitEndpoint) -> Self {
        self.layer(RateLimitLayer::new(state, endpoint))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_key_prefix() {
        assert_eq!(RateLimitEndpoint::Login.key_prefix(), "rate_limit:login:ip");
        assert_eq!(RateLimitEndpoint::Register.key_prefix(), "rate_limit:register:ip");
        assert_eq!(RateLimitEndpoint::AcceptInvite.key_prefix(), "rate_limit:accept_invite:ip");
    }

    #[test]
    fn test_endpoint_types() {
        assert!(RateLimitEndpoint::Login.is_ip_based());
        assert!(RateLimitEndpoint::Register.is_ip_based());
        assert!(RateLimitEndpoint::AcceptInvite.is_ip_based());

        assert!(RateLimitEndpoint::ForgotPassword.is_email_based());

        assert!(RateLimitEndpoint::Refresh.is_user_based());
        assert!(RateLimitEndpoint::ResendVerification.is_user_based());
    }

    #[tokio::test]
    async fn test_shared_limiter_in_memory() {
        let config = RateLimitConfig::default();
        let limiter = SharedRateLimiter::from_config(&config).await;

        let result = limiter
            .check("test:key", 5, Duration::from_secs(60))
            .await
            .unwrap();
        assert!(result.allowed);
        assert_eq!(result.remaining, 4);
    }

    #[tokio::test]
    async fn test_shared_limiter_get_count() {
        let config = RateLimitConfig::default();
        let limiter = SharedRateLimiter::from_config(&config).await;

        // Initial count should be 0
        let count = limiter.get_count("test:count:key").await.unwrap();
        assert_eq!(count, 0);

        // After one request, count should be 1
        limiter
            .check("test:count:key", 10, Duration::from_secs(60))
            .await
            .unwrap();
        let count = limiter.get_count("test:count:key").await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_rate_limit_state() {
        let config = RateLimitConfig {
            login_max_attempts: 3,
            login_window_seconds: 60,
            ..Default::default()
        };
        let state = RateLimitState::from_config(config).await;

        // First 3 should be allowed
        for _ in 0..3 {
            let result = state
                .check_endpoint(RateLimitEndpoint::Login, "192.168.1.1")
                .await
                .unwrap();
            assert!(result.allowed);
        }

        // 4th should be denied
        let result = state
            .check_endpoint(RateLimitEndpoint::Login, "192.168.1.1")
            .await
            .unwrap();
        assert!(!result.allowed);
    }
}
