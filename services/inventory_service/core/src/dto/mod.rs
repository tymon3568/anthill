//! Data Transfer Objects (DTOs) for inventory service
//!
//! This module contains all request and response structures for the API.

pub mod adjustment;
pub mod category;
pub mod common;
pub mod cycle_count;
pub mod delivery;
pub mod product;
pub mod product_image;
pub mod product_import;
pub mod product_variant;
pub mod receipt;
pub mod reconciliation;
pub mod removal_strategy;
pub mod reports;
pub mod rma;
pub mod scrap;
pub mod stock_levels;
pub mod stock_take;
pub mod transfer;

// Re-export main types for convenience
pub use category::{
    BulkOperationResponse, CategoryCreateRequest, CategoryListQuery, CategoryListResponse,
    CategoryResponse, CategorySortField, CategoryStatsResponse, CategoryTreeResponse,
    CategoryUpdateRequest, MoveToCategoryRequest, SortDirection,
};
// pub use delivery::{PickItemRequest, PickItemsRequest, PickItemsResponse};
pub use common::PaginationInfo;
pub use product::{
    ProductCreateRequest, ProductListQuery, ProductListResponse, ProductResponse,
    ProductUpdateRequest,
};
pub use product_image::{
    DeleteImageResponse, ProductImageResponse, ProductImagesListResponse, ReorderImagesRequest,
    UpdateProductImageRequest, UploadImageResponse,
};
pub use product_import::{
    ExportProductsQuery, ImportResult, ImportRowError, ImportValidationResult, ProductCsvRow,
};
pub use product_variant::{
    BulkVariantIds, BulkVariantOperationResponse, VariantCreateRequest, VariantListQuery,
    VariantListResponse, VariantResponse, VariantUpdateRequest,
};
pub use receipt::{
    ReceiptCreateRequest, ReceiptItemCreateRequest, ReceiptItemResponse, ReceiptListQuery,
    ReceiptListResponse, ReceiptResponse, ReceiptSummaryResponse,
};
pub use reconciliation::{
    ApproveReconciliationRequest, ApproveReconciliationResponse, CountReconciliationRequest,
    CountReconciliationResponse, CreateReconciliationRequest, CreateReconciliationResponse,
    FinalizeReconciliationRequest, FinalizeReconciliationResponse, ReconciliationAnalyticsQuery,
    ReconciliationAnalyticsResponse, ReconciliationCountItem, ReconciliationDetailResponse,
    ReconciliationListQuery, ReconciliationListResponse, VarianceAnalysisResponse,
};
pub use removal_strategy::{
    RemovalStrategyCreateRequest, RemovalStrategyListQuery, RemovalStrategyListResponse,
    RemovalStrategyResponse, RemovalStrategyUpdateRequest, StockLocationInfo, StockSuggestion,
    StrategyAnalyticsResponse, SuggestRemovalRequest, SuggestRemovalResponse,
};
pub use rma::{
    ApproveRmaRequest, ApproveRmaResponse, CreateRmaRequest, CreateRmaResponse, ReceiveRmaRequest,
    ReceiveRmaResponse,
};
pub use stock_take::{
    CountItem, CountStockTakeRequest, CountStockTakeResponse, CreateStockTakeRequest,
    CreateStockTakeResponse, FinalizeStockTakeRequest, FinalizeStockTakeResponse, StockAdjustment,
    StockTakeDetailResponse, StockTakeListQuery, StockTakeListResponse,
};
pub use transfer::{
    CreateTransferItemRequest, CreateTransferRequest, TransferItemResponse, TransferResponse,
    UpdateTransferItemRequest, UpdateTransferRequest,
};

// Cycle counting DTOs
pub use cycle_count::{
    CountSubmission, CountType, CreateCycleCountRequest, CycleCountLine, CycleCountLineStatus,
    CycleCountListQuery, CycleCountListResponse, CycleCountResponse, CycleCountSession,
    CycleCountStatus, CycleCountSummary, CycleCountWithLinesResponse, GenerateLinesRequest,
    LineAdjustment, ReconcileRequest, ReconcileResponse, SkipLinesRequest, SubmitCountsRequest,
};

// Reports DTOs
pub use reports::{
    AgeBucket, AgeBucketPreset, AgingBasis, StockAgingReportQuery, StockAgingReportResponse,
    StockAgingReportRow, TurnoverGroupBy, TurnoverReportQuery, TurnoverReportResponse,
    TurnoverReportRow,
};

// Scrap management DTOs
pub use scrap::{
    AddScrapLinesRequest, CreateScrapRequest, PostScrapRequest, ScrapDocument,
    ScrapDocumentResponse, ScrapDocumentWithLinesResponse, ScrapLine, ScrapLineInput,
    ScrapListQuery, ScrapListResponse, ScrapReasonCode, ScrapStatus,
};

// Stock Levels DTOs
pub use stock_levels::{
    StockLevelListQuery, StockLevelListResponse, StockLevelResponse, StockLevelSummary, StockStatus,
};

// Stock Adjustment DTOs
pub use adjustment::{
    AddAdjustmentLinesRequest, AdjustmentDocument, AdjustmentDocumentResponse,
    AdjustmentDocumentWithLinesResponse, AdjustmentLine, AdjustmentLineInput, AdjustmentListQuery,
    AdjustmentListResponse, AdjustmentReasonCode, AdjustmentStatus, AdjustmentSummary,
    AdjustmentType, CreateAdjustmentRequest, PostAdjustmentRequest,
};
