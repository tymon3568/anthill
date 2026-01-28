//! Product Variant Data Transfer Objects
//!
//! This module defines the DTOs used for product variant API operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::domains::inventory::product_variant::ProductVariant;
use crate::dto::common::PaginationInfo;

/// Product variant creation request DTO
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct VariantCreateRequest {
    /// Parent product ID
    pub parent_product_id: Uuid,

    /// Variant SKU (unique within tenant)
    #[validate(length(min = 1, max = 100))]
    pub sku: String,

    /// Optional barcode
    #[validate(length(max = 100))]
    pub barcode: Option<String>,

    /// Variant attributes (e.g., {"color": "red", "size": "L"})
    pub variant_attributes: serde_json::Value,

    /// Price difference from parent product (in cents)
    /// Default: 0
    pub price_difference: Option<i64>,

    /// Whether variant is active
    /// Default: true
    pub is_active: Option<bool>,
}

/// Product variant update request DTO
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct VariantUpdateRequest {
    /// Variant SKU
    #[validate(length(min = 1, max = 100))]
    pub sku: Option<String>,

    /// Barcode
    #[validate(length(max = 100))]
    pub barcode: Option<String>,

    /// Variant attributes
    pub variant_attributes: Option<serde_json::Value>,

    /// Price difference from parent product (in cents)
    pub price_difference: Option<i64>,

    /// Whether variant is active
    pub is_active: Option<bool>,
}

/// Product variant response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct VariantResponse {
    /// Primary key using UUID v7
    pub variant_id: Uuid,

    /// Multi-tenancy: Tenant ID
    pub tenant_id: Uuid,

    /// Parent product ID
    pub parent_product_id: Uuid,

    /// Variant attributes
    pub variant_attributes: serde_json::Value,

    /// Variant SKU
    pub sku: String,

    /// Barcode
    pub barcode: Option<String>,

    /// Price difference from parent product (in cents)
    pub price_difference: i64,

    /// Whether variant is active
    pub is_active: bool,

    /// Audit fields
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    /// Joined fields from parent product
    pub parent_product_name: Option<String>,
    pub parent_product_sku: Option<String>,
}

impl From<ProductVariant> for VariantResponse {
    fn from(variant: ProductVariant) -> Self {
        Self {
            variant_id: variant.variant_id,
            tenant_id: variant.tenant_id,
            parent_product_id: variant.parent_product_id,
            variant_attributes: variant.variant_attributes,
            sku: variant.sku,
            barcode: variant.barcode,
            price_difference: variant.price_difference,
            is_active: variant.is_active,
            created_at: variant.created_at,
            updated_at: variant.updated_at,
            parent_product_name: None,
            parent_product_sku: None,
        }
    }
}

impl VariantResponse {
    /// Create from variant with parent product info
    pub fn with_parent_info(
        variant: ProductVariant,
        parent_product_name: Option<String>,
        parent_product_sku: Option<String>,
    ) -> Self {
        Self {
            variant_id: variant.variant_id,
            tenant_id: variant.tenant_id,
            parent_product_id: variant.parent_product_id,
            variant_attributes: variant.variant_attributes,
            sku: variant.sku,
            barcode: variant.barcode,
            price_difference: variant.price_difference,
            is_active: variant.is_active,
            created_at: variant.created_at,
            updated_at: variant.updated_at,
            parent_product_name,
            parent_product_sku,
        }
    }
}

/// Sort direction enum for variant list queries
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum VariantSortDirection {
    /// Ascending order
    #[default]
    Asc,
    /// Descending order
    Desc,
}

/// Product variant list query parameters
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema, utoipa::IntoParams))]
#[serde(rename_all = "camelCase")]
pub struct VariantListQuery {
    /// Filter by parent product ID
    pub parent_product_id: Option<Uuid>,

    /// Filter by active status
    pub is_active: Option<bool>,

    /// Search in SKU, barcode, and parent product name
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
    #[serde(default)]
    pub sort_dir: VariantSortDirection,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    20
}

fn default_sort_by() -> String {
    "sku".to_string()
}

/// Product variant list response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct VariantListResponse {
    /// List of variants
    pub variants: Vec<VariantResponse>,

    /// Pagination information
    pub pagination: PaginationInfo,
}

/// Bulk variant IDs request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct BulkVariantIds {
    /// List of variant IDs to operate on
    pub variant_ids: Vec<Uuid>,
}

/// Bulk operation response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct BulkVariantOperationResponse {
    /// Whether the operation was successful
    pub success: bool,

    /// Number of affected records
    pub affected_count: i64,

    /// Message describing the result
    pub message: String,
}

impl BulkVariantOperationResponse {
    /// Create a successful bulk operation response
    pub fn success(affected_count: i64, action: &str) -> Self {
        Self {
            success: true,
            affected_count,
            message: format!("Successfully {} {} variant(s)", action, affected_count),
        }
    }

    /// Create a failed bulk operation response
    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            affected_count: 0,
            message,
        }
    }
}
