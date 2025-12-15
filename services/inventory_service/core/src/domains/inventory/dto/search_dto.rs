use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::dto::PaginationInfo;

/// Product search request DTO
#[derive(Debug, Clone, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ProductSearchRequest {
    /// Search query string (full-text search)
    #[validate(length(max = 255))]
    pub query: Option<String>,

    /// Filter by category IDs (supports hierarchy)
    pub category_ids: Option<Vec<Uuid>>,

    /// Price range filters (in cents)
    pub price_min: Option<i64>,
    pub price_max: Option<i64>,

    /// Availability filter
    pub in_stock_only: Option<bool>,

    /// Product type filter
    #[validate(custom(function = "validate_product_types"))]
    pub product_types: Option<Vec<String>>,

    /// Active products only
    pub active_only: Option<bool>,

    /// Sellable products only
    pub sellable_only: Option<bool>,

    /// Sorting options
    pub sort_by: Option<ProductSortBy>,
    pub sort_order: Option<SortOrder>,

    /// Pagination
    #[validate(range(min = 1, max = 1000))]
    pub page: Option<u32>,
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<u32>,
}

impl Default for ProductSearchRequest {
    fn default() -> Self {
        Self {
            query: None,
            category_ids: None,
            price_min: None,
            price_max: None,
            in_stock_only: None,
            product_types: None,
            active_only: Some(true),
            sellable_only: Some(true),
            sort_by: Some(ProductSortBy::Relevance),
            sort_order: Some(SortOrder::Desc),
            page: Some(1),
            limit: Some(20),
        }
    }
}

/// Product sort options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ProductSortBy {
    Relevance,
    Name,
    Price,
    Popularity,
    CreatedAt,
    UpdatedAt,
}

/// Sort order
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Product search response DTO
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ProductSearchResponse {
    /// Search results
    pub products: Vec<ProductSearchResult>,

    /// Pagination info
    pub pagination: PaginationInfo,

    /// Search facets for filtering
    pub facets: SearchFacets,

    /// Search metadata
    pub meta: SearchMeta,
}

/// Individual product search result with highlights
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ProductSearchResult {
    /// Product ID
    pub product_id: Uuid,

    /// Product details
    pub sku: String,
    pub name: String,
    pub description: Option<String>,

    /// Pricing
    pub sale_price: Option<i64>,
    pub cost_price: Option<i64>,
    pub currency_code: String,

    /// Product type and category
    pub product_type: String,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub category_path: Option<String>,

    /// Inventory status
    pub track_inventory: bool,
    pub in_stock: Option<bool>, // Will be calculated based on inventory levels

    /// Product status
    pub is_active: bool,
    pub is_sellable: bool,

    /// Search highlights (highlighted text snippets)
    pub highlights: Vec<String>,

    /// Relevance score (0.0 to 1.0)
    pub relevance_score: f32,

    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Search facets for filtering UI
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SearchFacets {
    /// Category facets
    pub categories: Vec<CategoryFacet>,

    /// Price range facets
    pub price_ranges: Vec<PriceRangeFacet>,

    /// Product type facets
    pub product_types: Vec<ProductTypeFacet>,
}

/// Category facet
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CategoryFacet {
    pub category_id: Uuid,
    pub name: String,
    pub path: String,
    pub product_count: u32,
    pub level: i32,
}

/// Price range facet
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct PriceRangeFacet {
    pub min_price: i64,
    pub max_price: i64,
    pub product_count: u32,
    pub label: String,
}

/// Product type facet
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ProductTypeFacet {
    pub product_type: String,
    pub product_count: u32,
    pub label: String,
}

/// Search metadata
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SearchMeta {
    /// Original search query
    pub query: Option<String>,

    /// Search execution time in milliseconds
    pub execution_time_ms: u64,

    /// Total products found before filtering
    pub total_found: u64,

    /// Applied filters summary
    pub applied_filters: AppliedFilters,
}

/// Applied filters summary
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct AppliedFilters {
    pub category_ids: Option<Vec<Uuid>>,
    pub price_min: Option<i64>,
    pub price_max: Option<i64>,
    pub in_stock_only: Option<bool>,
    pub product_types: Option<Vec<String>>,
    pub active_only: Option<bool>,
    pub sellable_only: Option<bool>,
}

/// Search suggestions/autocomplete request
#[derive(Debug, Clone, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SearchSuggestionsRequest {
    #[validate(length(min = 1, max = 100))]
    pub query: String,

    #[validate(range(min = 1, max = 20))]
    pub limit: Option<u32>,
}

impl Default for SearchSuggestionsRequest {
    fn default() -> Self {
        Self {
            query: " ".to_string(),
            limit: Some(10),
        }
    }
}

/// Search suggestions response
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SearchSuggestionsResponse {
    pub suggestions: Vec<SearchSuggestion>,
}

/// Individual search suggestion
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SearchSuggestion {
    pub text: String,
    pub product_count: u32,
    pub suggestion_type: SuggestionType,
}

/// Suggestion types
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum SuggestionType {
    ProductName,
    Category,
    Sku,
}

/// Valid product type values
pub const VALID_PRODUCT_TYPES: &[&str] = &["goods", "service", "consumable"];

/// Validation functions
fn validate_product_types(product_types: &[String]) -> Result<(), validator::ValidationError> {
    for pt in product_types {
        if !VALID_PRODUCT_TYPES.contains(&pt.as_str()) {
            return Err(validator::ValidationError::new("invalid_product_type"));
        }
    }
    Ok(())
}
