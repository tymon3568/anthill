//! Inventory Service Main Application
//!
//! This is the main entry point for the inventory service.
//! It sets up the web server and starts the application.

use inventory_service_api::{create_router, worker};
use shared_config::Config;
use shared_db::init_pool;
use std::net::SocketAddr;
use tokio::net::TcpListener;

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

    // Initialize event consumers and outbox worker (if NATS is configured)
    if let Some(nats_url) = &config.nats_url {
        // Connect to NATS
        let nats_client = match async_nats::connect(nats_url).await {
            Ok(client) => {
                tracing::info!("Connected to NATS at {}", nats_url);
                client
            },
            Err(e) => {
                tracing::error!("Failed to connect to NATS: {}", e);
                tracing::warn!("Service will start without event processing capabilities");
                return Ok(());
            },
        };

        // Initialize event consumers
        if let Err(e) =
            inventory_service_api::consumers::init_event_consumers(pool.clone(), nats_url).await
        {
            tracing::error!("Failed to initialize NATS event consumers: {}", e);
        } else {
            tracing::info!("NATS event consumers initialized successfully");
        }

        // Start outbox worker
        let worker_config = worker::OutboxWorkerConfig::default();
        tokio::spawn(async move {
            if let Err(e) = worker::start_outbox_worker(pool, nats_client, worker_config).await {
                tracing::error!("Outbox worker failed: {}", e);
            }
        });
        tracing::info!("Outbox worker started");
    }

    // Create the application router
    let app = create_router(pool, &config).await;

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Inventory service listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
