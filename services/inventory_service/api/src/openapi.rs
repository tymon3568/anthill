#[allow(unused_imports)]
use utoipa::OpenApi;

// Import handlers for paths
#[allow(unused_imports)]
use crate::handlers::category::{
    bulk_activate_categories, bulk_deactivate_categories, bulk_delete_categories,
    can_delete_category, create_category, delete_category, get_breadcrumbs, get_category,
    get_category_stats, get_category_tree, get_children, get_top_categories, list_categories,
    move_products_to_category, search_categories, update_category, BulkCategoryIds,
    CategoryTreeQuery, SearchQuery, TopCategoriesQuery,
};
#[allow(unused_imports)]
use crate::handlers::health::HealthResp;
#[allow(unused_imports)]
use crate::handlers::lot_serial::{
    create_lot_serial, delete_lot_serial, get_lot_serial, get_lot_serial_lifecycle,
    list_lot_serials_by_product, quarantine_expired_lots, update_lot_serial,
    CreateLotSerialRequest, ListLotSerialsQuery, QuarantineResponse,
};
#[allow(unused_imports)]
use crate::handlers::picking::{
    confirm_picking_plan, create_picking_method, delete_picking_method, get_picking_method,
    list_picking_methods, optimize_picking, set_default_method, update_picking_method,
};
#[allow(unused_imports)]
use crate::handlers::products::{
    create_product, delete_product, get_product, list_products, update_product,
};
#[allow(unused_imports)]
use crate::handlers::putaway::{confirm_putaway, suggest_putaway};
#[allow(unused_imports)]
use crate::handlers::quality::{
    create_qc_point, delete_qc_point, get_qc_point, list_qc_points, update_qc_point,
};
#[allow(unused_imports)]
use crate::handlers::receipt::{create_receipt, get_receipt, list_receipts, validate_receipt};
#[allow(unused_imports)]
use crate::handlers::reconciliation::{
    approve_reconciliation, count_reconciliation, create_reconciliation, finalize_reconciliation,
    get_reconciliation, get_reconciliation_analytics, get_variance_analysis, list_reconciliations,
    scan_barcode,
};
#[allow(unused_imports)]
use crate::handlers::replenishment::{
    check_product_replenishment, create_reorder_rule, delete_reorder_rule, get_reorder_rule,
    list_reorder_rules_for_product, run_replenishment_check, update_reorder_rule,
};
#[allow(unused_imports)]
use crate::handlers::reports::{
    get_dead_stock, get_inventory_turnover, get_low_stock, get_stock_aging, get_stock_ledger,
};
#[allow(unused_imports)]
use crate::handlers::rma::{approve_rma, create_rma, receive_rma};
#[allow(unused_imports)]
use crate::handlers::search::{
    search_products, search_suggestions, ErrorResponse as SearchErrorResponse, ProductSearchQuery,
    SearchSuggestionsQuery,
};
#[allow(unused_imports)]
use crate::handlers::stock_take::{
    count_stock_take, create_stock_take, finalize_stock_take, get_stock_take, list_stock_takes,
};
#[allow(unused_imports)]
use crate::handlers::transfer::{confirm_transfer, create_transfer, receive_transfer};
#[allow(unused_imports)]
use crate::handlers::valuation::{
    adjust_cost, get_valuation, get_valuation_history, get_valuation_layers, revalue_inventory,
    set_standard_cost, set_valuation_method, CostAdjustmentPayload,
    ErrorResponse as ValuationErrorResponse, HistoryQueryParams, RevaluationPayload,
    SetStandardCostPayload, SetValuationMethodPayload,
};
#[allow(unused_imports)]
use crate::handlers::warehouses::{
    create_location, create_warehouse, create_zone, delete_warehouse, get_warehouse,
    get_warehouse_tree, get_warehouses, update_warehouse, ErrorResponse as WarehouseErrorResponse,
};

// Import DTOs for components
use inventory_service_core::domains::inventory::dto::picking_method_dto::{
    CreatePickingMethodRequest, PickingMethodResponse, UpdatePickingMethodRequest,
};
use inventory_service_core::domains::inventory::dto::search_dto::{
    ProductSearchResponse, SearchSuggestionsResponse,
};
use inventory_service_core::domains::inventory::dto::transfer_dto::{
    ConfirmTransferRequest, ConfirmTransferResponse, CreateTransferRequest, CreateTransferResponse,
    ReceiveTransferRequest, ReceiveTransferResponse,
};
use inventory_service_core::domains::inventory::dto::valuation_dto::{
    ValuationDto, ValuationHistoryResponse, ValuationLayersResponse,
};
use inventory_service_core::domains::inventory::dto::warehouse_dto::{
    CreateWarehouseLocationRequest, CreateWarehouseRequest, CreateWarehouseZoneRequest,
    WarehouseLocationResponse, WarehouseResponse, WarehouseTreeResponse, WarehouseZoneResponse,
};
use inventory_service_core::domains::quality::{
    CreateQualityControlPoint, QualityControlPoint, UpdateQualityControlPoint,
};
use inventory_service_core::domains::replenishment::{
    CreateReorderRule, ReplenishmentCheckResult, UpdateReorderRule,
};
use inventory_service_core::dto::category::{
    BulkOperationResponse, CategoryCreateRequest, CategoryListResponse, CategoryResponse,
    CategoryStatsResponse, CategoryUpdateRequest, MoveToCategoryRequest,
};
use inventory_service_core::dto::common::PaginationInfo;
use inventory_service_core::dto::product::{
    ProductCreateRequest, ProductListQuery, ProductListResponse, ProductResponse,
    ProductUpdateRequest,
};
use inventory_service_core::dto::receipt::{
    ReceiptCreateRequest, ReceiptItemCreateRequest, ReceiptItemResponse, ReceiptListResponse,
    ReceiptResponse, ReceiptSummaryResponse,
};
use inventory_service_core::dto::reconciliation::{
    ApproveReconciliationRequest, ApproveReconciliationResponse, CountReconciliationRequest,
    CountReconciliationResponse, CreateReconciliationRequest, CreateReconciliationResponse,
    FinalizeReconciliationRequest, FinalizeReconciliationResponse, ReconciliationAnalyticsQuery,
    ReconciliationAnalyticsResponse, ReconciliationDetailResponse, ReconciliationListQuery,
    ReconciliationListResponse, ScanBarcodeRequest, ScanBarcodeResponse, VarianceAnalysisResponse,
};
// Reports DTOs are defined in handlers/reports.rs
use crate::handlers::reports::{
    DeadStockEntry, DeadStockQuery, InventoryTurnoverEntry, InventoryTurnoverQuery, LowStockEntry,
    LowStockQuery, StockAgingEntry, StockAgingQuery, StockLedgerEntry, StockLedgerQuery,
};
use inventory_service_core::dto::rma::{
    ApproveRmaRequest, ApproveRmaResponse, CreateRmaRequest, CreateRmaResponse, ReceiveRmaRequest,
    ReceiveRmaResponse,
};
use inventory_service_core::dto::stock_take::{
    CountStockTakeRequest, CountStockTakeResponse, CreateStockTakeRequest, CreateStockTakeResponse,
    FinalizeStockTakeRequest, FinalizeStockTakeResponse, StockTakeDetailResponse,
    StockTakeListQuery, StockTakeListResponse,
};
use inventory_service_core::models::{
    ConfirmPutawayRequest, ConfirmPutawayResponse, LotSerial, LotSerialLifecycle, PutawayRequest,
    PutawayResponse, PutawaySuggestion,
};

// Health OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::health::health_check,
    ),
    components(
        schemas(HealthResp)
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
    ),
)]
pub struct HealthApiDoc;

// Categories CRUD operations
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::category::create_category,
        crate::handlers::category::get_category,
        crate::handlers::category::update_category,
        crate::handlers::category::delete_category,
    ),
    components(schemas(
        CategoryCreateRequest,
        CategoryUpdateRequest,
        CategoryResponse
    )),
    tags((name = "categories", description = "Category management endpoints")),
)]
pub struct CategoriesCrudApiDoc;

// Categories listing operations
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::category::list_categories,
        crate::handlers::category::get_category_tree,
        crate::handlers::category::search_categories,
        crate::handlers::category::get_top_categories,
    ),
    components(schemas(CategoryListResponse))
)]
pub struct CategoriesListingApiDoc;

// Categories hierarchy operations
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::category::get_children,
        crate::handlers::category::get_breadcrumbs,
        crate::handlers::category::get_category_stats,
        crate::handlers::category::can_delete_category,
    ),
    components(schemas(
        CategoryStatsResponse,
        inventory_service_core::domains::category::CategoryBreadcrumb
    ))
)]
pub struct CategoriesHierarchyApiDoc;

// Categories bulk operations
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::category::bulk_activate_categories,
        crate::handlers::category::bulk_deactivate_categories,
        crate::handlers::category::bulk_delete_categories,
        crate::handlers::category::move_products_to_category,
    ),
    components(schemas(BulkCategoryIds, BulkOperationResponse, MoveToCategoryRequest))
)]
pub struct CategoriesBulkApiDoc;

// Lot Serial OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::lot_serial::create_lot_serial,
        crate::handlers::lot_serial::get_lot_serial,
        crate::handlers::lot_serial::update_lot_serial,
        crate::handlers::lot_serial::delete_lot_serial,
        crate::handlers::lot_serial::list_lot_serials_by_product,
        crate::handlers::lot_serial::get_lot_serial_lifecycle,
        crate::handlers::lot_serial::quarantine_expired_lots,
    ),
    components(
        schemas(
            LotSerial,
            LotSerialLifecycle,
            CreateLotSerialRequest,
            ListLotSerialsQuery,
            QuarantineResponse,
        )
    ),
    tags(
        (name = "lot-serial", description = "Lot serial management endpoints"),
    ),
)]
pub struct LotSerialApiDoc;

// Picking OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::picking::create_picking_method,
        crate::handlers::picking::get_picking_method,
        crate::handlers::picking::list_picking_methods,
        crate::handlers::picking::update_picking_method,
        crate::handlers::picking::delete_picking_method,
        crate::handlers::picking::set_default_method,
        crate::handlers::picking::optimize_picking,
        crate::handlers::picking::confirm_picking_plan,
    ),
    components(
        schemas(
            CreatePickingMethodRequest,
            PickingMethodResponse,
            UpdatePickingMethodRequest,
        )
    ),
    tags(
        (name = "picking", description = "Warehouse picking and optimization operations"),
    ),
)]
pub struct PickingApiDoc;

// Receipts OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::receipt::create_receipt,
        crate::handlers::receipt::get_receipt,
        crate::handlers::receipt::list_receipts,
        crate::handlers::receipt::validate_receipt,
    ),
    components(
        schemas(
            ReceiptCreateRequest,
            ReceiptResponse,
            ReceiptListResponse,
            ReceiptItemCreateRequest,
            ReceiptItemResponse,
            ReceiptSummaryResponse,
        )
    ),
    tags(
        (name = "receipts", description = "Goods receipt note operations"),
    ),
)]
pub struct ReceiptsApiDoc;

// Search OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::search::search_products,
        crate::handlers::search::search_suggestions,
    ),
    components(
        schemas(
            ProductSearchQuery,
            SearchSuggestionsQuery,
            ProductSearchResponse,
            SearchSuggestionsResponse,
            SearchErrorResponse,
        )
    ),
    tags(
        (name = "search", description = "Product search and suggestions"),
    ),
)]
pub struct SearchApiDoc;

// Valuation OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::valuation::get_valuation,
        crate::handlers::valuation::get_valuation_history,
        crate::handlers::valuation::get_valuation_layers,
        crate::handlers::valuation::set_valuation_method,
        crate::handlers::valuation::set_standard_cost,
        crate::handlers::valuation::adjust_cost,
        crate::handlers::valuation::revalue_inventory,
    ),
    components(
        schemas(
            ValuationDto,
            ValuationHistoryResponse,
            ValuationLayersResponse,
            SetValuationMethodPayload,
            SetStandardCostPayload,
            CostAdjustmentPayload,
            RevaluationPayload,
            HistoryQueryParams,
            ValuationErrorResponse,
        )
    ),
    tags(
        (name = "valuation", description = "Inventory valuation and costing operations"),
    ),
)]
pub struct ValuationApiDoc;

// Warehouse OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::warehouses::create_warehouse,
        crate::handlers::warehouses::get_warehouse,
        crate::handlers::warehouses::get_warehouses,
        crate::handlers::warehouses::update_warehouse,
        crate::handlers::warehouses::delete_warehouse,
        crate::handlers::warehouses::get_warehouse_tree,
        crate::handlers::warehouses::create_zone,
        crate::handlers::warehouses::create_location,
    ),
    components(
        schemas(
            CreateWarehouseRequest,
            WarehouseResponse,
            WarehouseTreeResponse,
            CreateWarehouseZoneRequest,
            WarehouseZoneResponse,
            CreateWarehouseLocationRequest,
            WarehouseLocationResponse,
            WarehouseErrorResponse,
        )
    ),
    tags(
        (name = "warehouses", description = "Warehouse management endpoints"),
    ),
)]
pub struct WarehouseApiDoc;

/// OpenAPI documentation for Inventory Service
#[derive(OpenApi)]
#[openapi(
    paths(
        // Health
        crate::handlers::health::health_check,
        // Categories - CRUD operations (excluding recursive tree endpoints)
        crate::handlers::category::create_category,
        crate::handlers::category::get_category,
        crate::handlers::category::update_category,
        crate::handlers::category::delete_category,
        crate::handlers::category::list_categories,
        crate::handlers::category::search_categories,
        crate::handlers::category::get_top_categories,
        crate::handlers::category::get_children,
        crate::handlers::category::get_breadcrumbs,
        crate::handlers::category::get_category_stats,
        crate::handlers::category::bulk_activate_categories,
        crate::handlers::category::bulk_deactivate_categories,
        crate::handlers::category::bulk_delete_categories,
        crate::handlers::category::move_products_to_category,
        // Products - CRUD operations
        crate::handlers::products::create_product,
        crate::handlers::products::get_product,
        crate::handlers::products::list_products,
        crate::handlers::products::update_product,
        crate::handlers::products::delete_product,
        // Warehouses - CRUD operations (excluding recursive tree endpoints)
        crate::handlers::warehouses::create_warehouse,
        crate::handlers::warehouses::get_warehouse,
        crate::handlers::warehouses::get_warehouses,
        crate::handlers::warehouses::update_warehouse,
        crate::handlers::warehouses::delete_warehouse,
        crate::handlers::warehouses::create_zone,
        crate::handlers::warehouses::create_location,
        // Receipts - Full operations
        crate::handlers::receipt::create_receipt,
        crate::handlers::receipt::get_receipt,
        crate::handlers::receipt::list_receipts,
        crate::handlers::receipt::validate_receipt,
        // Lot Serial - Full operations
        crate::handlers::lot_serial::create_lot_serial,
        crate::handlers::lot_serial::get_lot_serial,
        crate::handlers::lot_serial::update_lot_serial,
        crate::handlers::lot_serial::delete_lot_serial,
        crate::handlers::lot_serial::list_lot_serials_by_product,
        crate::handlers::lot_serial::get_lot_serial_lifecycle,
        crate::handlers::lot_serial::quarantine_expired_lots,
        // Picking - Full operations
        crate::handlers::picking::create_picking_method,
        crate::handlers::picking::get_picking_method,
        crate::handlers::picking::list_picking_methods,
        crate::handlers::picking::update_picking_method,
        crate::handlers::picking::delete_picking_method,
        crate::handlers::picking::set_default_method,
        crate::handlers::picking::optimize_picking,
        crate::handlers::picking::confirm_picking_plan,
        // Putaway - Basic operations
        crate::handlers::putaway::confirm_putaway,
        crate::handlers::putaway::suggest_putaway,
        // Quality - Full operations
        crate::handlers::quality::create_qc_point,
        crate::handlers::quality::get_qc_point,
        crate::handlers::quality::list_qc_points,
        crate::handlers::quality::update_qc_point,
        crate::handlers::quality::delete_qc_point,
        // Reconciliation - Full operations
        crate::handlers::reconciliation::create_reconciliation,
        crate::handlers::reconciliation::count_reconciliation,
        crate::handlers::reconciliation::finalize_reconciliation,
        crate::handlers::reconciliation::approve_reconciliation,
        crate::handlers::reconciliation::list_reconciliations,
        crate::handlers::reconciliation::get_reconciliation,
        crate::handlers::reconciliation::get_reconciliation_analytics,
        crate::handlers::reconciliation::get_variance_analysis,
        crate::handlers::reconciliation::scan_barcode,
        // Replenishment - Full operations
        crate::handlers::replenishment::create_reorder_rule,
        crate::handlers::replenishment::get_reorder_rule,
        crate::handlers::replenishment::update_reorder_rule,
        crate::handlers::replenishment::delete_reorder_rule,
        crate::handlers::replenishment::list_reorder_rules_for_product,
        crate::handlers::replenishment::run_replenishment_check,
        crate::handlers::replenishment::check_product_replenishment,
        // Reports - Full operations
        crate::handlers::reports::get_stock_ledger,
        crate::handlers::reports::get_stock_aging,
        crate::handlers::reports::get_inventory_turnover,
        crate::handlers::reports::get_low_stock,
        crate::handlers::reports::get_dead_stock,
        // RMA - Full operations
        crate::handlers::rma::create_rma,
        crate::handlers::rma::approve_rma,
        crate::handlers::rma::receive_rma,
        // Transfer - Full operations
        crate::handlers::transfer::create_transfer,
        crate::handlers::transfer::confirm_transfer,
        crate::handlers::transfer::receive_transfer,
        // Valuation - Full operations
        crate::handlers::valuation::get_valuation,
        crate::handlers::valuation::get_valuation_history,
        crate::handlers::valuation::get_valuation_layers,
        crate::handlers::valuation::set_valuation_method,
        crate::handlers::valuation::set_standard_cost,
        crate::handlers::valuation::adjust_cost,
        crate::handlers::valuation::revalue_inventory,
    ),
    components(
        schemas(
            // Health
            HealthResp,
            // Categories
            CategoryCreateRequest,
            CategoryUpdateRequest,
            CategoryResponse,
            CategoryListResponse,
            CategoryStatsResponse,
            BulkCategoryIds,
            BulkOperationResponse,
            MoveToCategoryRequest,
            inventory_service_core::domains::category::CategoryBreadcrumb,
            // Products
            ProductCreateRequest,
            ProductResponse,
            ProductListResponse,
            ProductUpdateRequest,
            ProductListQuery,
            // Warehouses
            CreateWarehouseRequest,
            WarehouseResponse,
            CreateWarehouseZoneRequest,
            WarehouseZoneResponse,
            CreateWarehouseLocationRequest,
            WarehouseLocationResponse,
            WarehouseErrorResponse,
            // Receipts
            ReceiptCreateRequest,
            ReceiptResponse,
            ReceiptListResponse,
            ReceiptItemCreateRequest,
            ReceiptItemResponse,
            ReceiptSummaryResponse,
            // Lot Serial
            LotSerial,
            LotSerialLifecycle,
            CreateLotSerialRequest,
            ListLotSerialsQuery,
            QuarantineResponse,
            // Picking
            CreatePickingMethodRequest,
            PickingMethodResponse,
            UpdatePickingMethodRequest,
            // Putaway
            ConfirmPutawayRequest,
            ConfirmPutawayResponse,
            PutawayRequest,
            PutawayResponse,
            PutawaySuggestion,
            // Quality
            CreateQualityControlPoint,
            QualityControlPoint,
            UpdateQualityControlPoint,
            // Reconciliation
            CreateReconciliationRequest,
            CreateReconciliationResponse,
            CountReconciliationRequest,
            CountReconciliationResponse,
            FinalizeReconciliationRequest,
            FinalizeReconciliationResponse,
            ApproveReconciliationRequest,
            ApproveReconciliationResponse,
            ReconciliationListResponse,
            ReconciliationDetailResponse,

            ReconciliationAnalyticsResponse,
            VarianceAnalysisResponse,
            ScanBarcodeRequest,
            ScanBarcodeResponse,
            // Replenishment
            CreateReorderRule,
            ReplenishmentCheckResult,
            UpdateReorderRule,
            // Reports
            StockLedgerQuery,
            StockLedgerEntry,
            StockAgingQuery,
            StockAgingEntry,
            InventoryTurnoverQuery,
            InventoryTurnoverEntry,
            LowStockQuery,
            LowStockEntry,
            DeadStockQuery,
            DeadStockEntry,
            // RMA
            CreateRmaRequest,
            CreateRmaResponse,
            ApproveRmaRequest,
            ApproveRmaResponse,
            ReceiveRmaRequest,
            ReceiveRmaResponse,
            // Transfer
            CreateTransferRequest,
            CreateTransferResponse,
            ConfirmTransferRequest,
            ConfirmTransferResponse,
            ReceiveTransferRequest,
            ReceiveTransferResponse,
            // Valuation
            ValuationDto,
            ValuationHistoryResponse,
            ValuationLayersResponse,
            SetValuationMethodPayload,
            SetStandardCostPayload,
            CostAdjustmentPayload,
            RevaluationPayload,
            HistoryQueryParams,
            ValuationErrorResponse,
            // Common
            PaginationInfo,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "categories", description = "Category management endpoints"),
        (name = "products", description = "Product management endpoints"),
        (name = "warehouses", description = "Warehouse management endpoints"),
        (name = "receipts", description = "Goods receipt note operations"),
        (name = "lot-serial", description = "Lot serial management endpoints"),
        (name = "picking", description = "Warehouse picking and optimization operations"),
        (name = "putaway", description = "Putaway and storage location operations"),
        (name = "quality", description = "Quality control point management"),
        (name = "reconciliation", description = "Inventory reconciliation operations"),
        (name = "replenishment", description = "Automatic replenishment rules"),
        (name = "reports", description = "Inventory reports and analytics"),
        (name = "rma", description = "Return merchandise authorization"),
        (name = "transfer", description = "Stock transfer operations"),
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
pub fn export_spec() -> Result<(), Box<dyn ::std::error::Error>> {
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
