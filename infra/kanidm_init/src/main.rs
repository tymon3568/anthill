mod client;
mod setup;

use anyhow::{Context, Result};
use client::KanidmAdminClient;
use setup::KanidmSetup;
use std::env;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("═══════════════════════════════════════");
    info!("Kanidm Initialization for Anthill");
    info!("═══════════════════════════════════════");
    info!("");

    // Load configuration from environment
    let kanidm_url = env::var("KANIDM_URL").unwrap_or_else(|_| "https://localhost:8300".to_string());
    let admin_username = env::var("KANIDM_ADMIN_USER").unwrap_or_else(|_| "admin".to_string());
    let admin_password = env::var("KANIDM_ADMIN_PASSWORD")
        .context("KANIDM_ADMIN_PASSWORD environment variable is required")?;

    info!("🔧 Configuration:");
    info!("  - Kanidm URL: {}", kanidm_url);
    info!("  - Admin User: {}", admin_username);
    info!("");

    // Wait for Kanidm to be ready
    wait_for_kanidm(&kanidm_url).await?;

    // Create client and authenticate
    let mut client = KanidmAdminClient::new(&kanidm_url)?;

    match client.authenticate(&admin_username, &admin_password).await {
        Ok(_) => info!(""),
        Err(e) => {
            error!("❌ Authentication failed: {}", e);
            error!("");
            error!("💡 Tip: You may need to recover the admin account first:");
            error!("   docker exec -it kanidm_idm kanidmd recover-account admin");
            error!("");
            return Err(e);
        }
    }

    // Run setup
    let setup = KanidmSetup::new(client);

    if let Err(e) = setup.setup_oauth2_client().await {
        error!("❌ OAuth2 client setup failed: {}", e);
        return Err(e);
    }

    if let Err(e) = setup.setup_groups().await {
        error!("❌ Groups setup failed: {}", e);
        return Err(e);
    }

    if let Err(e) = setup.setup_test_users().await {
        error!("❌ Test users setup failed: {}", e);
        return Err(e);
    }

    // Print summary
    setup.print_summary();

    Ok(())
}

async fn wait_for_kanidm(base_url: &str) -> Result<()> {
    info!("⏳ Waiting for Kanidm to be ready...");

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(5))
        .build()?;

    let status_url = format!("{}/status", base_url);

    for attempt in 1..=30 {
        match client.get(&status_url).send().await {
            Ok(response) if response.status().is_success() => {
                info!("✅ Kanidm is ready!");
                info!("");
                return Ok(());
            }
            Ok(response) => {
                warn!(
                    "  Attempt {}/30: Kanidm returned status {}, waiting 2s...",
                    attempt,
                    response.status()
                );
            }
            Err(e) => {
                warn!("  Attempt {}/30: {}, waiting 2s...", attempt, e);
            }
        }

        sleep(Duration::from_secs(2)).await;
    }

    anyhow::bail!("Kanidm did not become ready after 30 attempts (60 seconds)")
}
