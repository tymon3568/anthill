//! Product search HTTP handlers
//!
//! This module contains the Axum handlers for product search endpoints.

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use axum::{
    extract::{Extension, Query},
    response::Json,
    routing::get,
    Router,
};

use inventory_service_core::domains::inventory::dto::search_dto::{
    ProductSearchRequest, ProductSearchResponse, SearchSuggestionsRequest,
    SearchSuggestionsResponse,
};

use crate::state::AppState;
use shared_auth::extractors::AuthUser;
use shared_error::AppError;

/// Error response for OpenAPI documentation
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// Error code
    pub code: String,
}

/// Create the search routes
pub fn create_search_routes() -> Router {
    Router::new()
        .route("/", get(search_products))
        .route("/suggestions", get(search_suggestions))
}

/// GET /api/v1/inventory/products/search - Advanced product search
///
/// Performs comprehensive product search with full-text search, filtering,
/// and pagination. Supports category hierarchy, price ranges, and multiple
/// sorting options.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Query Parameters
/// * `query` - Full-text search query (optional)
/// * `categoryIds` - Comma-separated category IDs for filtering (optional)
/// * `priceMin` - Minimum price filter (optional)
/// * `priceMax` - Maximum price filter (optional)
/// * `inStockOnly` - Filter for in-stock products only (optional)
/// * `productTypes` - Comma-separated product types (optional)
/// * `activeOnly` - Filter for active products only (default: true)
/// * `sellableOnly` - Filter for sellable products only (default: true)
/// * `sortBy` - Sort field: relevance, name, price, popularity, createdAt, updatedAt (default: relevance)
/// * `sortOrder` - Sort order: asc, desc (default: desc)
/// * `page` - Page number (default: 1, min: 1)
/// * `limit` - Items per page (default: 20, max: 100)
///
/// # Returns
/// * `200` - Search results with pagination, facets, and metadata
/// * `400` - Invalid query parameters
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
///
/// # Example
/// ```
/// GET /api/v1/inventory/products/search?query=laptop&categoryIds=123e4567-e89b-12d3-a456-426614174000&priceMin=1000000&sortBy=price&sortOrder=asc&page=1&limit=20
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/inventory/search",
    tag = "search",
    operation_id = "search_products",
    params(
        ProductSearchQuery
    ),
    responses(
        (status = 200, description = "Search results with pagination, facets, and metadata", body = ProductSearchResponse),
        (status = 400, description = "Invalid query parameters", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn search_products(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Query(params): Query<ProductSearchQuery>,
) -> Result<Json<ProductSearchResponse>, AppError> {
    // Convert query parameters to search request
    let request = params.into_search_request()?;

    // Perform search
    let response = state
        .product_service
        .search_products(auth_user.tenant_id, request)
        .await?;

    Ok(Json(response))
}

/// GET /api/v1/inventory/products/suggestions - Search suggestions/autocomplete
///
/// Provides search suggestions and autocomplete functionality for product search.
/// Returns popular search terms, product names, SKUs, and categories that match
/// the partial query.
///
/// # Authentication
/// Requires authenticated user with appropriate tenant access
///
/// # Query Parameters
/// * `query` - Partial search query (required, min: 1 char)
/// * `limit` - Maximum number of suggestions (default: 10, max: 20)
///
/// # Returns
/// * `200` - List of search suggestions with types and counts
/// * `400` - Missing or invalid query parameter
/// * `401` - Authentication required
/// * `403` - Insufficient permissions
///
/// # Example
/// ```
/// GET /api/v1/inventory/products/suggestions?query=lapt&limit=5
/// ```
#[utoipa::path(
    get,
    path = "/api/v1/inventory/search/suggestions",
    tag = "search",
    operation_id = "search_suggestions",
    params(
        SearchSuggestionsQuery
    ),
    responses(
        (status = 200, description = "List of search suggestions with types and counts", body = SearchSuggestionsResponse),
        (status = 400, description = "Missing or invalid query parameter", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn search_suggestions(
    auth_user: AuthUser,
    Extension(state): Extension<AppState>,
    Query(params): Query<SearchSuggestionsQuery>,
) -> Result<Json<SearchSuggestionsResponse>, AppError> {
    // Convert query parameters to suggestions request
    let request = SearchSuggestionsRequest {
        query: params.query,
        limit: params.limit,
    };

    // Get suggestions
    let response = state
        .product_service
        .get_search_suggestions(auth_user.tenant_id, request)
        .await?;

    Ok(Json(response))
}

/// Query parameters for product search endpoint
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams, utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ProductSearchQuery {
    pub query: Option<String>,
    pub category_ids: Option<String>, // Comma-separated UUIDs
    pub price_min: Option<i64>,
    pub price_max: Option<i64>,
    pub in_stock_only: Option<bool>,
    pub product_types: Option<String>, // Comma-separated types
    pub active_only: Option<bool>,
    pub sellable_only: Option<bool>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl ProductSearchQuery {
    /// Convert query parameters to ProductSearchRequest
    fn into_search_request(self) -> Result<ProductSearchRequest, shared_error::AppError> {
        use inventory_service_core::domains::inventory::dto::search_dto::{
            ProductSortBy, SortOrder,
        };

        // Parse category IDs
        let category_ids = if let Some(ids) = &self.category_ids {
            let parsed: Result<Vec<uuid::Uuid>, _> = ids
                .split(',')
                .map(|id| id.trim().parse::<uuid::Uuid>())
                .collect();
            match parsed {
                Ok(ids) => Some(ids),
                Err(_) => {
                    return Err(shared_error::AppError::ValidationError(
                        "Invalid category ID format".to_string(),
                    ))
                },
            }
        } else {
            None
        };

        // Parse product types
        let product_types = self.product_types.as_ref().map(|types| {
            types
                .split(',')
                .map(|t| t.trim().to_string())
                .collect::<Vec<_>>()
        });

        // Parse sort options
        let sort_by = self.sort_by.as_ref().map(|s| match s.as_str() {
            "name" => ProductSortBy::Name,
            "price" => ProductSortBy::Price,
            "popularity" => ProductSortBy::Popularity,
            "createdAt" => ProductSortBy::CreatedAt,
            "updatedAt" => ProductSortBy::UpdatedAt,
            _ => ProductSortBy::Relevance,
        });

        let sort_order = self.sort_order.as_ref().map(|s| match s.as_str() {
            "asc" => SortOrder::Asc,
            _ => SortOrder::Desc,
        });

        let request = ProductSearchRequest {
            query: self.query,
            category_ids,
            price_min: self.price_min,
            price_max: self.price_max,
            in_stock_only: self.in_stock_only,
            product_types,
            active_only: self.active_only,
            sellable_only: self.sellable_only,
            sort_by,
            sort_order,
            page: self.page,
            limit: self.limit,
        };

        // Validate price range
        if let (Some(min), Some(max)) = (request.price_min, request.price_max) {
            if min > max {
                return Err(shared_error::AppError::ValidationError(
                    "price_min cannot be greater than price_max".to_string(),
                ));
            }
        }

        Ok(request)
    }
}

/// Query parameters for search suggestions endpoint
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams, utoipa::ToSchema))]
pub struct SearchSuggestionsQuery {
    pub query: String,
    pub limit: Option<u32>,
}
