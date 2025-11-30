//! Inventory Service Main Application
//!
//! This is the main entry point for the inventory service.
//! It sets up the web server and starts the application.

use std::net::SocketAddr;

use inventory_service_api::create_router;
use shared_config::Config;
use shared_db::init_pool;
use tokio::net::TcpListener;
use tower::make::Shared;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "inventory_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;

    // Initialize database connection pool
    let pool = init_pool(&config.database_url, config.max_connections.unwrap_or(10)).await?;

    // Initialize event consumers (if NATS is configured)
    if let Some(nats_url) = &config.nats_url {
        if let Err(e) =
            inventory_service_api::consumers::init_event_consumers(pool.clone(), nats_url).await
        {
            tracing::error!("Failed to initialize NATS event consumers: {}", e);
            tracing::warn!("Service will start without event processing capabilities");
        } else {
            tracing::info!("NATS event consumers initialized successfully");
        }
    }

    // Create the application router
    let app = create_router(pool, &config).await;

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Inventory service listening on {}", addr);

    axum::serve(listener, tower::make::Shared::new(app)).await?;

    Ok(())
}
