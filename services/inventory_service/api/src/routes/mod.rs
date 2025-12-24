//! Route definitions and router creation
//!
//! This module defines the API routes and creates the main router.

mod quality;
mod replenishment;
mod reports;

// Standard library/external crates
use async_trait::async_trait;
use axum::{extract::Extension, http::HeaderValue, routing::get, Router};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

// Shared crates
use shared_auth::enforcer::create_enforcer;
use shared_config::Config;
use shared_error::AppError;
use shared_kanidm_client::{KanidmClient, KanidmConfig};

// Inventory-service core
use inventory_service_core::dto::delivery::{
    PackItemsRequest, PackItemsResponse, PickItemsRequest, PickItemsResponse, ShipItemsRequest,
    ShipItemsResponse,
};
use inventory_service_core::services::delivery::DeliveryService;

// Inventory-service infra - Repositories
use inventory_service_infra::repositories::{
    category::CategoryRepositoryImpl,
    lot_serial::LotSerialRepositoryImpl,
    picking_method::PickingMethodRepositoryImpl,
    product::ProductRepositoryImpl,
    putaway::PgPutawayRepository,
    quality::PgQualityControlPointRepository,
    receipt::ReceiptRepositoryImpl,
    reconciliation::{PgStockReconciliationItemRepository, PgStockReconciliationRepository},
    replenishment::PgReorderRuleRepository,
    rma::{PgRmaItemRepository, PgRmaRepository},
    stock::{PgInventoryLevelRepository, PgStockMoveRepository},
    stock_take::{PgStockTakeLineRepository, PgStockTakeRepository},
    transfer::{PgTransferItemRepository, PgTransferRepository},
    valuation::ValuationRepositoryImpl,
    warehouse::WarehouseRepositoryImpl,
};

// Inventory-service infra - Services
use inventory_service_infra::services::{
    CategoryServiceImpl,
    LotSerialServiceImpl,
    PgPutawayService,
    PgQualityControlPointService,
    // RemovalStrategyServiceImpl,
    PgReplenishmentService,
    PgRmaService,
    PgStockReconciliationService,
    PgStockTakeService,
    PgTransferService,
    PickingMethodServiceImpl,
    ProductServiceImpl,
    ReceiptServiceImpl,
    // DeliveryServiceImpl, // Not exported in infra mod.rs, using dummy
    RedisDistributedLockService,
    ValuationServiceImpl,
};

// Local handlers/state
use crate::handlers::category::create_category_routes;
use crate::handlers::health::health_check;
use crate::handlers::lot_serial::create_lot_serial_routes;
use crate::handlers::picking::create_picking_routes;
use crate::handlers::products::create_product_routes;
use crate::handlers::putaway::create_putaway_routes;
use crate::handlers::quality::create_quality_routes;
use crate::handlers::receipt::create_receipt_routes;
use crate::handlers::reconciliation::create_reconciliation_routes;
use crate::handlers::replenishment::create_replenishment_routes;
use crate::handlers::reports::create_reports_routes;
use crate::handlers::rma::create_rma_routes;
use crate::handlers::search::create_search_routes;
use crate::handlers::stock_take::create_stock_take_routes;
use crate::handlers::transfer::create_transfer_routes;
use crate::handlers::valuation::create_valuation_routes;
use crate::handlers::warehouses::create_warehouse_routes;
use crate::middleware::AuthzState;
use crate::openapi::ApiDoc;
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

/// Dummy delivery service to avoid compile errors when delivery is disabled
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

    // Initialize Redis URL for idempotency
    let redis_url = if is_production {
        config
            .redis_url
            .clone()
            .expect("REDIS_URL must be configured in production")
    } else {
        config
            .redis_url
            .clone()
            .unwrap_or_else(|| "redis://localhost:6379".to_string())
    };

    // Initialize idempotency state
    let idempotency_config = crate::middleware::IdempotencyConfig {
        redis_url: redis_url.clone(),
        ttl_seconds: 24 * 60 * 60, // 24 hours
        header_name: "x-idempotency-key".to_string(),
    };
    let idempotency_state = Arc::new(
        crate::middleware::IdempotencyState::new(idempotency_config)
            .expect("Failed to initialize idempotency state"),
    );

    // Initialize repositories
    let category_repo = CategoryRepositoryImpl::new(pool.clone());
    let lot_serial_repo = LotSerialRepositoryImpl::new(pool.clone());
    let picking_method_repo = PickingMethodRepositoryImpl::new(pool.clone());
    let product_repo = Arc::new(ProductRepositoryImpl::new(pool.clone()));
    let valuation_repo = ValuationRepositoryImpl::new(pool.clone());
    let valuation_layer_repo = ValuationRepositoryImpl::new(pool.clone());
    let valuation_history_repo = ValuationRepositoryImpl::new(pool.clone());
    let warehouse_repo = Arc::new(WarehouseRepositoryImpl::new(pool.clone()));
    let receipt_repo = ReceiptRepositoryImpl::new(pool.clone());
    // let delivery_repo = PgDeliveryOrderRepository::new(Arc::new(pool.clone()));
    let transfer_repo = PgTransferRepository::new(Arc::new(pool.clone()));
    let stock_take_repo = PgStockTakeRepository::new(Arc::new(pool.clone()));
    let reconciliation_repo = PgStockReconciliationRepository::new(Arc::new(pool.clone()));
    let rma_repo = PgRmaRepository::new(Arc::new(pool.clone()));
    let replenishment_repo = PgReorderRuleRepository::new(pool.clone());
    let quality_repo = PgQualityControlPointRepository::new(pool.clone());
    let putaway_repo = Arc::new(PgPutawayRepository::new(pool.clone()));

    // Additional repos for services
    let transfer_item_repo = PgTransferItemRepository::new(Arc::new(pool.clone()));
    let stock_take_line_repo = PgStockTakeLineRepository::new(Arc::new(pool.clone()));
    let reconciliation_item_repo = PgStockReconciliationItemRepository::new(Arc::new(pool.clone()));
    let rma_item_repo = PgRmaItemRepository::new(Arc::new(pool.clone()));
    let stock_move_repo = Arc::new(PgStockMoveRepository::new(Arc::new(pool.clone())));
    let inventory_level_repo = Arc::new(PgInventoryLevelRepository::new(Arc::new(pool.clone())));

    // Initialize Services
    let distributed_lock_service = RedisDistributedLockService::new(&redis_url)
        .expect("Failed to initialize distributed lock service");
    let distributed_lock_service_arc = Arc::new(distributed_lock_service);

    let category_service = CategoryServiceImpl::new(category_repo);
    let lot_serial_service =
        LotSerialServiceImpl::new(lot_serial_repo, stock_move_repo.clone(), warehouse_repo.clone());
    let picking_method_service = PickingMethodServiceImpl::new(Arc::new(picking_method_repo));
    let product_service = ProductServiceImpl::new(product_repo.clone());
    let valuation_service = ValuationServiceImpl::new(
        Arc::new(valuation_repo),
        Arc::new(valuation_layer_repo),
        Arc::new(valuation_history_repo),
    );

    let receipt_service = ReceiptServiceImpl::new(
        Arc::new(receipt_repo),
        product_repo.clone(),
        distributed_lock_service_arc.clone(),
    );

    let delivery_service = DummyDeliveryService;

    let transfer_service = PgTransferService::new(
        Arc::new(transfer_repo),
        Arc::new(transfer_item_repo),
        stock_move_repo.clone(),
        inventory_level_repo.clone(),
    );

    let stock_take_service = PgStockTakeService::new(
        Arc::new(pool.clone()),
        Arc::new(stock_take_repo),
        Arc::new(stock_take_line_repo),
        stock_move_repo.clone(),
        inventory_level_repo.clone(),
    );

    let reconciliation_service = PgStockReconciliationService::new(
        Arc::new(pool.clone()),
        Arc::new(reconciliation_repo),
        Arc::new(reconciliation_item_repo),
        stock_move_repo.clone(),
        inventory_level_repo.clone(),
        product_repo.clone(),
    );

    let rma_service =
        PgRmaService::new(Arc::new(rma_repo), Arc::new(rma_item_repo), stock_move_repo.clone());

    let replenishment_service = PgReplenishmentService::new(
        Arc::new(replenishment_repo),
        inventory_level_repo.clone(),
        None, // Nats client option
    );

    let quality_service = PgQualityControlPointService::new(Arc::new(quality_repo));

    // Create new PgStockMoveRepository for putaway service as it might require value or doesn't impl Clone
    let stock_move_repo_for_putaway = PgStockMoveRepository::new(Arc::new(pool.clone()));
    let putaway_service = PgPutawayService::new(putaway_repo, stock_move_repo_for_putaway);

    let kanidm_client = create_kanidm_client(config);

    let state = AppState {
        category_service: Arc::new(category_service),
        lot_serial_service: Arc::new(lot_serial_service),
        picking_method_service: Arc::new(picking_method_service),
        product_service: Arc::new(product_service),
        valuation_service: Arc::new(valuation_service),
        warehouse_repository: warehouse_repo,
        receipt_service: Arc::new(receipt_service),
        delivery_service: Arc::new(delivery_service),
        transfer_service: Arc::new(transfer_service),
        stock_take_service: Arc::new(stock_take_service),
        reconciliation_service: Arc::new(reconciliation_service),
        rma_service: Arc::new(rma_service),
        replenishment_service: Arc::new(replenishment_service),
        quality_service: Arc::new(quality_service),
        putaway_service: Arc::new(putaway_service),
        distributed_lock_service: distributed_lock_service_arc,
        enforcer: enforcer.clone(),
        jwt_secret: config.jwt_secret.clone(),
        kanidm_client: kanidm_client.clone(),
        idempotency_state: idempotency_state.clone(),
    };

    let authz_state = AuthzState {
        enforcer: state.enforcer.clone(),
        jwt_secret: state.jwt_secret.clone(),
        kanidm_client: state.kanidm_client.clone(),
    };

    // Route creations
    let category_routes = create_category_routes();
    let lot_serial_routes = create_lot_serial_routes();
    let picking_routes = create_picking_routes();
    let product_routes = create_product_routes();
    let putaway_routes = create_putaway_routes();
    let receipt_routes = create_receipt_routes();
    let reconciliation_routes = create_reconciliation_routes();
    let rma_routes = create_rma_routes();
    let search_routes = create_search_routes();
    let stock_take_routes = create_stock_take_routes();
    let transfer_routes = create_transfer_routes();
    let valuation_routes = create_valuation_routes();
    let warehouse_routes = create_warehouse_routes();
    let quality_routes = create_quality_routes();
    let replenishment_routes = create_replenishment_routes();
    let reports_routes = create_reports_routes();

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

    // Protected routes
    let protected_routes = Router::new()
        .nest("/api/v1/inventory/categories", category_routes)
        .nest("/api/v1/inventory/lot-serials", lot_serial_routes)
        .nest("/api/v1/inventory/picking", picking_routes)
        .nest("/api/v1/inventory/products", product_routes)
        .nest("/api/v1/inventory/putaway", putaway_routes)
        .nest("/api/v1/inventory/receipts", receipt_routes)
        .nest("/api/v1/inventory/reconciliation", reconciliation_routes)
        .nest("/api/v1/inventory/rma", rma_routes)
        .nest("/api/v1/inventory/search", search_routes)
        .nest("/api/v1/inventory/stock-take", stock_take_routes)
        .nest("/api/v1/inventory/transfers", transfer_routes)
        .nest("/api/v1/inventory/valuation", valuation_routes)
        .nest("/api/v1/inventory/warehouses", warehouse_routes)
        .nest("/api/v1/inventory/quality", quality_routes)
        .nest("/api/v1/inventory/replenishment", replenishment_routes)
        .nest("/api/v1/inventory/reports", reports_routes);

    let protected_routes_with_layers = protected_routes
        .layer(Extension(pool.clone()))
        .layer(Extension(config.clone()))
        .layer(Extension(state))
        .layer(axum::middleware::from_fn_with_state(
            idempotency_state,
            crate::middleware::idempotency_middleware,
        ))
        .layer(axum::middleware::from_fn(crate::middleware::casbin_middleware))
        .layer(Extension(authz_state));

    Router::new()
        .route("/health", get(health_check))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(protected_routes_with_layers)
        .layer(Extension(pool.clone()))
        .layer(Extension(config.clone()))
        .layer(cors)
}
