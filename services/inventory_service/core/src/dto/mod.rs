//! Data Transfer Objects (DTOs) for inventory service
//!
//! This module contains all request and response structures for the API.

pub mod category;
pub mod delivery;
pub mod receipt;

// Re-export main types for convenience
pub use category::{
    BulkOperationResponse, CategoryCreateRequest, CategoryListQuery, CategoryListResponse,
    CategoryResponse, CategorySortField, CategoryStatsResponse, CategoryTreeResponse,
    CategoryUpdateRequest, MoveToCategoryRequest, PaginationInfo, SortDirection,
};
pub use delivery::{PickItemRequest, PickItemsRequest, PickItemsResponse};
pub use receipt::{
    ReceiptCreateRequest, ReceiptItemCreateRequest, ReceiptItemResponse, ReceiptListQuery,
    ReceiptListResponse, ReceiptResponse, ReceiptSummaryResponse,
};
