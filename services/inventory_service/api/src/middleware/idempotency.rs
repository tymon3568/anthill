use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use redis::AsyncCommands;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Configuration for idempotency middleware
#[derive(Clone)]
pub struct IdempotencyConfig {
    /// Redis connection URL
    pub redis_url: String,
    /// TTL for idempotency keys in seconds (default: 24 hours)
    pub ttl_seconds: u64,
    /// Header name for idempotency key (default: "x-idempotency-key")
    pub header_name: String,
}

impl Default for IdempotencyConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            ttl_seconds: 24 * 60 * 60, // 24 hours
            header_name: "x-idempotency-key".to_string(),
        }
    }
}

/// Idempotency middleware state
#[derive(Clone)]
pub struct IdempotencyState {
    config: IdempotencyConfig,
    redis_client: redis::Client,
}

impl IdempotencyState {
    pub fn new(config: IdempotencyConfig) -> Result<Self, redis::RedisError> {
        let redis_client = redis::Client::open(config.redis_url.clone())?;
        Ok(Self {
            config,
            redis_client,
        })
    }

    /// Check if a request with the given idempotency key has already been processed
    pub async fn is_processed(&self, key: &str) -> Result<bool, redis::RedisError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let exists: bool = conn.exists(key).await?;
        Ok(exists)
    }

    /// Mark a request as processed with TTL
    pub async fn mark_processed(&self, key: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        conn.set_ex::<_, _, ()>(key, "processed", self.config.ttl_seconds)
            .await?;
        Ok(())
    }
}

/// Idempotency middleware for Axum
pub async fn idempotency_middleware(
    State(state): State<Arc<IdempotencyState>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    // Only apply to POST, PUT, PATCH methods
    if !matches!(
        request.method(),
        &axum::http::Method::POST | &axum::http::Method::PUT | &axum::http::Method::PATCH
    ) {
        return next.run(request).await;
    }

    // Extract idempotency key from header
    let header_name = &state.config.header_name;
    let idempotency_key = match headers.get(header_name) {
        Some(value) => match value.to_str() {
            Ok(key) => key.trim(),
            Err(_) => {
                warn!("Invalid idempotency key header value");
                return (StatusCode::BAD_REQUEST, "Invalid idempotency key format").into_response();
            },
        },
        None => {
            // Idempotency key is optional for GET requests, but required for mutations
            warn!("Missing required {} header for mutation request", header_name);
            return (StatusCode::BAD_REQUEST, format!("Missing required {} header", header_name))
                .into_response();
        },
    };

    // Validate key format (should be UUID or similar)
    if idempotency_key.is_empty() || idempotency_key.len() > 128 {
        warn!("Invalid idempotency key length: {}", idempotency_key.len());
        return (StatusCode::BAD_REQUEST, "Invalid idempotency key").into_response();
    }

    // Check if request has already been processed
    match state.is_processed(idempotency_key).await {
        Ok(true) => {
            info!("Duplicate request detected with key: {}", idempotency_key);
            return (
                StatusCode::CONFLICT,
                format!("Request with idempotency key '{}' already processed", idempotency_key),
            )
                .into_response();
        },
        Ok(false) => {
            // Key doesn't exist, proceed with request
        },
        Err(e) => {
            error!("Redis error checking idempotency key: {}", e);
            // In case of Redis failure, allow request to proceed to avoid blocking
            // This is a fail-open approach for better availability
            warn!("Redis unavailable, allowing request to proceed");
        },
    }

    // Execute the request
    let response = next.run(request).await;

    // Only mark as processed if request was successful (2xx status)
    if response.status().is_success() {
        if let Err(e) = state.mark_processed(idempotency_key).await {
            error!("Failed to mark request as processed: {}", e);
            // Don't fail the response if Redis write fails
        } else {
            info!("Marked request as processed: {}", idempotency_key);
        }
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, middleware::from_fn_with_state, Router};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_idempotency_key_validation() {
        // Test missing header
        let config = IdempotencyConfig::default();
        let state = Arc::new(IdempotencyState::new(config).unwrap());

        let app = Router::new()
            .route("/", axum::routing::post(|| async { "ok" }))
            .layer(from_fn_with_state(state, idempotency_middleware));

        let request = Request::builder()
            .method("POST")
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_empty_idempotency_key() {
        let config = IdempotencyConfig {
            header_name: "x-test-key".to_string(),
            ..Default::default()
        };
        let state = Arc::new(IdempotencyState::new(config).unwrap());

        let app = Router::new()
            .route("/", axum::routing::post(|| async { "ok" }))
            .layer(from_fn_with_state(state.clone(), idempotency_middleware));

        let request = Request::builder()
            .method("POST")
            .uri("/")
            .header("x-test-key", "")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
