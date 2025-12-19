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

// Tokio for timeout
// Removed unused tokio imports

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
use inventory_service_core::repositories::{InventoryLevelRepository, StockMoveRepository};
use inventory_service_core::services::delivery::DeliveryService;

// Inventory-service infra

use inventory_service_infra::repositories::category::CategoryRepositoryImpl;
use inventory_service_infra::repositories::lot_serial::LotSerialRepositoryImpl;

use inventory_service_infra::repositories::delivery_order::PgDeliveryOrderRepository;
use inventory_service_infra::repositories::picking_method::PickingMethodRepositoryImpl;
use inventory_service_infra::repositories::product::ProductRepositoryImpl;
use inventory_service_infra::repositories::putaway::PgPutawayRepository;
use inventory_service_infra::repositories::quality::PgQualityControlPointRepository;
use inventory_service_infra::repositories::receipt::ReceiptRepositoryImpl;
use inventory_service_infra::repositories::reconciliation::{
    PgStockReconciliationItemRepository, PgStockReconciliationRepository,
};
use inventory_service_infra::repositories::replenishment::PgReorderRuleRepository;
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
use inventory_service_infra::services::distributed_lock::RedisDistributedLockService;
use inventory_service_infra::services::lot_serial::LotSerialServiceImpl;
use inventory_service_infra::services::picking_method::PickingMethodServiceImpl;
use inventory_service_infra::services::product::ProductServiceImpl;
use inventory_service_infra::services::putaway::PgPutawayService;
use inventory_service_infra::services::quality::PgQualityControlPointService;
use inventory_service_infra::services::receipt::ReceiptServiceImpl;
use inventory_service_infra::services::reconciliation::PgStockReconciliationService;
use inventory_service_infra::services::replenishment::PgReplenishmentService;
use inventory_service_infra::services::rma::PgRmaService;
use inventory_service_infra::services::stock_take::PgStockTakeService;
use inventory_service_infra::services::transfer::PgTransferService;
// Removed unused ValuationServiceImpl import

// Removed unused valuation payload imports
use inventory_service_core::domains::inventory::dto::valuation_dto::{
    ValuationDto, ValuationHistoryResponse, ValuationLayersResponse,
};
use inventory_service_core::domains::inventory::valuation::ValuationMethod;
// Removed unused Money import
// Removed unused TenantContext import

// Local handlers/state
use crate::handlers::category::create_category_routes;
#[cfg(feature = "delivery")]
use crate::handlers::delivery::create_delivery_routes;
use crate::handlers::health::health_check;
use crate::handlers::lot_serial::create_lot_serial_routes;
use crate::handlers::picking::create_picking_routes;
use crate::handlers::products::create_product_routes;
use crate::handlers::putaway::create_putaway_routes;
use crate::handlers::receipt::create_receipt_routes;
use crate::handlers::reconciliation::create_reconciliation_routes;
use crate::handlers::rma::create_rma_routes;
use crate::handlers::search::create_search_routes;
use crate::handlers::stock_take::create_stock_take_routes;
use crate::handlers::transfer::create_transfer_routes;
use crate::handlers::valuation::create_valuation_routes;
use crate::handlers::warehouses::create_warehouse_routes;
use crate::openapi::ApiDoc;
use crate::routes::quality::create_quality_routes;
use crate::routes::replenishment::create_replenishment_routes;
use crate::routes::reports::create_reports_routes;
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

pub struct SimpleDummyValuationService;
#[async_trait]
impl inventory_service_core::services::valuation::ValuationService for SimpleDummyValuationService {
    async fn get_valuation(
        &self,
        _request: inventory_service_core::domains::inventory::dto::valuation_dto::GetValuationRequest,
    ) -> Result<
        inventory_service_core::domains::inventory::dto::valuation_dto::ValuationDto,
        shared_error::AppError,
    > {
        Err(shared_error::AppError::ServiceUnavailable("Not implemented".to_string()))
    }

    async fn set_valuation_method(
        &self,
        _request: inventory_service_core::domains::inventory::dto::valuation_dto::SetValuationMethodRequest,
    ) -> Result<
        inventory_service_core::domains::inventory::dto::valuation_dto::ValuationDto,
        shared_error::AppError,
    > {
        Err(shared_error::AppError::ServiceUnavailable("Not implemented".to_string()))
    }

    async fn set_standard_cost(
        &self,
        _request: inventory_service_core::domains::inventory::dto::valuation_dto::SetStandardCostRequest,
    ) -> Result<
        inventory_service_core::domains::inventory::dto::valuation_dto::ValuationDto,
        shared_error::AppError,
    > {
        Err(shared_error::AppError::ServiceUnavailable("Not implemented".to_string()))
    }

    async fn get_valuation_layers(
        &self,
        _request: inventory_service_core::domains::inventory::dto::valuation_dto::GetValuationLayersRequest,
    ) -> Result<
        inventory_service_core::domains::inventory::dto::valuation_dto::ValuationLayersResponse,
        shared_error::AppError,
    > {
        Err(shared_error::AppError::ServiceUnavailable("Not implemented".to_string()))
    }

    async fn get_valuation_history(
        &self,
        _request: inventory_service_core::domains::inventory::dto::valuation_dto::GetValuationHistoryRequest,
    ) -> Result<
        inventory_service_core::domains::inventory::dto::valuation_dto::ValuationHistoryResponse,
        shared_error::AppError,
    > {
        Err(shared_error::AppError::ServiceUnavailable("Not implemented".to_string()))
    }

    async fn adjust_cost(
        &self,
        _request: inventory_service_core::domains::inventory::dto::valuation_dto::CostAdjustmentRequest,
    ) -> Result<
        inventory_service_core::domains::inventory::dto::valuation_dto::ValuationDto,
        shared_error::AppError,
    > {
        Err(shared_error::AppError::ServiceUnavailable("Not implemented".to_string()))
    }

    async fn revalue_inventory(
        &self,
        _request: inventory_service_core::domains::inventory::dto::valuation_dto::RevaluationRequest,
    ) -> Result<
        inventory_service_core::domains::inventory::dto::valuation_dto::ValuationDto,
        shared_error::AppError,
    > {
        Err(shared_error::AppError::ServiceUnavailable("Not implemented".to_string()))
    }

    async fn process_stock_movement(
        &self,
        _tenant_id: uuid::Uuid,
        _product_id: uuid::Uuid,
        _quantity_change: i64,
        _unit_cost: Option<i64>,
        _user_id: Option<uuid::Uuid>,
    ) -> Result<
        inventory_service_core::domains::inventory::dto::valuation_dto::ValuationDto,
        shared_error::AppError,
    > {
        Err(shared_error::AppError::ServiceUnavailable("Not implemented".to_string()))
    }

    async fn calculate_inventory_value(
        &self,
        _tenant_id: uuid::Uuid,
        _product_id: uuid::Uuid,
    ) -> Result<i64, shared_error::AppError> {
        Err(shared_error::AppError::ServiceUnavailable("Not implemented".to_string()))
    }

    async fn get_valuation_method(
        &self,
        _tenant_id: uuid::Uuid,
        _product_id: uuid::Uuid,
    ) -> Result<
        inventory_service_core::domains::inventory::valuation::ValuationMethod,
        shared_error::AppError,
    > {
        Err(shared_error::AppError::ServiceUnavailable("Not implemented".to_string()))
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

    // NOTE: The following repository and service initialization code is temporarily commented out
    // to isolate and debug stack overflow issues during service startup. This simplified setup
    // allows the service to start with minimal dependencies while the root cause of the overflow
    // is investigated. TODO: Re-enable full initialization once stack overflow is resolved.
    // Initialize repositories - COMMENTED OUT MOST TO ISOLATE STACK OVERFLOW
    // let category_repo = CategoryRepositoryImpl::new(pool.clone());
    // let lot_serial_repo = LotSerialRepositoryImpl::new(pool.clone());
    // let picking_method_repo = PickingMethodRepositoryImpl::new(pool.clone());
    // let product_repo = Arc::new(ProductRepositoryImpl::new(pool.clone()));
    // let _valuation_repo = ValuationRepositoryImpl::new(pool.clone());
    // let warehouse_repo = Arc::new(WarehouseRepositoryImpl::new(pool.clone()));
    // let receipt_repo = ReceiptRepositoryImpl::new(pool.clone());
    // let _delivery_repo = PgDeliveryOrderRepository::new(Arc::new(pool.clone()));
    // let transfer_repo = PgTransferRepository::new(Arc::new(pool.clone()));
    // let stock_take_repo = PgStockTakeRepository::new(Arc::new(pool.clone()));
    // let reconciliation_repo = PgStockReconciliationRepository::new(Arc::new(pool.clone()));
    // let rma_repo = PgRmaRepository::new(Arc::new(pool.clone()));
    // let replenishment_repo = PgReorderRuleRepository::new(pool.clone());
    // let quality_repo = PgQualityControlPointRepository::new(pool.clone());
    // let putaway_repo = Arc::new(PgPutawayRepository::new(pool.clone()));
    // let stock_move_repo = Arc::new(PgStockMoveRepository::new(Arc::new(pool.clone())));
    // let inventory_level_repo = Arc::new(PgInventoryLevelRepository::new(Arc::new(pool.clone())));

    // Keep only basic category service for testing
    let category_repo = CategoryRepositoryImpl::new(pool.clone());
    let category_service = CategoryServiceImpl::new(category_repo);

    // Comment out complex services
    // let lot_serial_service = ... (commented out)
    // let picking_method_service = ... (commented out)
    // let product_service = ... (commented out)
    let valuation_service = Arc::new(SimpleDummyValuationService);
    // let distributed_lock_service = ... (commented out)
    // let receipt_service = ... (commented out)
    let delivery_service = DummyDeliveryService;
    // let transfer_service = ... (commented out)
    // let stock_take_service = ... (commented out)
    // let reconciliation_service = ... (commented out)
    // let rma_service = ... (commented out)
    // let replenishment_service = ... (commented out)
    // let quality_service = ... (commented out)
    // let putaway_service = ... (commented out)
    let kanidm_client = create_kanidm_client(config);

    // Comment out AppState creation to isolate stack overflow
    let state = AppState {
        category_service: Arc::new(category_service),
        lot_serial_service: Arc::new(DummyLotSerialService2),
        picking_method_service: Arc::new(DummyPickingMethodService2),
        product_service: Arc::new(DummyProductService2),
        valuation_service: Arc::new(SimpleDummyValuationService),
        warehouse_repository: Arc::new(DummyWarehouseRepository2),
        receipt_service: Arc::new(DummyReceiptService2),
        delivery_service: Arc::new(delivery_service),
        transfer_service: Arc::new(DummyTransferService2),
        stock_take_service: Arc::new(DummyStockTakeService2),
        reconciliation_service: Arc::new(DummyReconciliationService2),
        rma_service: Arc::new(DummyRmaService2),
        replenishment_service: Arc::new(DummyReplenishmentService2),
        quality_service: Arc::new(DummyQualityService2),
        putaway_service: Arc::new(DummyPutawayService2),
        distributed_lock_service: Arc::new(DummyDistributedLockService2),
        enforcer,
        jwt_secret: config.jwt_secret.clone(),
        kanidm_client,
        idempotency_state: idempotency_state.clone(),
    };

    // Comment out AuthzState creation
    let authz_state = AuthzState {
        enforcer: state.enforcer.clone(),
        jwt_secret: state.jwt_secret.clone(),
        kanidm_client: state.kanidm_client.clone(),
    };

    // Comment out route creations to isolate stack overflow
    let category_routes = create_category_routes();
    // let lot_serial_routes = create_lot_serial_routes();
    // let picking_routes = create_picking_routes();
    // let product_routes = create_product_routes();
    // let putaway_routes = create_putaway_routes();
    // let receipt_routes = create_receipt_routes();
    // let reconciliation_routes = create_reconciliation_routes();
    // let rma_routes = create_rma_routes();
    // let search_routes = create_search_routes();
    // let stock_take_routes = create_stock_take_routes();
    // let transfer_routes = create_transfer_routes();
    // let valuation_routes = create_valuation_routes();
    // let warehouse_routes = create_warehouse_routes();
    // let quality_routes = create_quality_routes();
    // let replenishment_routes = create_replenishment_routes();
    // let reports_routes = create_reports_routes();

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

    // Comment out protected routes to isolate stack overflow
    let protected_routes = Router::new().nest("/api/v1/inventory/categories", category_routes);
    //     .nest("/api/v1/inventory/lot-serials", lot_serial_routes)
    //     .nest("/api/v1/inventory/picking", picking_routes)
    //     .nest("/api/v1/inventory/products", product_routes)
    //     .nest("/api/v1/inventory/putaway", putaway_routes)
    //     .nest("/api/v1/inventory/receipts", receipt_routes)
    //     .nest("/api/v1/inventory/reconciliation", reconciliation_routes)
    //     .nest("/api/v1/inventory/rma", rma_routes)
    //     .nest("/api/v1/inventory/search", search_routes)
    //     .nest("/api/v1/inventory/stock-take", stock_take_routes)
    //     .nest("/api/v1/inventory/transfers", transfer_routes)
    //     .nest("/api/v1/inventory/valuation", valuation_routes)
    //     .nest("/api/v1/inventory/warehouses", warehouse_routes)
    //     .nest("/api/v1/inventory/quality", quality_routes)
    //     .nest("/api/v1/inventory/replenishment", replenishment_routes)
    //     .nest("/api/v1/inventory/reports", reports_routes);

    // Comment out protected routes with layers
    let protected_routes_with_layers = protected_routes
        .layer(Extension(pool.clone()))
        .layer(Extension(config.clone()))
        .layer(Extension(state))
        .layer(axum::middleware::from_fn_with_state(
            idempotency_state,
            crate::middleware::idempotency_middleware,
        ))
        .layer(axum::middleware::from_fn(casbin_middleware))
        .layer(Extension(authz_state));

    Router::new()
        .route("/health", get(health_check))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(protected_routes_with_layers)
        .layer(Extension(pool.clone()))
        .layer(Extension(config.clone()))
        .layer(cors)
}
