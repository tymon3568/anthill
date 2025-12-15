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

use uuid::Uuid;

// Tokio for timeout
use tokio::time::{timeout, Duration};

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
use inventory_service_infra::services::valuation::ValuationServiceImpl;

use crate::handlers::valuation::{CostAdjustmentPayload, HistoryQueryParams, RevaluationPayload};
use inventory_service_core::domains::inventory::dto::valuation_dto::{
    ValuationDto, ValuationHistoryResponse, ValuationLayersResponse,
};
use inventory_service_core::domains::inventory::valuation::ValuationMethod;
use shared_types::Money;
use shared_types::TenantContext;

// Local handlers/state
use crate::handlers::category::create_category_routes;
#[cfg(feature = "delivery")]
use crate::handlers::delivery::create_delivery_routes;
use crate::handlers::health::{health_check, HealthResp};
use crate::handlers::lot_serial::create_lot_serial_routes;
use crate::handlers::picking::create_picking_routes;
use crate::handlers::putaway::create_putaway_routes;
use crate::handlers::receipt::create_receipt_routes;
use crate::handlers::reconciliation::create_reconciliation_routes;
use crate::handlers::rma::create_rma_routes;
use crate::handlers::search::create_search_routes;
use crate::handlers::stock_take::create_stock_take_routes;
use crate::handlers::transfer::create_transfer_routes;
use crate::handlers::valuation::create_valuation_routes;
use crate::handlers::warehouses::create_warehouse_routes;
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

use inventory_service_core::domains::inventory::dto::valuation_dto::*;
/// Dummy valuation service
use inventory_service_core::services::valuation::ValuationService;
pub struct DummyValuationService;

#[async_trait]
impl ValuationService for DummyValuationService {
    async fn get_valuation(
        &self,
        _ctx: &TenantContext,
        _product_id: Uuid,
    ) -> Result<ValuationDto, AppError> {
        Err(AppError::NotImplemented)
    }

    async fn set_valuation_method(
        &self,
        _ctx: &TenantContext,
        _product_id: Uuid,
        _method: ValuationMethod,
    ) -> Result<(), AppError> {
        Err(AppError::NotImplemented)
    }

    async fn set_standard_cost(
        &self,
        _ctx: &TenantContext,
        _product_id: Uuid,
        _cost: Money,
    ) -> Result<(), AppError> {
        Err(AppError::NotImplemented)
    }

    async fn get_valuation_layers(
        &self,
        _ctx: &TenantContext,
        _product_id: Uuid,
    ) -> Result<ValuationLayersResponse, AppError> {
        Err(AppError::NotImplemented)
    }

    async fn get_valuation_history(
        &self,
        _ctx: &TenantContext,
        _product_id: Uuid,
        _query: HistoryQueryParams,
    ) -> Result<ValuationHistoryResponse, AppError> {
        Err(AppError::NotImplemented)
    }

    async fn adjust_cost(
        &self,
        _ctx: &TenantContext,
        _product_id: Uuid,
        _adjustment: CostAdjustmentPayload,
    ) -> Result<(), AppError> {
        Err(AppError::NotImplemented)
    }

    async fn revalue_inventory(
        &self,
        _ctx: &TenantContext,
        _product_id: Uuid,
        _payload: RevaluationPayload,
    ) -> Result<(), AppError> {
        Err(AppError::NotImplemented)
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

    // DEBUG: Skip category service for now
    // let category_repo = CategoryRepositoryImpl::new(pool.clone());
    // let category_service = CategoryServiceImpl::new(category_repo);

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
    let product_repo = ProductRepositoryImpl::new(pool.clone());
    let valuation_repo = ValuationRepositoryImpl::new(pool.clone());
    let warehouse_repo = WarehouseRepositoryImpl::new(pool.clone());
    let receipt_repo = ReceiptRepositoryImpl::new(pool.clone());
    let delivery_repo = PgDeliveryOrderRepository::new(Arc::new(pool.clone()));
    let transfer_repo = PgTransferRepository::new(Arc::new(pool.clone()));
    let stock_take_repo = PgStockTakeRepository::new(Arc::new(pool.clone()));
    let reconciliation_repo = PgStockReconciliationRepository::new(Arc::new(pool.clone()));
    let rma_repo = PgRmaRepository::new(Arc::new(pool.clone()));
    let replenishment_repo = PgReorderRuleRepository::new(pool.clone());
    let quality_repo = PgQualityControlPointRepository::new(pool.clone());
    let putaway_repo = Arc::new(PgPutawayRepository::new(pool.clone()));
    let stock_move_repo = Arc::new(PgStockMoveRepository::new(Arc::new(pool.clone())));
    let inventory_level_repo = Arc::new(PgInventoryLevelRepository::new(Arc::new(pool.clone())));

    let category_service = CategoryServiceImpl::new(category_repo);
    let lot_serial_service = LotSerialServiceImpl::new(
        lot_serial_repo,
        Arc::new(stock_move_repo),
        Arc::new(warehouse_repo),
    );
    let picking_method_service = PickingMethodServiceImpl::new(Arc::new(picking_method_repo));
    let product_service = ProductServiceImpl::new(Arc::new(product_repo));
    let valuation_service = DummyValuationService;
    let distributed_lock_service = RedisDistributedLockService::new(&redis_url)
        .expect("Failed to create distributed lock service");
    let receipt_service = ReceiptServiceImpl::new(
        Arc::new(receipt_repo),
        Arc::new(product_repo),
        distributed_lock_service.clone(),
    );
    let delivery_service = DummyDeliveryService;
    let transfer_item_repo = PgTransferItemRepository::new(Arc::new(pool.clone()));
    let transfer_service = PgTransferService::new(
        Arc::new(transfer_repo),
        Arc::new(transfer_item_repo),
        stock_move_repo.clone(),
        Arc::new(inventory_level_repo),
    );
    let stock_take_line_repo = PgStockTakeLineRepository::new(Arc::new(pool.clone()));
    let stock_take_service = PgStockTakeService::new(
        Arc::new(pool.clone()),
        Arc::new(stock_take_repo),
        Arc::new(stock_take_line_repo),
        stock_move_repo.clone(),
        Arc::new(inventory_level_repo),
    );
    let reconciliation_service = PgStockReconciliationService::new(
        Arc::new(pool.clone()),
        Arc::new(reconciliation_repo),
        Arc::new(PgStockReconciliationItemRepository::new(Arc::new(pool.clone()))),
        stock_move_repo.clone(),
        Arc::new(inventory_level_repo),
        Arc::new(product_repo),
    );
    let rma_service = PgRmaService::new(
        Arc::new(rma_repo),
        Arc::new(PgRmaItemRepository::new(Arc::new(pool.clone()))),
        stock_move_repo.clone(),
    );
    let replenishment_service = PgReplenishmentService::new(
        Arc::new(replenishment_repo),
        Arc::new(inventory_level_repo),
        None,
    );
    let quality_service = PgQualityControlPointService::new(Arc::new(quality_repo));
    let putaway_service = PgPutawayService::new(putaway_repo, stock_move_repo);
    let kanidm_client = create_kanidm_client(config);

    let state = AppState {
        category_service: Arc::new(category_service),
        lot_serial_service: Arc::new(lot_serial_service),
        picking_method_service: Arc::new(picking_method_service),
        product_service: Arc::new(product_service),
        valuation_service: Arc::new(valuation_service),
        warehouse_repository: Arc::new(warehouse_repo),
        receipt_service: Arc::new(receipt_service),
        delivery_service: Arc::new(delivery_service),
        transfer_service: Arc::new(transfer_service),
        stock_take_service: Arc::new(stock_take_service),
        reconciliation_service: Arc::new(reconciliation_service),
        rma_service: Arc::new(rma_service),
        replenishment_service: Arc::new(replenishment_service),
        quality_service: Arc::new(quality_service),
        putaway_service: Arc::new(putaway_service),
        distributed_lock_service: Arc::new(distributed_lock_service),
        enforcer,
        jwt_secret: config.jwt_secret.clone(),
        kanidm_client,
        idempotency_state: idempotency_state.clone(),
    };

    // Create AuthzState for middleware
    let authz_state = AuthzState {
        enforcer: state.enforcer.clone(),
        jwt_secret: state.jwt_secret.clone(),
        kanidm_client: state.kanidm_client.clone(),
    };

    let category_routes = create_category_routes();
    let lot_serial_routes = create_lot_serial_routes();
    let picking_routes = create_picking_routes();
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

    let protected_routes = Router::new()
        .nest("/api/v1/inventory/categories", category_routes)
        .nest("/api/v1/inventory/lot-serials", lot_serial_routes)
        .nest("/api/v1/inventory/picking", picking_routes)
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
        .nest("/api/v1/inventory/reports", reports_routes)
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
        .merge(protected_routes)
        .layer(cors)
}

// function moved
