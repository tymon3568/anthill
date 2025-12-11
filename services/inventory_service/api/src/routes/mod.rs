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
use inventory_service_infra::repositories::picking_method::PickingMethodRepositoryImpl;
use inventory_service_infra::repositories::product::ProductRepositoryImpl;
use inventory_service_infra::repositories::putaway::PgPutawayRepository;
use inventory_service_infra::repositories::quality::PgQualityControlPointRepository;
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
use inventory_service_infra::services::putaway::PgPutawayService;
use inventory_service_infra::services::quality::PgQualityControlPointService;
use inventory_service_infra::services::replenishment::PgReplenishmentService;

// Local handlers/state
use crate::handlers::category::create_category_routes;
#[cfg(feature = "delivery")]
use crate::handlers::delivery::create_delivery_routes;
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

    let lot_serial_repo =
        inventory_service_infra::repositories::lot_serial::LotSerialRepositoryImpl::new(
            pool.clone(),
        );

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

    let warehouse_repo_impl = WarehouseRepositoryImpl::new(pool.clone());
    let warehouse_repo = Arc::new(warehouse_repo_impl)
        as Arc<dyn inventory_service_core::repositories::WarehouseRepository>;

    // Initialize stock repositories
    let stock_move_repo = Arc::new(PgStockMoveRepository::new(Arc::new(pool.clone())));
    let lot_serial_service =
        LotSerialServiceImpl::new(lot_serial_repo, stock_move_repo.clone(), warehouse_repo.clone());
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

    // Initialize replenishment repositories and services
    let reorder_rule_repo = Arc::new(PgReorderRuleRepository::new(pool.clone()));

    // Initialize quality repositories and services
    let qc_point_repo = Arc::new(PgQualityControlPointRepository::new(pool.clone()));
    let quality_service = Arc::new(PgQualityControlPointService::new(qc_point_repo));

    // Initialize NATS client for event publishing
    let nats_client = if let Ok(nats_url) = std::env::var("NATS_URL") {
        match timeout(Duration::from_secs(5), shared_events::NatsClient::connect(&nats_url)).await {
            Ok(Ok(client)) => Some(Arc::new(client)),
            Ok(Err(e)) => {
                tracing::warn!("Failed to connect to NATS: {}", e);
                None
            },
            Err(_) => {
                tracing::warn!("NATS connection timed out");
                None
            },
        }
    } else {
        tracing::info!("NATS_URL not set, event publishing disabled");
        None
    };

    let replenishment_service = Arc::new(PgReplenishmentService::new(
        reorder_rule_repo,
        inventory_level_repo.clone(),
        nats_client,
    ));

    // Initialize Redis URL for both idempotency and distributed locking
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

    // Initialize distributed lock service
    let distributed_lock_service = Arc::new(
        RedisDistributedLockService::new(&redis_url)
            .expect("Failed to initialize distributed lock service"),
    );

    // Initialize receipt repositories and services
    let receipt_repo =
        inventory_service_infra::repositories::receipt::ReceiptRepositoryImpl::new(pool.clone());

    let receipt_service = inventory_service_infra::services::receipt::ReceiptServiceImpl::new(
        Arc::new(receipt_repo),
        product_repo.clone(),
        distributed_lock_service.clone(),
    );

    // Initialize putaway repositories and services
    let putaway_repo = Arc::new(PgPutawayRepository::new(pool.clone()));
    let putaway_service = Arc::new(PgPutawayService::new(
        putaway_repo,
        PgStockMoveRepository::new(Arc::new(pool.clone())),
    ));

    // Initialize picking method repositories and services
    let picking_method_repo: Arc<
        dyn inventory_service_core::repositories::picking_method::PickingMethodRepository
            + Send
            + Sync,
    > = Arc::new(PickingMethodRepositoryImpl::new(pool.clone()));
    let picking_method_service = Arc::new(PickingMethodServiceImpl::new(picking_method_repo));

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

    // Create application state
    let state = AppState {
        category_service: Arc::new(category_service),
        lot_serial_service: Arc::new(lot_serial_service),
        picking_method_service,
        product_service: Arc::new(product_service),
        valuation_service: Arc::new(valuation_service),
        warehouse_repository: warehouse_repo.clone(),
        receipt_service: Arc::new(receipt_service),
        delivery_service: Arc::new(DummyDeliveryService {}),
        transfer_service,
        stock_take_service,
        reconciliation_service,
        rma_service,
        replenishment_service,
        quality_service,
        putaway_service,
        enforcer,
        jwt_secret: config.jwt_secret.clone(),
        kanidm_client: create_kanidm_client(config),
        idempotency_state: idempotency_state.clone(),
        distributed_lock_service: distributed_lock_service.clone(),
    };

    // Create AuthzState for middleware
    let authz_state = AuthzState {
        enforcer: state.enforcer.clone(),
        jwt_secret: state.jwt_secret.clone(),
        kanidm_client: state.kanidm_client.clone(),
    };

    // Create routes
    let category_routes = create_category_routes();
    #[cfg(feature = "delivery")]
    let delivery_routes = create_delivery_routes();
    let putaway_routes = create_putaway_routes();
    let receipt_routes = create_receipt_routes();
    let reconciliation_routes = create_reconciliation_routes();
    let reports_routes = create_reports_routes();
    let rma_routes = create_rma_routes();
    let search_routes = create_search_routes();
    let transfer_routes = create_transfer_routes();
    let stock_take_routes = create_stock_take_routes();
    let valuation_routes = create_valuation_routes();
    let warehouse_routes = create_warehouse_routes();

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

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        .route("/health", get(crate::handlers::health::health_check))
        .nest("/api/v1/inventory", category_routes);

    let protected_routes = protected_routes
        .nest("/api/v1/inventory/reconciliations", reconciliation_routes)
        .nest("/api/v1/inventory/receipts", receipt_routes)
        .nest("/api/v1/inventory/reports", reports_routes)
        .nest("/api/v1/inventory/rma", rma_routes)
        .nest("/api/v1/inventory/products", search_routes)
        .nest("/api/v1/inventory/stock-takes", stock_take_routes)
        .nest("/api/v1/inventory/transfers", transfer_routes)
        .nest("/api/v1/inventory/valuation", valuation_routes)
        .nest("/api/v1/inventory/warehouses", warehouse_routes)
        .nest("/api/v1/warehouse/putaway", putaway_routes)
        .nest("/api/v1/warehouse/picking", create_picking_routes())
        .nest("/api/v1/inventory/lot-serials", create_lot_serial_routes())
        .nest("/api/v1/inventory/quality", create_quality_routes())
        .nest("/api/v1/inventory/replenishment", create_replenishment_routes());

    #[cfg(feature = "delivery")]
    let protected_routes = protected_routes.nest("/api/v1/inventory/deliveries", delivery_routes);

    let protected_routes = protected_routes
        .layer(Extension(pool.clone()))
        .layer(Extension(config.clone()))
        .layer(Extension(state))
        .layer(axum::middleware::from_fn_with_state(
            idempotency_state,
            crate::middleware::idempotency_middleware,
        ))
        .layer(axum::middleware::from_fn(casbin_middleware))
        .layer(Extension(authz_state));

    // Apply global layers
    protected_routes.layer(cors)
}

// function moved
