//! Route definitions and router creation
//!
//! This module defines the API routes and creates the main router.

// Standard library/external crates
use async_trait::async_trait;
use axum::{extract::Extension, http::HeaderValue, routing::get, Router};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};
use uuid::Uuid;

// Shared crates
use shared_auth::{
    enforcer::create_enforcer,
    middleware::{casbin_middleware, AuthzState},
};
use shared_config::Config;
use shared_error::AppError;
use shared_kanidm_client::{KanidmClient, KanidmConfig};

// Inventory-service core
use inventory_service_core::dto::delivery::{
    PackItemsRequest, PackItemsResponse, PickItemsRequest, PickItemsResponse, ShipItemsRequest,
    ShipItemsResponse,
};
use inventory_service_core::services::delivery::DeliveryService;

// Inventory-service infra
use inventory_service_infra::repositories::category::CategoryRepositoryImpl;
use inventory_service_infra::repositories::product::ProductRepositoryImpl;
use inventory_service_infra::repositories::reconciliation::{
    PgStockReconciliationItemRepository, PgStockReconciliationRepository,
};
use inventory_service_infra::repositories::rma::{PgRmaItemRepository, PgRmaRepository};
use inventory_service_infra::repositories::stock::{
    PgInventoryLevelRepository, PgStockMoveRepository,
};
use inventory_service_infra::repositories::stock_take::{
    PgStockTakeLineRepository, PgStockTakeRepository,
};
use inventory_service_infra::repositories::transfer::{
    PgTransferItemRepository, PgTransferRepository,
};
use inventory_service_infra::repositories::valuation::ValuationRepositoryImpl;
use inventory_service_infra::repositories::warehouse::WarehouseRepositoryImpl;
use inventory_service_infra::services::category::CategoryServiceImpl;
use inventory_service_infra::services::lot_serial::LotSerialServiceImpl;

// Local handlers/state
use crate::handlers::category::create_category_routes;
#[cfg(feature = "delivery")]
use crate::handlers::delivery::create_delivery_routes;
use crate::handlers::lot_serial::create_lot_serial_routes;
use crate::handlers::receipt::create_receipt_routes;
use crate::handlers::reconciliation::create_reconciliation_routes;
use crate::handlers::rma::create_rma_routes;
use crate::handlers::search::create_search_routes;
use crate::handlers::stock_take::create_stock_take_routes;
use crate::handlers::transfer::create_transfer_routes;
use crate::handlers::valuation::create_valuation_routes;
use crate::handlers::warehouses::create_warehouse_routes;
use crate::state::AppState;

/// Create Kanidm client from configuration
fn create_kanidm_client(config: &Config) -> KanidmClient {
    let is_dev = std::env::var("APP_ENV")
        .or_else(|_| std::env::var("RUST_ENV"))
        .map(|e| e != "production")
        .unwrap_or(true);

    // In production, require full Kanidm configuration
    if !is_dev
        && (config.kanidm_url.is_none()
            || config.kanidm_client_id.is_none()
            || config.kanidm_client_secret.is_none()
            || config.kanidm_redirect_url.is_none())
    {
        panic!(
            "Kanidm configuration is required in production environment. \
             Set KANIDM_URL, KANIDM_CLIENT_ID, KANIDM_CLIENT_SECRET, \
             and KANIDM_REDIRECT_URL environment variables."
        );
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

// Dummy delivery service to avoid compile errors when delivery is disabled
pub struct DummyDeliveryService;

#[async_trait]
impl DeliveryService for DummyDeliveryService {
    async fn pick_items(
        &self,
        _tenant_id: Uuid,
        _delivery_id: Uuid,
        _user_id: Uuid,
        _request: PickItemsRequest,
    ) -> Result<PickItemsResponse, AppError> {
        Err(AppError::ServiceUnavailable(
            "Delivery service is disabled. Enable with --features delivery".to_string(),
        ))
    }

    async fn pack_items(
        &self,
        _tenant_id: Uuid,
        _delivery_id: Uuid,
        _user_id: Uuid,
        _request: PackItemsRequest,
    ) -> Result<PackItemsResponse, AppError> {
        Err(AppError::ServiceUnavailable(
            "Delivery service is disabled. Enable with --features delivery".to_string(),
        ))
    }

    async fn ship_items(
        &self,
        _tenant_id: Uuid,
        _delivery_id: Uuid,
        _user_id: Uuid,
        _request: ShipItemsRequest,
    ) -> Result<ShipItemsResponse, AppError> {
        Err(AppError::ServiceUnavailable(
            "Delivery service is disabled. Enable with --features delivery".to_string(),
        ))
    }
}

/// Create the main application router
pub async fn create_router(pool: PgPool, config: &Config) -> Router<AppState> {
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

    let lot_serial_repo =
        inventory_service_infra::repositories::lot_serial::LotSerialRepositoryImpl::new(
            pool.clone(),
        );
    let lot_serial_service = LotSerialServiceImpl::new(lot_serial_repo);

    let product_repo = Arc::new(ProductRepositoryImpl::new(pool.clone()));
    let product_service =
        inventory_service_infra::services::product::ProductServiceImpl::new(product_repo.clone());

    let valuation_repo = Arc::new(ValuationRepositoryImpl::new(pool.clone()));
    let valuation_service = inventory_service_infra::services::valuation::ValuationServiceImpl::new(
        valuation_repo.clone()
            as Arc<dyn inventory_service_core::repositories::valuation::ValuationRepository>,
        valuation_repo.clone()
            as Arc<dyn inventory_service_core::repositories::valuation::ValuationLayerRepository>,
        valuation_repo
            as Arc<dyn inventory_service_core::repositories::valuation::ValuationHistoryRepository>,
    );

    let warehouse_repo = WarehouseRepositoryImpl::new(pool.clone());

    // Initialize stock repositories
    let stock_move_repo = Arc::new(PgStockMoveRepository::new(Arc::new(pool.clone())));
    let inventory_level_repo = Arc::new(PgInventoryLevelRepository::new(Arc::new(pool.clone())));

    // Initialize transfer repositories and services
    let transfer_repo = Arc::new(PgTransferRepository::new(Arc::new(pool.clone())));
    let transfer_item_repo = Arc::new(PgTransferItemRepository::new(Arc::new(pool.clone())));

    let transfer_service =
        Arc::new(inventory_service_infra::services::transfer::PgTransferService::new(
            transfer_repo,
            transfer_item_repo,
            stock_move_repo.clone(),
            inventory_level_repo.clone(),
        ));

    // Initialize stock take repositories and services
    let stock_take_repo = Arc::new(PgStockTakeRepository::new(Arc::new(pool.clone())));
    let stock_take_line_repo = Arc::new(PgStockTakeLineRepository::new(Arc::new(pool.clone())));

    let stock_take_service =
        Arc::new(inventory_service_infra::services::stock_take::PgStockTakeService::new(
            Arc::new(pool.clone()),
            stock_take_repo,
            stock_take_line_repo,
            stock_move_repo.clone(),
            inventory_level_repo.clone(),
        ));

    // Initialize reconciliation repositories and services
    let reconciliation_repo =
        Arc::new(PgStockReconciliationRepository::new(Arc::new(pool.clone())));
    let reconciliation_item_repo =
        Arc::new(PgStockReconciliationItemRepository::new(Arc::new(pool.clone())));

    let reconciliation_service = Arc::new(
        inventory_service_infra::services::reconciliation::PgStockReconciliationService::new(
            Arc::new(pool.clone()),
            reconciliation_repo,
            reconciliation_item_repo,
            stock_move_repo.clone(),
            inventory_level_repo.clone(),
            product_repo.clone(),
        ),
    );

    // Initialize RMA repositories and services
    let rma_repo = Arc::new(PgRmaRepository::new(Arc::new(pool.clone())));
    let rma_item_repo = Arc::new(PgRmaItemRepository::new(Arc::new(pool.clone())));

    let rma_service = Arc::new(inventory_service_infra::services::rma::PgRmaService::new(
        rma_repo,
        rma_item_repo,
        stock_move_repo.clone(),
    ));

    // Initialize receipt repositories and services
    let receipt_repo =
        inventory_service_infra::repositories::receipt::ReceiptRepositoryImpl::new(pool.clone());

    let receipt_service = inventory_service_infra::services::receipt::ReceiptServiceImpl::new(
        Arc::new(receipt_repo),
        product_repo.clone(),
    );

    // Create application state
    let state = AppState {
        category_service: Arc::new(category_service),
        lot_serial_service: Arc::new(lot_serial_service),
        product_service: Arc::new(product_service),
        valuation_service: Arc::new(valuation_service),
        warehouse_repository: Arc::new(warehouse_repo),
        receipt_service: Arc::new(receipt_service),
        delivery_service: Arc::new(DummyDeliveryService {}),
        transfer_service,
        stock_take_service,
        reconciliation_service,
        rma_service,
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
    #[cfg(feature = "delivery")]
    let delivery_routes = create_delivery_routes(state.clone());
    let receipt_routes = create_receipt_routes(state.clone());
    let reconciliation_routes = create_reconciliation_routes(state.clone());
    let rma_routes = create_rma_routes(state.clone());
    let search_routes = create_search_routes(state.clone());
    let transfer_routes = create_transfer_routes(state.clone());
    let stock_take_routes = create_stock_take_routes(state.clone());
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
                    Err(e) => {
                        panic!("CORS configuration error: {}", e);
                    },
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

    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/health", get(crate::handlers::health::health_check))
        .layer(Extension(pool.clone()))
        .layer(Extension(config.clone()))
        .with_state(state.clone());

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        .with_state(state.clone())
        .layer(Extension(authz_state))
        .nest("/api/v1/inventory", category_routes);

    let protected_routes = protected_routes
        .nest("/api/v1/inventory/reconciliations", reconciliation_routes)
        .nest("/api/v1/inventory/receipts", receipt_routes)
        .nest("/api/v1/inventory/rma", rma_routes)
        .nest("/api/v1/inventory/products", search_routes)
        .nest("/api/v1/inventory/stock-takes", stock_take_routes)
        .nest("/api/v1/inventory/transfers", transfer_routes)
        .nest("/api/v1/inventory/valuation", valuation_routes)
        .nest("/api/v1/inventory/warehouses", warehouse_routes)
        .layer(Extension(pool))
        .layer(Extension(config.clone()))
        .layer(axum::middleware::from_fn(casbin_middleware));

    // Merge public and protected routes, apply global layers
    let mut router = Router::new().with_state(state.clone());
    router = router.merge(public_routes);
    router = router.merge(protected_routes);
    router = router.nest("/api/v1/inventory/lot-serials", create_lot_serial_routes(state.clone()));
    router = router.layer(cors);
    router
}

// function moved
