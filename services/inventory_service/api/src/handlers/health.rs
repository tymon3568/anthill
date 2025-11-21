use axum::{http::StatusCode, response::Response, Json};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use shared_error::AppError;

#[derive(Serialize)]
pub struct HealthResp {
    pub status: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub database: String,
    pub nats: String,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    operation_id = "inventory_health_check",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResp),
        (status = 503, description = "Service is unhealthy", body = HealthResp)
    )
)]
pub async fn health_check(
    axum::Extension(pool): axum::Extension<PgPool>,
) -> Result<Response, AppError> {
    // Check database connection
    let db_status = match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => "healthy".to_string(),
        Err(e) => {
            tracing::error!("Database health check failed: {}", e);
            "unhealthy".to_string()
        },
    };

    // Check NATS connection
    let nats_status = match shared_events::get_nats_client() {
        Ok(client) => {
            // Try to check if NATS is responsive by attempting a simple operation
            match client.client.connection_state().await {
                async_nats::connection::State::Connected => "healthy".to_string(),
                _ => {
                    tracing::warn!("NATS connection is not in connected state");
                    "unhealthy".to_string()
                },
            }
        },
        Err(_) => {
            // NATS not initialized - this is ok if not configured
            "not_configured".to_string()
        },
    };

    let overall_status = if db_status == "healthy"
        && (nats_status == "healthy" || nats_status == "not_configured")
    {
        "ok"
    } else {
        "degraded"
    };

    let resp = HealthResp {
        status: overall_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: Utc::now(),
        database: db_status,
        nats: nats_status,
    };

    if overall_status == "degraded" {
        Ok((StatusCode::SERVICE_UNAVAILABLE, Json(resp)).into_response())
    } else {
        Ok((StatusCode::OK, Json(resp)).into_response())
    }
}
