//! Data Transfer Objects (DTOs) for inventory service
//!
//! This module contains all request and response structures for the API.

pub mod category;
pub mod common;
pub mod delivery;
pub mod receipt;
pub mod reconciliation;
pub mod removal_strategy;
pub mod rma;
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
pub use receipt::{
    ReceiptCreateRequest, ReceiptItemCreateRequest, ReceiptItemResponse, ReceiptListQuery,
    ReceiptListResponse, ReceiptResponse, ReceiptSummaryResponse,
};
pub use reconciliation::{
    ApproveReconciliationRequest, ApproveReconciliationResponse, CountReconciliationRequest,
    CountReconciliationResponse, CreateReconciliationRequest, CreateReconciliationResponse,
    FinalizeReconciliationRequest, FinalizeReconciliationResponse, ReconciliationAnalyticsResponse,
    ReconciliationCountItem, ReconciliationDetailResponse, ReconciliationListQuery,
    ReconciliationListResponse, VarianceAnalysisResponse,
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
