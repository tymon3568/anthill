//! Product Import/Export Data Transfer Objects
//!
//! DTOs for bulk import and export of products via CSV.

use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;

/// CSV row representation for product import/export
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ProductCsvRow {
    /// Product SKU (unique within tenant)
    pub sku: String,
    /// Product name
    pub name: String,
    /// Optional product description
    pub description: Option<String>,
    /// Product type (goods, service, consumable)
    pub product_type: Option<String>,
    /// Optional category ID for product organization
    pub category_id: Option<Uuid>,
    /// Sale price in cents (smallest currency unit)
    pub sale_price: Option<i64>,
    /// Cost price in cents (smallest currency unit)
    pub cost_price: Option<i64>,
    /// Currency code (ISO 4217)
    pub currency: Option<String>,
    /// Product weight in grams
    pub weight: Option<i32>,
    /// Product length in mm
    pub length: Option<i32>,
    /// Product width in mm
    pub width: Option<i32>,
    /// Product height in mm
    pub height: Option<i32>,
    /// Barcode for product identification
    pub barcode: Option<String>,
    /// Type of barcode (ean13, upc_a, isbn, custom)
    pub barcode_type: Option<String>,
    /// Whether product is active
    pub is_active: Option<bool>,
}

/// Result of validating a CSV file before import
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ImportValidationResult {
    /// Whether the file is valid for import
    pub is_valid: bool,
    /// Total number of rows in the file
    pub total_rows: i32,
    /// Number of valid rows
    pub valid_rows: i32,
    /// List of validation errors
    pub errors: Vec<ImportRowError>,
    /// Preview of first few rows (if valid)
    pub preview: Option<Vec<ProductCsvRow>>,
}

/// Error for a specific row in the import file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ImportRowError {
    /// Row number (1-indexed)
    pub row_number: i32,
    /// Field name that has the error
    pub field: String,
    /// Error message
    pub error: String,
}

/// Result of importing products from CSV
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    /// Number of products created
    pub created: i32,
    /// Number of products updated (upsert mode)
    pub updated: i32,
    /// Number of products that failed to import
    pub failed: i32,
    /// List of errors for failed rows
    pub errors: Vec<ImportRowError>,
}

/// Query parameters for exporting products
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema, utoipa::IntoParams))]
#[serde(rename_all = "camelCase")]
pub struct ExportProductsQuery {
    /// Filter by category ID
    pub category_id: Option<Uuid>,
    /// Filter by product type
    pub product_type: Option<String>,
    /// Filter by active status
    pub is_active: Option<bool>,
    /// Search term for SKU or name
    pub search: Option<String>,
}
