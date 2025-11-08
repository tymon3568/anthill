//! Route definitions and router creation
//!
//! This module defines the API routes and creates the main router.

use axum::{http::HeaderValue, Router};
use shared_auth::enforcer::create_enforcer;
use shared_config::Config;
use shared_kanidm_client::{KanidmClient, KanidmConfig};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

use crate::handlers::category::{create_category_routes, AppState};
use inventory_service_infra::repositories::category::CategoryRepositoryImpl;
use inventory_service_infra::services::category::CategoryServiceImpl;

/// Create Kanidm client from configuration
fn create_kanidm_client(config: &Config) -> KanidmClient {
    let kanidm_config = KanidmConfig {
        kanidm_url: config
            .kanidm_url
            .clone()
            .unwrap_or_else(|| "http://localhost:8300".to_string()),
        client_id: config
            .kanidm_client_id
            .clone()
            .unwrap_or_else(|| "dev".to_string()),
        client_secret: config
            .kanidm_client_secret
            .clone()
            .unwrap_or_else(|| "dev".to_string()),
        redirect_uri: config
            .kanidm_redirect_url
            .clone()
            .unwrap_or_else(|| "http://localhost:8001/oauth/callback".to_string()),
        scopes: vec!["openid".to_string()],
        skip_jwt_verification: true, // DEV/TEST MODE ONLY - should be false in production
        allowed_issuers: vec![config
            .kanidm_url
            .clone()
            .unwrap_or_else(|| "http://localhost:8300".to_string())],
        expected_audience: config.kanidm_client_id.clone(),
    };

    KanidmClient::new(kanidm_config).expect("Failed to create Kanidm client")
}

/// Create the main application router
pub async fn create_router(pool: PgPool, config: &Config) -> Router {
    // Validate CORS configuration for production
    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    let rust_env = std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
    let is_production = app_env == "production" || rust_env == "production";

    if is_production && config.get_cors_origins().is_empty() {
        panic!(
            "CORS_ORIGINS must be configured in production environment. \
             Set CORS_ORIGINS=https://your-domain.com,https://admin.your-domain.com"
        );
    }

    // Initialize Casbin enforcer
    let model_paths = [
        "shared/auth/model.conf",             // From workspace root
        "../../../shared/auth/model.conf",    // From services/inventory_service/api
        "../../../../shared/auth/model.conf", // From target/debug
    ];

    let model_path = model_paths
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .copied()
        .unwrap_or("shared/auth/model.conf");

    let enforcer = create_enforcer(&config.database_url, Some(model_path))
        .await
        .expect("Failed to initialize Casbin enforcer");

    // Initialize repository and service
    let category_repo = CategoryRepositoryImpl::new(pool.clone());
    let category_service = CategoryServiceImpl::new(category_repo);

    // Create application state
    let state = AppState {
        category_service: Arc::new(category_service),
        enforcer,
        jwt_secret: config.jwt_secret.clone(),
        kanidm_client: create_kanidm_client(config),
    };

    // Create category routes with state
    let category_routes = create_category_routes(state);

    // Add CORS configuration
    let cors = CorsLayer::new()
        .allow_origin({
            let origins = config.get_cors_origins();
            if origins.is_empty() {
                AllowOrigin::any()
            } else {
                let header_values: Vec<HeaderValue> = origins
                    .into_iter()
                    .filter_map(|origin| HeaderValue::from_str(&origin).ok())
                    .collect();
                AllowOrigin::list(header_values)
            }
        })
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api/v1/inventory", category_routes)
        .layer(cors)
}
