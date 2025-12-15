use utoipa::OpenApi;

// Import handlers for paths
use crate::handlers::category::{
    bulk_activate_categories, bulk_deactivate_categories, bulk_delete_categories,
    can_delete_category, create_category, delete_category, get_breadcrumbs, get_category,
    get_category_stats, get_category_tree, get_children, get_top_categories, list_categories,
    move_products_to_category, search_categories, update_category, BulkCategoryIds,
    CategoryTreeQuery, SearchQuery, TopCategoriesQuery,
};
use crate::handlers::health::HealthResp;
use crate::handlers::lot_serial::{
    create_lot_serial, delete_lot_serial, get_lot_serial, get_lot_serial_lifecycle,
    list_lot_serials_by_product, quarantine_expired_lots, update_lot_serial,
    CreateLotSerialRequest, ListLotSerialsQuery, QuarantineResponse,
};
use crate::handlers::picking::{
    confirm_picking_plan, create_picking_method, delete_picking_method, get_picking_method,
    list_picking_methods, optimize_picking, set_default_method, update_picking_method,
};
use crate::handlers::receipt::{create_receipt, get_receipt, list_receipts, validate_receipt};
use crate::handlers::search::{
    search_products, search_suggestions, ErrorResponse as SearchErrorResponse, ProductSearchQuery,
    SearchSuggestionsQuery,
};
use crate::handlers::valuation::{
    adjust_cost, get_valuation, get_valuation_history, get_valuation_layers, revalue_inventory,
    set_standard_cost, set_valuation_method, CostAdjustmentPayload,
    ErrorResponse as ValuationErrorResponse, HistoryQueryParams, RevaluationPayload,
    SetStandardCostPayload, SetValuationMethodPayload,
};

// Import DTOs for components
use inventory_service_core::domains::inventory::dto::picking_method_dto::{
    CreatePickingMethodRequest, PickingMethodResponse, UpdatePickingMethodRequest,
};
use inventory_service_core::domains::inventory::dto::search_dto::{
    ProductSearchResponse, SearchSuggestionsResponse,
};
use inventory_service_core::domains::inventory::dto::valuation_dto::{
    ValuationDto, ValuationHistoryResponse, ValuationLayersResponse,
};
use inventory_service_core::dto::category::{
    BulkOperationResponse, CategoryCreateRequest, CategoryListResponse, CategoryResponse,
    CategoryStatsResponse, CategoryTreeResponse, CategoryUpdateRequest, MoveToCategoryRequest,
};
use inventory_service_core::dto::common::PaginationInfo;
use inventory_service_core::dto::receipt::{
    ReceiptCreateRequest, ReceiptItemCreateRequest, ReceiptItemResponse, ReceiptListResponse,
    ReceiptResponse, ReceiptSummaryResponse,
};
use inventory_service_core::models::{LotSerial, LotSerialLifecycle};

/// OpenAPI documentation for Inventory Service
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::health::health_check,
        crate::handlers::category::create_category,
        crate::handlers::category::list_categories,
        crate::handlers::category::get_category_tree,
        crate::handlers::category::search_categories,
        crate::handlers::category::get_top_categories,
        crate::handlers::category::get_category,
        crate::handlers::category::update_category,
        crate::handlers::category::delete_category,
        crate::handlers::category::get_children,
        crate::handlers::category::get_breadcrumbs,
        crate::handlers::category::get_category_stats,
        crate::handlers::category::can_delete_category,
        crate::handlers::category::bulk_activate_categories,
        crate::handlers::category::bulk_deactivate_categories,
        crate::handlers::category::bulk_delete_categories,
        crate::handlers::category::move_products_to_category,
        crate::handlers::lot_serial::create_lot_serial,
        crate::handlers::lot_serial::get_lot_serial,
        crate::handlers::lot_serial::list_lot_serials_by_product,
        crate::handlers::lot_serial::update_lot_serial,
        crate::handlers::lot_serial::delete_lot_serial,
        crate::handlers::lot_serial::quarantine_expired_lots,
        crate::handlers::lot_serial::get_lot_serial_lifecycle,
        crate::handlers::picking::create_picking_method,
        crate::handlers::picking::list_picking_methods,
        crate::handlers::picking::get_picking_method,
        crate::handlers::picking::update_picking_method,
        crate::handlers::picking::delete_picking_method,
        crate::handlers::receipt::create_receipt,
        crate::handlers::receipt::list_receipts,
        crate::handlers::receipt::get_receipt,
        crate::handlers::receipt::validate_receipt,
        crate::handlers::search::search_products,
        crate::handlers::search::search_suggestions,
        crate::handlers::valuation::get_valuation,
        crate::handlers::valuation::set_valuation_method,
        crate::handlers::valuation::set_standard_cost,
        crate::handlers::valuation::get_valuation_layers,
        crate::handlers::valuation::get_valuation_history,
        crate::handlers::valuation::adjust_cost,
        crate::handlers::valuation::revalue_inventory,
    ),
    components(
        schemas(
            // Common
            HealthResp,
            PaginationInfo,
            // Category
            BulkCategoryIds,
            BulkOperationResponse,
            CategoryCreateRequest,
            CategoryListResponse,
            CategoryResponse,
            CategoryStatsResponse,
            CategoryTreeResponse,
            CategoryUpdateRequest,
            MoveToCategoryRequest,
            // Lot Serial
            CreateLotSerialRequest,
            LotSerial,
            ListLotSerialsQuery,
            LotSerialLifecycle,
            QuarantineResponse,
            // Picking Method
            CreatePickingMethodRequest,
            PickingMethodResponse,
            UpdatePickingMethodRequest,
            // Receipt
            ReceiptCreateRequest,
            ReceiptItemCreateRequest,
            ReceiptItemResponse,
            ReceiptListResponse,
            ReceiptResponse,
            ReceiptSummaryResponse,
            // Search
            SearchErrorResponse,
            ProductSearchQuery,
            ProductSearchResponse,
            SearchSuggestionsQuery,
            SearchSuggestionsResponse,
            // Valuation
            ValuationDto,
            ValuationHistoryResponse,
            ValuationLayersResponse,
            ValuationErrorResponse,
            CostAdjustmentPayload,
            HistoryQueryParams,
            RevaluationPayload,
            SetStandardCostPayload,
            SetValuationMethodPayload,
        )
    ),
    tags(
        (name = "categories", description = "Category management endpoints"),
        (name = "lot-serial", description = "Lot serial management endpoints"),
        (name = "warehouse", description = "Warehouse and picking operations"),
        (name = "receipts", description = "Goods receipt note operations"),
        (name = "search", description = "Product search and suggestions"),
        (name = "valuation", description = "Inventory valuation and costing operations"),
    ),
    info(
        title = "Inventory Service API",
        version = "0.1.0",
        description = "Multi-tenant inventory management and warehouse operations service",
        contact(
            name = "Anthill Team",
            email = "team@example.com"
        ),
        license(name = "MIT"),
    ),
    servers(
        (url = "http://localhost:8001", description = "Local development server"),
    ),
)]
pub struct ApiDoc;

/// Export OpenAPI spec to YAML file (only with --features export-spec)
#[cfg(feature = "export-spec")]
#[allow(dead_code)]
pub fn export_spec() -> Result<(), Box<dyn std::error::Error>> {
    use std::path::Path;

    let openapi = ApiDoc::openapi();
    let yaml = serde_yaml::to_string(&openapi).map_err(std::io::Error::other)?;

    let path =
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../shared/openapi/inventory.yaml"));

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, yaml)?;

    eprintln!("📄 OpenAPI spec exported to {:?}", path);
    Ok(())
}
