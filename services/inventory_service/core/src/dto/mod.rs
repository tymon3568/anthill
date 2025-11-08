//! Data Transfer Objects (DTOs) for inventory service
//!
//! This module contains all request and response structures for the API.

pub mod category;

// Re-export main types for convenience
pub use category::{
    BulkOperationResponse, CategoryCreateRequest, CategoryListQuery, CategoryListResponse,
    CategoryResponse, CategorySortField, CategoryStatsResponse, CategoryTreeResponse,
    CategoryUpdateRequest, MoveToCategoryRequest, PaginationInfo, SortDirection,
};
