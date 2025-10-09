use sqlx::PgPool;
use super::config::Config;

/// Shared application state
/// 
/// This state is cloned for each request handler via the `State` extractor.
/// All fields must be cheap to clone (Arc, Pool, etc.).
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool
    pub db: PgPool,
    
    /// Application configuration
    pub config: Config,
}

impl AppState {
    /// Create a new AppState
    pub fn new(db: PgPool, config: Config) -> Self {
        Self { db, config }
    }
}
