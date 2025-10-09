use sqlx::postgres::PgPoolOptions;
use super::{app_state::AppState, config::Config};
use crate::common::error::AppError;

/// Initialize the application
/// 
/// This function loads configuration, connects to the database,
/// and creates the AppState.
pub async fn init() -> Result<AppState, AppError> {
    tracing::info!("🚀 Initializing User Service...");
    
    // Load configuration
    let config = Config::from_env()
        .map_err(|e| AppError::InternalError(format!("Failed to load config: {}", e)))?;
    
    tracing::info!("✅ Configuration loaded");
    
    // Connect to database
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to connect to database: {}", e)))?;
    
    tracing::info!("✅ Database connected");
    
    // Create AppState
    let state = AppState::new(db, config);
    
    tracing::info!("✅ AppState initialized");
    
    Ok(state)
}
