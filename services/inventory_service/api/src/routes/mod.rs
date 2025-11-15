//! Route definitions and router creation
//!
//! This module defines the API routes and creates the main router.

use axum::{http::HeaderValue, Router};
use shared_auth::{
    enforcer::create_enforcer,
    middleware::{casbin_middleware, AuthzState},
};
use shared_config::Config;
use shared_kanidm_client::{KanidmClient, KanidmConfig};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::handlers::category::{create_category_routes, AppState};
use crate::handlers::search::create_search_routes;
use crate::handlers::valuation::create_valuation_routes;
use crate::handlers::warehouses::create_warehouse_routes;
use inventory_service_infra::repositories::category::CategoryRepositoryImpl;
use inventory_service_infra::repositories::product::ProductRepositoryImpl;
use inventory_service_infra::repositories::valuation::ValuationRepositoryImpl;
use inventory_service_infra::repositories::warehouse::WarehouseRepositoryImpl;
use inventory_service_infra::services::category::CategoryServiceImpl;
use inventory_service_infra::services::product::ProductServiceImpl;
use inventory_service_infra::services::valuation::ValuationServiceImpl;

/// Create Kanidm client from configuration
fn create_kanidm_client(config: &Config) -> KanidmClient {
    let is_dev = std::env::var("APP_ENV")
        .or_else(|_| std::env::var("RUST_ENV"))
        .map(|e| e != "production")
        .unwrap_or(true);

    // In production, require full Kanidm configuration
    if !is_dev {
        if config.kanidm_url.is_none()
            || config.kanidm_client_id.is_none()
            || config.kanidm_client_secret.is_none()
            || config.kanidm_redirect_url.is_none()
        {
            panic!(
                "Kanidm configuration is required in production environment. \
                 Set KANIDM_URL, KANIDM_CLIENT_ID, KANIDM_CLIENT_SECRET, \
                 and KANIDM_REDIRECT_URL environment variables."
            );
        }
    }

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
        skip_jwt_verification: is_dev,
        allowed_issuers: vec![config
            .kanidm_url
            .clone()
            .unwrap_or_else(|| "http://localhost:8300".to_string())],
        expected_audience: config.kanidm_client_id.clone(),
    };

    KanidmClient::new(kanidm_config).expect(
        "Failed to create Kanidm client. Check kanidm_url, client_id, client_secret, and redirect_uri configuration."
    )
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
        .copied();

    let model_path = match model_path {
        Some(path) => path,
        None => {
            panic!(
                "Casbin model file not found. Tried paths: {:?}. \
                 Ensure shared/auth/model.conf exists in the workspace.",
                model_paths
            );
        },
    };

    let enforcer = create_enforcer(&config.database_url, Some(model_path))
        .await
        .expect("Failed to initialize Casbin enforcer");

    // Initialize repositories and services
    let category_repo = CategoryRepositoryImpl::new(pool.clone());
    let category_service = CategoryServiceImpl::new(category_repo);

    let product_repo = ProductRepositoryImpl::new(pool.clone());
    let product_service = ProductServiceImpl::new(Arc::new(product_repo));

    let valuation_repo = ValuationRepositoryImpl::new(pool.clone());
    let valuation_service = ValuationServiceImpl::new(
        Arc::new(valuation_repo.clone())
            as Arc<dyn inventory_service_core::repositories::valuation::ValuationRepository>,
        Arc::new(valuation_repo.clone())
            as Arc<dyn inventory_service_core::repositories::valuation::ValuationLayerRepository>,
        Arc::new(valuation_repo)
            as Arc<dyn inventory_service_core::repositories::valuation::ValuationHistoryRepository>,
    );

    let warehouse_repo = WarehouseRepositoryImpl::new(pool.clone());

    // Create application state
    let state = AppState {
        category_service: Arc::new(category_service),
        product_service: Arc::new(product_service),
        valuation_service: Arc::new(valuation_service),
        warehouse_repository: Arc::new(warehouse_repo),
        enforcer,
        jwt_secret: config.jwt_secret.clone(),
        kanidm_client: create_kanidm_client(config),
    };

    // Create AuthzState for middleware
    let authz_state = AuthzState {
        enforcer: state.enforcer.clone(),
        jwt_secret: state.jwt_secret.clone(),
    };

    // Create routes with state
    let category_routes = create_category_routes(state.clone());
    let search_routes = create_search_routes(state.clone());
    let valuation_routes = create_valuation_routes(state.clone());
    let warehouse_routes = create_warehouse_routes(state.clone());

    // Add CORS configuration
    let cors = CorsLayer::new()
        .allow_origin({
            let origins = config.get_cors_origins();
            if origins.is_empty() {
                AllowOrigin::any()
            } else {
                let header_values: Result<Vec<HeaderValue>, _> = origins
                    .into_iter()
                    .map(|origin| {
                        HeaderValue::from_str(&origin)
                            .map_err(|e| format!("Invalid CORS origin '{}': {}", origin, e))
                    })
                    .collect();

                match header_values {
                    Ok(values) => AllowOrigin::list(values),
                    Err(e) => panic!("CORS configuration error: {}", e),
                }
            }
        })
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);

    Router::new()
        .nest("/api/v1/inventory", category_routes)
        .nest("/api/v1/inventory/products", search_routes)
        .nest("/api/v1/inventory/valuation", valuation_routes)
        .nest("/api/v1/inventory/warehouses", warehouse_routes)
        .layer(axum::middleware::from_fn_with_state(authz_state, casbin_middleware))
        .layer(cors)
}
