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
use chrono::Utc;
use inventory_service_core::domains::inventory::dto::transfer_dto::*;

use inventory_service_core::services::delivery::DeliveryService;
// Core service traits for UniversalDummyService
use inventory_service_core::repositories::putaway::PutawayService;
use inventory_service_core::repositories::warehouse::WarehouseRepository;
use inventory_service_core::services::distributed_lock::DistributedLockService;
use inventory_service_core::services::lot_serial::LotSerialService;
use inventory_service_core::services::picking_method::PickingMethodService;
use inventory_service_core::services::product::ProductService;
use inventory_service_core::services::quality::QualityControlPointService;
use inventory_service_core::services::receipt::ReceiptService;
use inventory_service_core::services::reconciliation::StockReconciliationService;
use inventory_service_core::services::replenishment::ReplenishmentService;
use inventory_service_core::services::rma::RmaService;
use inventory_service_core::services::stock_take::StockTakeService;
use inventory_service_core::services::transfer::TransferService;
use inventory_service_core::services::valuation::ValuationService;
use inventory_service_core::domains::inventory::dto::valuation_dto::*;
use inventory_service_core::domains::inventory::valuation::ValuationMethod;
use inventory_service_infra::services::category::CategoryServiceImpl;
use inventory_service_core::domains::replenishment::{
    CreateReorderRule, ReorderRule, ReplenishmentCheckResult, UpdateReorderRule,
};
use inventory_service_core::domains::quality::{
    CreateQualityControlPoint, QualityControlPoint, UpdateQualityControlPoint
};
use inventory_service_core::models::{
    PutawayRequest, PutawaySuggestion, ConfirmPutawayRequest, ConfirmPutawayResponse
};
// DTO Imports
use inventory_service_core::dto::receipt::*;
// use inventory_service_core::dto::transfer::*;
use inventory_service_core::dto::stock_take::*;
use inventory_service_core::dto::reconciliation::*;
use inventory_service_core::dto::rma::*;
use inventory_service_core::dto::product::*;
use inventory_service_core::domains::inventory::dto::search_dto::*;
use inventory_service_core::domains::inventory::product::Product;
use inventory_service_core::domains::inventory::picking_method::PickingMethod;
use inventory_service_core::domains::inventory::dto::picking_method_dto::*;
use inventory_service_core::models::{LotSerial, LotSerialLifecycle, LotSerialTrackingType, LotSerialStatus};
use inventory_service_core::domains::inventory::warehouse::Warehouse;
use inventory_service_core::domains::inventory::warehouse_zone::WarehouseZone;
use inventory_service_core::domains::inventory::warehouse_location::WarehouseLocation;
use inventory_service_core::domains::inventory::dto::warehouse_dto::*;
use serde_json::Value as JsonValue;

// Inventory-service infra - Repositories
use inventory_service_infra::repositories::{
    category::CategoryRepositoryImpl,

};

// Define UniversalDummyService
#[derive(Clone)]
pub struct UniversalDummyService;

#[async_trait]
impl LotSerialService for UniversalDummyService {
    async fn create_lot_serial(&self, _lot_serial: &LotSerial) -> Result<(), AppError> { unimplemented!() }
    async fn get_lot_serial(&self, _tenant_id: Uuid, _lot_serial_id: Uuid) -> Result<Option<LotSerial>, AppError> { unimplemented!() }
    async fn get_lifecycle(&self, _tenant_id: Uuid, _lot_serial_id: Uuid) -> Result<LotSerialLifecycle, AppError> { unimplemented!() }
    async fn list_lot_serials_by_product(&self, _tenant_id: Uuid, _product_id: Uuid, _tracking_type: Option<LotSerialTrackingType>, _status: Option<LotSerialStatus>) -> Result<Vec<LotSerial>, AppError> { unimplemented!() }
    async fn update_lot_serial(&self, _lot_serial: &LotSerial) -> Result<(), AppError> { unimplemented!() }
    async fn delete_lot_serial(&self, _tenant_id: Uuid, _lot_serial_id: Uuid) -> Result<(), AppError> { unimplemented!() }
    async fn quarantine_expired_lots(&self, _tenant_id: Uuid) -> Result<i64, AppError> { unimplemented!() }
}
#[async_trait]
impl PickingMethodService for UniversalDummyService {
    async fn create_method(&self, _tenant_id: Uuid, _req: CreatePickingMethodRequest, _user_id: Uuid) -> Result<PickingMethod, AppError> { unimplemented!() }
    async fn get_method(&self, _tenant_id: Uuid, _method_id: Uuid) -> Result<Option<PickingMethod>, AppError> { unimplemented!() }
    async fn get_methods_by_warehouse(&self, _tenant_id: Uuid, _warehouse_id: Uuid) -> Result<Vec<PickingMethod>, AppError> { unimplemented!() }
    async fn get_active_methods_by_warehouse(&self, _tenant_id: Uuid, _warehouse_id: Uuid) -> Result<Vec<PickingMethod>, AppError> { unimplemented!() }
    async fn get_default_method_by_warehouse(&self, _tenant_id: Uuid, _warehouse_id: Uuid) -> Result<Option<PickingMethod>, AppError> { unimplemented!() }
    async fn update_method(&self, _tenant_id: Uuid, _method_id: Uuid, _req: UpdatePickingMethodRequest, _user_id: Uuid) -> Result<PickingMethod, AppError> { unimplemented!() }
    async fn delete_method(&self, _tenant_id: Uuid, _method_id: Uuid, _user_id: Uuid) -> Result<bool, AppError> { unimplemented!() }
    async fn set_default_method(&self, _tenant_id: Uuid, _warehouse_id: Uuid, _method_id: Uuid) -> Result<bool, AppError> { unimplemented!() }
    async fn optimize_picking(&self, _tenant_id: Uuid, _req: PickingOptimizationRequest) -> Result<PickingPlanResponse, AppError> { unimplemented!() }
    async fn confirm_picking_plan(&self, _tenant_id: Uuid, _req: ConfirmPickingPlanRequest, _user_id: Uuid) -> Result<bool, AppError> { unimplemented!() }
    async fn validate_method(&self, _tenant_id: Uuid, _method_id: Uuid) -> Result<bool, AppError> { unimplemented!() }
    async fn get_method_performance(&self, _tenant_id: Uuid, _method_id: Uuid, _date_range: Option<(chrono::DateTime<Utc>, chrono::DateTime<Utc>)>) -> Result<Option<JsonValue>, AppError> { unimplemented!() }
}
#[async_trait]
impl ProductService for UniversalDummyService {
    async fn create_product(&self, _tenant_id: Uuid, _req: ProductCreateRequest) -> Result<Product, AppError> { unimplemented!() }
    async fn update_product(&self, _tenant_id: Uuid, _product_id: Uuid, _req: ProductUpdateRequest) -> Result<Product, AppError> { unimplemented!() }
    async fn delete_product(&self, _tenant_id: Uuid, _product_id: Uuid) -> Result<(), AppError> { unimplemented!() }
    async fn get_product(&self, _tenant_id: Uuid, _product_id: Uuid) -> Result<Product, AppError> { unimplemented!() }
    async fn get_product_by_sku(&self, _tenant_id: Uuid, _sku: &str) -> Result<Product, AppError> { unimplemented!() }
    async fn list_products(&self, _tenant_id: Uuid, _query: ProductListQuery) -> Result<ProductListResponse, AppError> { unimplemented!() }
    async fn get_popular_search_terms(&self, _tenant_id: Uuid, _limit: u32) -> Result<Vec<(String, u32)>, AppError> { unimplemented!() }
    async fn record_search_analytics(&self, _tenant_id: Uuid, _query: &str, _results_count: u32, _user_id: Option<Uuid>) -> Result<(), AppError> { unimplemented!() }
    async fn search_products(&self, _tenant_id: Uuid, _req: ProductSearchRequest) -> Result<ProductSearchResponse, AppError> { unimplemented!() }
    async fn get_search_suggestions(&self, _tenant_id: Uuid, _req: SearchSuggestionsRequest) -> Result<SearchSuggestionsResponse, AppError> { unimplemented!() }
}
#[async_trait]
impl WarehouseRepository for UniversalDummyService {
    async fn create(&self, _tenant_id: Uuid, _req: CreateWarehouseRequest) -> Result<Warehouse, AppError> { unimplemented!() }
    async fn find_by_id(&self, _tenant_id: Uuid, _warehouse_id: Uuid) -> Result<Option<Warehouse>, AppError> { unimplemented!() }
    async fn find_by_code(&self, _tenant_id: Uuid, _code: &str) -> Result<Option<Warehouse>, AppError> { unimplemented!() }
    async fn find_all(&self, _tenant_id: Uuid) -> Result<Vec<Warehouse>, AppError> { unimplemented!() }
    async fn get_warehouse_tree(&self, _tenant_id: Uuid) -> Result<WarehouseTreeResponse, AppError> { unimplemented!() }
    async fn update(&self, _tenant_id: Uuid, _warehouse_id: Uuid, _warehouse: &Warehouse) -> Result<Warehouse, AppError> { unimplemented!() }
    async fn delete(&self, _tenant_id: Uuid, _warehouse_id: Uuid) -> Result<bool, AppError> { unimplemented!() }
    async fn get_children(&self, _tenant_id: Uuid, _warehouse_id: Uuid) -> Result<Vec<Warehouse>, AppError> { unimplemented!() }
    async fn get_ancestors(&self, _tenant_id: Uuid, _warehouse_id: Uuid) -> Result<Vec<Warehouse>, AppError> { unimplemented!() }
    async fn get_descendants(&self, _tenant_id: Uuid, _warehouse_id: Uuid) -> Result<Vec<Warehouse>, AppError> { unimplemented!() }
    async fn get_all_zones(&self, _tenant_id: Uuid) -> Result<Vec<WarehouseZone>, AppError> { unimplemented!() }
    async fn get_all_locations(&self, _tenant_id: Uuid) -> Result<Vec<WarehouseLocation>, AppError> { unimplemented!() }
    async fn create_zone(&self, _tenant_id: Uuid, _warehouse_id: Uuid, _req: CreateWarehouseZoneRequest) -> Result<WarehouseZone, AppError> { unimplemented!() }
    async fn create_location(&self, _tenant_id: Uuid, _warehouse_id: Uuid, _req: CreateWarehouseLocationRequest) -> Result<WarehouseLocation, AppError> { unimplemented!() }
    async fn validate_hierarchy(&self, _tenant_id: Uuid, _parent_id: Uuid, _child_id: Option<Uuid>) -> Result<bool, AppError> { unimplemented!() }
    async fn get_capacity_utilization(&self, _tenant_id: Uuid, _warehouse_id: Uuid) -> Result<Option<JsonValue>, AppError> { unimplemented!() }
    async fn get_warehouse_stats(&self, _tenant_id: Uuid, _warehouse_id: Uuid) -> Result<Option<JsonValue>, AppError> { unimplemented!() }
}
#[async_trait]
impl ReceiptService for UniversalDummyService {
    async fn create_receipt(&self, _tenant_id: Uuid, _user_id: Uuid, _req: ReceiptCreateRequest) -> Result<ReceiptResponse, AppError> { unimplemented!() }
    async fn validate_receipt(&self, _tenant_id: Uuid, _receipt_id: Uuid, _warehouse_id: Uuid) -> Result<ReceiptResponse, AppError> { unimplemented!() }
    async fn validate_receipt_request(&self, _tenant_id: Uuid, _req: &ReceiptCreateRequest) -> Result<(), AppError> { unimplemented!() }
    async fn get_receipt(&self, _tenant_id: Uuid, _receipt_id: Uuid) -> Result<ReceiptResponse, AppError> { unimplemented!() }
    async fn list_receipts(&self, _tenant_id: Uuid, _query: ReceiptListQuery) -> Result<ReceiptListResponse, AppError> { unimplemented!() }
}
#[async_trait]
impl TransferService for UniversalDummyService {
    async fn create_transfer(&self, _tenant_id: Uuid, _from_warehouse: Uuid, _req: CreateTransferRequest) -> Result<CreateTransferResponse, AppError> { unimplemented!() }
    async fn confirm_transfer(&self, _tenant_id: Uuid, _transfer_id: Uuid, _user_id: Uuid, _req: ConfirmTransferRequest) -> Result<ConfirmTransferResponse, AppError> { unimplemented!() }
    async fn receive_transfer(&self, _tenant_id: Uuid, _transfer_id: Uuid, _user_id: Uuid, _req: ReceiveTransferRequest) -> Result<ReceiveTransferResponse, AppError> { unimplemented!() }
}
#[async_trait]
impl StockTakeService for UniversalDummyService {
    async fn create_stock_take(&self, _tenant_id: Uuid, _warehouse_id: Uuid, _req: CreateStockTakeRequest) -> Result<CreateStockTakeResponse, AppError> { unimplemented!() }
    async fn count_stock_take(&self, _tenant_id: Uuid, _stock_take_id: Uuid, _user_id: Uuid, _req: CountStockTakeRequest) -> Result<CountStockTakeResponse, AppError> { unimplemented!() }
    async fn finalize_stock_take(&self, _tenant_id: Uuid, _stock_take_id: Uuid, _user_id: Uuid, _req: FinalizeStockTakeRequest) -> Result<FinalizeStockTakeResponse, AppError> { unimplemented!() }
    async fn get_stock_take(&self, _tenant_id: Uuid, _stock_take_id: Uuid) -> Result<StockTakeDetailResponse, AppError> { unimplemented!() }
    async fn list_stock_takes(&self, _tenant_id: Uuid, _query: StockTakeListQuery) -> Result<StockTakeListResponse, AppError> { unimplemented!() }
}
#[async_trait]
impl StockReconciliationService for UniversalDummyService {
    async fn create_reconciliation(&self, _tenant_id: Uuid, _warehouse_id: Uuid, _req: CreateReconciliationRequest) -> Result<CreateReconciliationResponse, AppError> { unimplemented!() }
    async fn count_reconciliation(&self, _tenant_id: Uuid, _reconciliation_id: Uuid, _user_id: Uuid, _req: CountReconciliationRequest) -> Result<CountReconciliationResponse, AppError> { unimplemented!() }
    async fn finalize_reconciliation(&self, _tenant_id: Uuid, _reconciliation_id: Uuid, _user_id: Uuid, _req: FinalizeReconciliationRequest) -> Result<FinalizeReconciliationResponse, AppError> { unimplemented!() }
    async fn approve_reconciliation(&self, _tenant_id: Uuid, _reconciliation_id: Uuid, _user_id: Uuid, _req: ApproveReconciliationRequest) -> Result<ApproveReconciliationResponse, AppError> { unimplemented!() }
    async fn get_reconciliation(&self, _tenant_id: Uuid, _reconciliation_id: Uuid) -> Result<ReconciliationDetailResponse, AppError> { unimplemented!() }
    async fn list_reconciliations(&self, _tenant_id: Uuid, _query: ReconciliationListQuery) -> Result<ReconciliationListResponse, AppError> { unimplemented!() }
    async fn get_analytics(&self, _tenant_id: Uuid, _warehouse_id: Option<Uuid>) -> Result<ReconciliationAnalyticsResponse, AppError> { unimplemented!() }
    async fn get_variance_analysis(&self, _tenant_id: Uuid, _reconciliation_id: Uuid) -> Result<VarianceAnalysisResponse, AppError> { unimplemented!() }
    async fn scan_barcode(&self, _tenant_id: Uuid, _reconciliation_id: Uuid, _user_id: Uuid, _req: ScanBarcodeRequest) -> Result<ScanBarcodeResponse, AppError> { unimplemented!() }
}
#[async_trait]
impl RmaService for UniversalDummyService {
    async fn create_rma(&self, _tenant_id: Uuid, _user_id: Uuid, _req: CreateRmaRequest) -> Result<CreateRmaResponse, AppError> { unimplemented!() }
    async fn approve_rma(&self, _tenant_id: Uuid, _rma_id: Uuid, _user_id: Uuid, _req: ApproveRmaRequest) -> Result<ApproveRmaResponse, AppError> { unimplemented!() }
    async fn receive_rma(&self, _tenant_id: Uuid, _rma_id: Uuid, _user_id: Uuid, _req: ReceiveRmaRequest) -> Result<ReceiveRmaResponse, AppError> { unimplemented!() }
}
#[async_trait]
#[async_trait]
impl ReplenishmentService for UniversalDummyService {
    async fn create_reorder_rule(&self, _tenant_id: Uuid, _req: CreateReorderRule) -> Result<ReorderRule, AppError> { unimplemented!() }
    async fn get_reorder_rule(&self, _tenant_id: Uuid, _rule_id: Uuid) -> Result<Option<ReorderRule>, AppError> { unimplemented!() }
    async fn update_reorder_rule(&self, _tenant_id: Uuid, _rule_id: Uuid, _req: UpdateReorderRule) -> Result<ReorderRule, AppError> { unimplemented!() }
    async fn delete_reorder_rule(&self, _tenant_id: Uuid, _rule_id: Uuid) -> Result<(), AppError> { unimplemented!() }
    async fn list_reorder_rules_for_product(&self, _tenant_id: Uuid, _product_id: Uuid, _warehouse_id: Option<Uuid>) -> Result<Vec<ReorderRule>, AppError> { unimplemented!() }
    async fn run_replenishment_check(&self, _tenant_id: Uuid) -> Result<Vec<ReplenishmentCheckResult>, AppError> { unimplemented!() }
    async fn check_product_replenishment(&self, _tenant_id: Uuid, _product_id: Uuid, _warehouse_id: Option<Uuid>) -> Result<ReplenishmentCheckResult, AppError> { unimplemented!() }
}

#[async_trait]
impl QualityControlPointService for UniversalDummyService {
    async fn create_qc_point(&self, _tenant_id: Uuid, _req: CreateQualityControlPoint) -> Result<QualityControlPoint, AppError> { unimplemented!() }
    async fn get_qc_point(&self, _tenant_id: Uuid, _qc_id: Uuid) -> Result<Option<QualityControlPoint>, AppError> { unimplemented!() }
    async fn update_qc_point(&self, _tenant_id: Uuid, _qc_id: Uuid, _req: UpdateQualityControlPoint) -> Result<QualityControlPoint, AppError> { unimplemented!() }
    async fn delete_qc_point(&self, _tenant_id: Uuid, _qc_id: Uuid) -> Result<(), AppError> { unimplemented!() }
    async fn list_qc_points(&self, _tenant_id: Uuid) -> Result<Vec<QualityControlPoint>, AppError> { unimplemented!() }
    async fn list_active_qc_points(&self, _tenant_id: Uuid) -> Result<Vec<QualityControlPoint>, AppError> { unimplemented!() }
    async fn list_qc_points_for_product(&self, _tenant_id: Uuid, _product_id: Uuid) -> Result<Vec<QualityControlPoint>, AppError> { unimplemented!() }
    async fn list_qc_points_for_warehouse(&self, _tenant_id: Uuid, _warehouse_id: Uuid) -> Result<Vec<QualityControlPoint>, AppError> { unimplemented!() }
}

#[async_trait]
impl PutawayService for UniversalDummyService {
    async fn suggest_putaway_locations(&self, _tenant_id: &Uuid, _req: &PutawayRequest) -> Result<Vec<PutawaySuggestion>, AppError> { unimplemented!() }
    async fn confirm_putaway(&self, _tenant_id: &Uuid, _req: &ConfirmPutawayRequest, _user_id: &Uuid) -> Result<ConfirmPutawayResponse, AppError> { unimplemented!() }
    async fn validate_location_capacity(&self, _tenant_id: &Uuid, _location_id: &Uuid, _qty: i64) -> Result<bool, AppError> { unimplemented!() }
}

#[async_trait]
impl DistributedLockService for UniversalDummyService {
    async fn acquire_lock(&self, _tenant_id: Uuid, _resource: &str, _key: &str, _ttl_ms: u32) -> Result<Option<String>, AppError> { unimplemented!() }
    async fn release_lock(&self, _tenant_id: Uuid, _resource: &str, _key: &str, _token: &str) -> Result<bool, AppError> { unimplemented!() }
    async fn is_locked(&self, _tenant_id: Uuid, _resource: &str, _key: &str) -> Result<bool, AppError> { unimplemented!() }
    async fn extend_lock(&self, _tenant_id: Uuid, _resource: &str, _key: &str, _token: &str, _ttl_ms: u32) -> Result<bool, AppError> { unimplemented!() }
    async fn force_release_lock(&self, _tenant_id: Uuid, _resource: &str, _key: &str) -> Result<bool, AppError> { unimplemented!() }
}

#[async_trait]
impl ValuationService for UniversalDummyService {
    async fn get_valuation(&self, _request: GetValuationRequest) -> Result<ValuationDto, AppError> { unimplemented!() }
    async fn set_valuation_method(&self, _request: SetValuationMethodRequest) -> Result<ValuationDto, AppError> { unimplemented!() }
    async fn set_standard_cost(&self, _request: SetStandardCostRequest) -> Result<ValuationDto, AppError> { unimplemented!() }
    async fn get_valuation_layers(&self, _request: GetValuationLayersRequest) -> Result<ValuationLayersResponse, AppError> { unimplemented!() }
    async fn get_valuation_history(&self, _request: GetValuationHistoryRequest) -> Result<ValuationHistoryResponse, AppError> { unimplemented!() }
    async fn adjust_cost(&self, _request: CostAdjustmentRequest) -> Result<ValuationDto, AppError> { unimplemented!() }
    async fn revalue_inventory(&self, _request: RevaluationRequest) -> Result<ValuationDto, AppError> { unimplemented!() }
    async fn process_stock_movement(&self, _tenant_id: Uuid, _product_id: Uuid, _change: i64, _cost: Option<i64>, _user: Option<Uuid>) -> Result<ValuationDto, AppError> { unimplemented!() }
    async fn calculate_inventory_value(&self, _tenant_id: Uuid, _product_id: Uuid) -> Result<i64, AppError> { unimplemented!() }
    async fn get_valuation_method(&self, _tenant_id: Uuid, _product_id: Uuid) -> Result<ValuationMethod, AppError> { unimplemented!() }
}


// Local handlers/state
use crate::handlers::health::health_check;
use crate::openapi::ApiDoc;

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

    // NOTE: The following repository and service initialization code is temporarily commented out
    // to isolate and debug stack overflow issues during service startup. This simplified setup
    // allows the service to start with minimal dependencies while the root cause of the overflow
    // Initialize repositories
    let category_repo = CategoryRepositoryImpl::new(pool.clone());
    // let lot_serial_repo = ...
    // ... (Keep commented out lines for reference or remove)

    // Services
    let category_service = CategoryServiceImpl::new(category_repo);
    let dummy_service = Arc::new(UniversalDummyService);
    let _valuation_service = dummy_service.clone();
    let _delivery_service = DummyDeliveryService;
    let _kanidm_client = create_kanidm_client(config);

    let state = crate::state::AppState {
        category_service: Arc::new(category_service),
        lot_serial_service: dummy_service.clone(),
        picking_method_service: dummy_service.clone(),
        product_service: dummy_service.clone(),
        valuation_service: _valuation_service,
        warehouse_repository: dummy_service.clone(),
        receipt_service: dummy_service.clone(),
        delivery_service: Arc::new(_delivery_service),
        transfer_service: dummy_service.clone(),
        stock_take_service: dummy_service.clone(),
        reconciliation_service: dummy_service.clone(),
        rma_service: dummy_service.clone(),
        replenishment_service: dummy_service.clone(),
        quality_service: dummy_service.clone(),
        putaway_service: dummy_service.clone(),
        distributed_lock_service: dummy_service.clone(),
        enforcer: enforcer.clone(),
        jwt_secret: config.jwt_secret.clone(),
        kanidm_client: _kanidm_client.clone(),
        idempotency_state: idempotency_state.clone(),
    };



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
    let protected_routes = Router::new().nest("/api/v1/inventory/categories", Router::new());

    let authz_state = crate::middleware::AuthzState {
        enforcer: enforcer.clone(),
        jwt_secret: config.jwt_secret.clone(),
        kanidm_client: _kanidm_client.clone(),
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

    Router::new()
        .route("/health", get(health_check))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(protected_routes_with_layers)
        .layer(Extension(pool.clone()))
        .layer(Extension(config.clone()))
        .layer(cors)
}
