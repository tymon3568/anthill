//! Product Data Transfer Objects
//!
//! This module defines the DTOs used for product API operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::domains::inventory::product::{Product, ProductTrackingMethod};
use crate::dto::common::PaginationInfo;

/// Sort direction enum for product list queries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    /// Ascending order
    Asc,
    /// Descending order
    Desc,
}

/// Product creation request DTO
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ProductCreateRequest {
    /// Product SKU (unique within tenant)
    #[validate(length(min = 1, max = 100))]
    pub sku: String,

    /// Product name
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// Optional product description
    #[validate(length(max = 1000))]
    pub description: Option<String>,

    /// Product type (goods, service, consumable)
    #[validate(length(min = 1, max = 50))]
    pub product_type: String,

    /// Optional category ID for product organization
    pub category_id: Option<Uuid>,

    /// Optional item group ID
    pub item_group_id: Option<Uuid>,

    /// Whether to track inventory for this product
    pub track_inventory: Option<bool>,

    /// Inventory tracking method
    pub tracking_method: Option<ProductTrackingMethod>,

    /// Default unit of measure ID
    pub default_uom_id: Option<Uuid>,

    /// Sale price in cents (smallest currency unit)
    #[validate(range(min = 0))]
    pub sale_price: Option<i64>,

    /// Cost price in cents (smallest currency unit)
    #[validate(range(min = 0))]
    pub cost_price: Option<i64>,

    /// Currency code (ISO 4217)
    #[validate(length(min = 3, max = 3))]
    pub currency_code: String,

    /// Product weight in grams
    #[validate(range(min = 0))]
    pub weight_grams: Option<i32>,

    /// Product dimensions (JSON)
    pub dimensions: Option<serde_json::Value>,

    /// Additional product attributes (JSON)
    pub attributes: Option<serde_json::Value>,

    /// Whether product is active
    pub is_active: Option<bool>,

    /// Whether product is available for sale
    pub is_sellable: Option<bool>,

    /// Whether product is available for purchase
    pub is_purchaseable: Option<bool>,
}

/// Product update request DTO
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ProductUpdateRequest {
    /// Product name
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,

    /// Product description
    #[validate(length(max = 1000))]
    pub description: Option<String>,

    /// Product type
    #[validate(length(min = 1, max = 50))]
    pub product_type: Option<String>,

    /// Category ID for product organization
    pub category_id: Option<Uuid>,

    /// Item group ID
    pub item_group_id: Option<Uuid>,

    /// Whether to track inventory
    pub track_inventory: Option<bool>,

    /// Inventory tracking method
    pub tracking_method: Option<ProductTrackingMethod>,

    /// Default unit of measure ID
    pub default_uom_id: Option<Uuid>,

    /// Sale price in cents
    #[validate(range(min = 0))]
    pub sale_price: Option<i64>,

    /// Cost price in cents
    #[validate(range(min = 0))]
    pub cost_price: Option<i64>,

    /// Currency code
    #[validate(length(min = 3, max = 3))]
    pub currency_code: Option<String>,

    /// Product weight in grams
    #[validate(range(min = 0))]
    pub weight_grams: Option<i32>,

    /// Product dimensions (JSON)
    pub dimensions: Option<serde_json::Value>,

    /// Additional product attributes (JSON)
    pub attributes: Option<serde_json::Value>,

    /// Whether product is active
    pub is_active: Option<bool>,

    /// Whether product is available for sale
    pub is_sellable: Option<bool>,

    /// Whether product is available for purchase
    pub is_purchaseable: Option<bool>,
}

/// Product response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ProductResponse {
    /// Primary key using UUID v7
    pub product_id: Uuid,

    /// Multi-tenancy: Tenant ID
    pub tenant_id: Uuid,

    /// Core product identifiers
    pub sku: String,
    pub name: String,
    pub description: Option<String>,

    /// Product classification
    pub product_type: String,
    pub category_id: Option<Uuid>,
    pub item_group_id: Option<Uuid>,

    /// Inventory tracking
    pub track_inventory: bool,
    pub tracking_method: ProductTrackingMethod,

    /// Units of measure
    pub default_uom_id: Option<Uuid>,

    /// Pricing (stored in smallest currency unit: cents/xu)
    pub sale_price: Option<i64>,
    pub cost_price: Option<i64>,
    pub currency_code: String,

    /// Product attributes
    pub weight_grams: Option<i32>,
    pub dimensions: Option<serde_json::Value>,
    pub attributes: Option<serde_json::Value>,

    /// Product lifecycle
    pub is_active: bool,
    pub is_sellable: bool,
    pub is_purchaseable: bool,

    /// Audit fields
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Product> for ProductResponse {
    fn from(product: Product) -> Self {
        Self {
            product_id: product.product_id,
            tenant_id: product.tenant_id,
            sku: product.sku,
            name: product.name,
            description: product.description,
            product_type: product.product_type,
            category_id: product.category_id,
            item_group_id: product.item_group_id,
            track_inventory: product.track_inventory,
            tracking_method: product.tracking_method,
            default_uom_id: product.default_uom_id,
            sale_price: product.sale_price,
            cost_price: product.cost_price,
            currency_code: product.currency_code,
            weight_grams: product.weight_grams,
            dimensions: product.dimensions,
            attributes: product.attributes,
            is_active: product.is_active,
            is_sellable: product.is_sellable,
            is_purchaseable: product.is_purchaseable,
            created_at: product.created_at,
            updated_at: product.updated_at,
        }
    }
}

/// Product list query parameters
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema, utoipa::IntoParams))]
#[serde(rename_all = "camelCase")]
pub struct ProductListQuery {
    /// Filter by product type
    pub product_type: Option<String>,

    /// Filter by category ID
    pub category_id: Option<Uuid>,

    /// Filter by active status
    pub is_active: Option<bool>,

    /// Filter by sellable status
    pub is_sellable: Option<bool>,

    /// Filter by purchaseable status
    pub is_purchaseable: Option<bool>,

    /// Search in name, SKU, and description
    pub search: Option<String>,

    /// Page number (1-based)
    #[serde(default = "default_page")]
    #[validate(range(min = 1))]
    pub page: i64,

    /// Items per page
    #[serde(default = "default_page_size")]
    #[validate(range(min = 1, max = 100))]
    pub page_size: i64,

    /// Sort field
    #[serde(default = "default_sort_by")]
    #[validate(length(min = 1, max = 50))]
    pub sort_by: String,

    /// Sort direction
    #[serde(default = "default_sort_dir")]
    pub sort_dir: SortDirection,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    20
}

fn default_sort_by() -> String {
    "name".to_string()
}

fn default_sort_dir() -> SortDirection {
    SortDirection::Asc
}

/// Product list response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ProductListResponse {
    /// List of products
    pub products: Vec<ProductResponse>,

    /// Pagination information
    pub pagination: PaginationInfo,
}
