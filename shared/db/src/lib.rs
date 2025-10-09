use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use shared_error::AppError;

/// Initialize database connection pool
pub async fn init_pool(database_url: &str, max_connections: u32) -> Result<PgPool, AppError> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to connect to database: {}", e)))
}
