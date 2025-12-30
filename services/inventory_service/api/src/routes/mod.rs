//! Route definitions and router creation
//!
//! This module defines the API routes and creates the main router.
//! Following 3-crate pattern: dependency injection happens here in the API layer.

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

// Inventory-service core - DTOs and traits for delivery stub
use inventory_service_core::dto::delivery::{
    PackItemsRequest, PackItemsResponse, PickItemsRequest, PickItemsResponse, ShipItemsRequest,
    ShipItemsResponse,
};
use inventory_service_core::services::delivery::DeliveryService;

// Inventory-service infra - Repository implementations
use inventory_service_infra::repositories::{
    CategoryRepositoryImpl, LotSerialRepositoryImpl, PgInventoryLevelRepository,
    PgPutawayRepository, PgQualityControlPointRepository, PgReorderRuleRepository,
    PgRmaItemRepository, PgRmaRepository, PgStockMoveRepository,
    PgStockReconciliationItemRepository, PgStockReconciliationRepository,
    PgStockTakeLineRepository, PgStockTakeRepository, PgTransferItemRepository,
    PgTransferRepository, PickingMethodRepositoryImpl, ProductRepositoryImpl,
    ReceiptRepositoryImpl, ValuationRepositoryImpl, WarehouseRepositoryImpl,
};

// Inventory-service infra - Service implementations
use inventory_service_infra::services::{
    CategoryServiceImpl, LotSerialServiceImpl, PgPutawayService, PgQualityControlPointService,
    PgReplenishmentService, PgRmaService, PgScrapService, PgStockReconciliationService,
    PgStockTakeService, PgTransferService, PickingMethodServiceImpl, ProductServiceImpl,
    ReceiptServiceImpl, RedisDistributedLockService, ValuationServiceImpl,
};

// Local handlers
use crate::handlers::category::create_category_routes;
use crate::handlers::cycle_count::create_cycle_count_routes;
use crate::handlers::delivery::create_delivery_routes;
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
use crate::handlers::scrap::create_scrap_routes;
use crate::handlers::search::create_search_routes;
use crate::handlers::stock_take::create_stock_take_routes;
use crate::handlers::transfer::create_transfer_routes;
use crate::handlers::valuation::create_valuation_routes;
use crate::handlers::warehouses::create_warehouse_routes;
use crate::openapi::ApiDoc;

/// Create Kanidm client from configuration
fn create_kanidm_client(config: &Config) -> KanidmClient {
    // Use consistent environment detection with create_router:
    // Production if EITHER APP_ENV or RUST_ENV is "production"
    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    let rust_env = std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
    let is_dev = !(app_env == "production" || rust_env == "production");

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

/// Stub delivery service used when delivery functionality is temporarily unavailable.
///
/// This implementation consistently returns a generic "delivery service is temporarily
/// unavailable" error suitable for API consumers. Infra-specific details about the
/// underlying implementation state are logged internally rather than exposed
/// in API responses.
pub struct StubDeliveryService;

#[async_trait]
impl DeliveryService for StubDeliveryService {
    async fn pick_items(
        &self,
        _tenant_id: Uuid,
        _delivery_id: Uuid,
        _user_id: Uuid,
        _request: PickItemsRequest,
    ) -> Result<PickItemsResponse, AppError> {
        tracing::warn!("Delivery pick_items called but DeliveryServiceImpl is disabled in infra");
        Err(AppError::ServiceUnavailable(
            "Delivery service is temporarily unavailable. Please try again later.".to_string(),
        ))
    }

    async fn pack_items(
        &self,
        _tenant_id: Uuid,
        _delivery_id: Uuid,
        _user_id: Uuid,
        _request: PackItemsRequest,
    ) -> Result<PackItemsResponse, AppError> {
        tracing::warn!("Delivery pack_items called but DeliveryServiceImpl is disabled in infra");
        Err(AppError::ServiceUnavailable(
            "Delivery service is temporarily unavailable. Please try again later.".to_string(),
        ))
    }

    async fn ship_items(
        &self,
        _tenant_id: Uuid,
        _delivery_id: Uuid,
        _user_id: Uuid,
        _request: ShipItemsRequest,
    ) -> Result<ShipItemsResponse, AppError> {
        tracing::warn!("Delivery ship_items called but DeliveryServiceImpl is disabled in infra");
        Err(AppError::ServiceUnavailable(
            "Delivery service is temporarily unavailable. Please try again later.".to_string(),
        ))
    }
}

/// Create the main application router with all services wired
///
/// This function performs dependency injection following the 3-crate pattern:
/// 1. Initialize repositories from infra layer
/// 2. Initialize services with their repository dependencies
/// 3. Create AppState with all services
/// 4. Wire all route modules
pub async fn create_router(pool: PgPool, config: &Config) -> Router {
    // =========================================================================
    // Environment & Production Checks
    // =========================================================================
    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    let rust_env = std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
    let is_production = app_env == "production" || rust_env == "production";

    if is_production && config.get_cors_origins().is_empty() {
        panic!(
            "CORS_ORIGINS must be configured in production environment. \
             Set CORS_ORIGINS=https://your-domain.com,https://admin.your-domain.com"
        );
    }

    // =========================================================================
    // Initialize Casbin Enforcer
    // =========================================================================
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

    // =========================================================================
    // Initialize Redis URL
    // =========================================================================
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

    // =========================================================================
    // Initialize Idempotency State
    // =========================================================================
    let idempotency_config = crate::middleware::IdempotencyConfig {
        redis_url: redis_url.clone(),
        ttl_seconds: 24 * 60 * 60, // 24 hours
        header_name: "x-idempotency-key".to_string(),
    };
    let idempotency_state = Arc::new(
        crate::middleware::IdempotencyState::new(idempotency_config)
            .expect("Failed to initialize idempotency state"),
    );

    // =========================================================================
    // Phase 1: Initialize Base Repositories
    // =========================================================================

    // Wrap pool in Arc for repositories that need it
    let pool_arc = Arc::new(pool.clone());

    // Category
    let category_repo = CategoryRepositoryImpl::new(pool.clone());

    // Product
    let product_repo = Arc::new(ProductRepositoryImpl::new(pool.clone()));

    // Warehouse
    let warehouse_repo = Arc::new(WarehouseRepositoryImpl::new(pool.clone()));

    // Stock repositories (used by many services) - these need Arc<PgPool>
    let stock_move_repo = Arc::new(PgStockMoveRepository::new(pool_arc.clone()));
    let inventory_level_repo = Arc::new(PgInventoryLevelRepository::new(pool_arc.clone()));

    // Lot/Serial
    let lot_serial_repo = LotSerialRepositoryImpl::new(pool.clone());

    // Picking Method
    let picking_method_repo = Arc::new(PickingMethodRepositoryImpl::new(pool.clone()));

    // Receipt
    let receipt_repo = Arc::new(ReceiptRepositoryImpl::new(pool.clone()));

    // Transfer - these need Arc<PgPool>
    let transfer_repo = Arc::new(PgTransferRepository::new(pool_arc.clone()));
    let transfer_item_repo = Arc::new(PgTransferItemRepository::new(pool_arc.clone()));

    // Stock Take - these need Arc<PgPool>
    let stock_take_repo = Arc::new(PgStockTakeRepository::new(pool_arc.clone()));
    let stock_take_line_repo = Arc::new(PgStockTakeLineRepository::new(pool_arc.clone()));

    // Reconciliation - these need Arc<PgPool>
    let reconciliation_repo = Arc::new(PgStockReconciliationRepository::new(pool_arc.clone()));
    let reconciliation_item_repo =
        Arc::new(PgStockReconciliationItemRepository::new(pool_arc.clone()));

    // RMA - these need Arc<PgPool>
    let rma_repo = Arc::new(PgRmaRepository::new(pool_arc.clone()));
    let rma_item_repo = Arc::new(PgRmaItemRepository::new(pool_arc.clone()));

    // Replenishment - needs PgPool (not Arc)
    let reorder_rule_repo = Arc::new(PgReorderRuleRepository::new(pool.clone()));

    // Quality - needs PgPool (not Arc)
    let quality_repo = Arc::new(PgQualityControlPointRepository::new(pool.clone()));

    // Putaway - needs PgPool (not Arc)
    let putaway_repo = Arc::new(PgPutawayRepository::new(pool.clone()));

    // Valuation
    let valuation_repo = Arc::new(ValuationRepositoryImpl::new(pool.clone()));

    // =========================================================================
    // Phase 2: Initialize Distributed Lock Service (Redis)
    // =========================================================================
    let distributed_lock_service = Arc::new(
        RedisDistributedLockService::new(&redis_url)
            .expect("Failed to create Redis distributed lock service"),
    );

    // =========================================================================
    // Phase 3: Initialize Services with Dependencies
    // =========================================================================

    // Category Service
    let category_service = Arc::new(CategoryServiceImpl::new(category_repo));

    // Product Service
    let product_service = Arc::new(ProductServiceImpl::new(product_repo.clone()));

    // Lot/Serial Service
    let lot_serial_service = Arc::new(LotSerialServiceImpl::new(
        lot_serial_repo,
        stock_move_repo.clone(),
        warehouse_repo.clone(),
    ));

    // Picking Method Service
    let picking_method_service = Arc::new(PickingMethodServiceImpl::new(picking_method_repo));

    // Receipt Service
    let receipt_service = Arc::new(ReceiptServiceImpl::new(
        receipt_repo,
        product_repo.clone(),
        distributed_lock_service.clone(),
    ));

    // Transfer Service
    let transfer_service = Arc::new(PgTransferService::new(
        transfer_repo,
        transfer_item_repo,
        stock_move_repo.clone(),
        inventory_level_repo.clone(),
    ));

    // Stock Take Service
    let stock_take_service = Arc::new(PgStockTakeService::new(
        pool_arc.clone(),
        stock_take_repo,
        stock_take_line_repo,
        stock_move_repo.clone(),
        inventory_level_repo.clone(),
    ));

    // Cycle Counting Service
    let cycle_counting_service =
        Arc::new(inventory_service_infra::services::PgCycleCountingService::new(
            pool_arc.clone(),
            stock_move_repo.clone(),
            inventory_level_repo.clone(),
        ));

    // Reconciliation Service
    let reconciliation_service = Arc::new(PgStockReconciliationService::new(
        pool_arc.clone(),
        reconciliation_repo,
        reconciliation_item_repo,
        stock_move_repo.clone(),
        inventory_level_repo.clone(),
        product_repo.clone(),
    ));

    // RMA Service
    let rma_service = Arc::new(PgRmaService::new(rma_repo, rma_item_repo, stock_move_repo.clone()));

    // Replenishment Service
    let replenishment_service = Arc::new(PgReplenishmentService::new(
        reorder_rule_repo,
        inventory_level_repo.clone(),
        None, // NATS client - optional for now
    ));

    // Quality Service
    let quality_service = Arc::new(PgQualityControlPointService::new(quality_repo));

    // Putaway Service
    // NOTE: PgPutawayService::new takes PgStockMoveRepository by value (not Arc).
    // Since PgStockMoveRepository internally uses Arc<PgPool>, creating a new instance
    // with the same pool_arc is resource-equivalent to the existing stock_move_repo.
    // TODO: Consider updating PgPutawayService to accept Arc<PgStockMoveRepository>
    // for consistency with other services (transfer, stock_take, reconciliation, rma).
    let putaway_service = Arc::new(PgPutawayService::new(
        putaway_repo,
        PgStockMoveRepository::new(pool_arc.clone()),
    ));

    // Valuation Service
    let valuation_service = Arc::new(ValuationServiceImpl::new(
        valuation_repo.clone(),
        valuation_repo.clone(), // layer_repo
        valuation_repo,         // history_repo
    ));

    // Delivery Service (stub - implementation disabled in infra)
    let delivery_service = Arc::new(StubDeliveryService);

    // Scrap Service
    let scrap_service = Arc::new(PgScrapService::new(pool_arc.clone()));

    // Kanidm Client
    let kanidm_client = create_kanidm_client(config);

    // =========================================================================
    // Phase 4: Create AppState with All Services
    // =========================================================================
    let state = crate::state::AppState {
        category_service,
        cycle_counting_service,
        lot_serial_service,
        picking_method_service,
        product_service,
        valuation_service,
        warehouse_repository: warehouse_repo,
        receipt_service,
        delivery_service,
        transfer_service,
        stock_take_service,
        reconciliation_service,
        rma_service,
        replenishment_service,
        quality_service,
        putaway_service,
        scrap_service,
        distributed_lock_service,
        enforcer: enforcer.clone(),
        jwt_secret: config.jwt_secret.clone(),
        kanidm_client: kanidm_client.clone(),
        idempotency_state: idempotency_state.clone(),
    };

    // =========================================================================
    // Phase 5: Configure CORS
    // =========================================================================
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
            axum::http::Method::PATCH,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::HeaderName::from_static("x-idempotency-key"),
        ]);

    // =========================================================================
    // Phase 6: Wire All Routes
    // =========================================================================
    let protected_routes = Router::new()
        // Category management
        .nest("/api/v1/inventory/categories", create_category_routes())
        // Product management
        .nest("/api/v1/inventory/products", create_product_routes())
        // Warehouse management
        .nest("/api/v1/inventory/warehouses", create_warehouse_routes())
        // Lot/Serial management
        .nest("/api/v1/inventory/lots", create_lot_serial_routes())
        // Receipt (goods receipt notes)
        .nest("/api/v1/inventory/receipts", create_receipt_routes())
        // Delivery orders
        .nest("/api/v1/inventory/deliveries", create_delivery_routes())
        // Stock transfers
        .nest("/api/v1/inventory/transfers", create_transfer_routes())
        // Stock takes (physical inventory)
        .nest("/api/v1/inventory/stock-takes", create_stock_take_routes())
        // Cycle counts
        .nest("/api/v1/inventory/cycle-counts", create_cycle_count_routes())
        // Stock reconciliation
        .nest(
            "/api/v1/inventory/reconciliations",
            create_reconciliation_routes(),
        )
        // RMA (returns)
        .nest("/api/v1/inventory/rma", create_rma_routes())
        // Picking methods
        .nest("/api/v1/inventory/picking", create_picking_routes())
        // Putaway rules
        .nest("/api/v1/inventory/putaway", create_putaway_routes())
        // Valuation
        .nest("/api/v1/inventory/valuation", create_valuation_routes())
        // Quality control
        .nest("/api/v1/inventory/quality", create_quality_routes())
        // Replenishment
        .nest(
            "/api/v1/inventory/replenishment",
            create_replenishment_routes(),
        )
        // Reports
        .nest("/api/v1/inventory/reports", create_reports_routes())
        // Search
        .nest("/api/v1/inventory/search", create_search_routes())
        // Scrap management
        .nest("/api/v1/inventory/scrap", create_scrap_routes());

    // =========================================================================
    // Phase 7: Apply Middleware Layers
    // =========================================================================
    let authz_state = crate::middleware::AuthzState {
        enforcer: enforcer.clone(),
        jwt_secret: config.jwt_secret.clone(),
        kanidm_client: kanidm_client.clone(),
    };

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

    // =========================================================================
    // Phase 8: Build Final Router
    // =========================================================================
    Router::new()
        .route("/health", get(health_check))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(protected_routes_with_layers)
        .layer(Extension(pool.clone()))
        .layer(Extension(config.clone()))
        .layer(cors)
}
