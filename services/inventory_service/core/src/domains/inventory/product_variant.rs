//! Product Variant domain entity
//!
//! Represents a variant of a product with specific attributes like color, size, etc.
//! Variants share the same base product but have different SKUs, barcodes, and price deltas.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// Product Variant domain entity
///
/// A variant represents a specific version of a product with unique attributes.
/// For example, a "T-Shirt" product may have variants for different colors and sizes.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ProductVariant {
    /// Primary key using UUID v7 (timestamp-based)
    pub variant_id: Uuid,

    /// Multi-tenancy: All queries must filter by tenant_id
    pub tenant_id: Uuid,

    /// Reference to the parent product
    pub parent_product_id: Uuid,

    /// Variant-specific attributes (e.g., {"color": "red", "size": "L"})
    pub variant_attributes: serde_json::Value,

    /// Unique SKU for this variant within tenant
    #[validate(length(min = 1, max = 100))]
    pub sku: String,

    /// Optional barcode for this variant
    #[validate(length(max = 100))]
    pub barcode: Option<String>,

    /// Price difference from parent product (in smallest currency unit: cents/xu)
    /// Positive = more expensive, Negative = cheaper
    pub price_difference: i64,

    /// Whether this variant is active
    pub is_active: bool,

    /// Audit fields
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl ProductVariant {
    /// Create a new product variant
    pub fn new(
        tenant_id: Uuid,
        parent_product_id: Uuid,
        sku: String,
        variant_attributes: serde_json::Value,
    ) -> Self {
        Self {
            variant_id: Uuid::now_v7(),
            tenant_id,
            parent_product_id,
            variant_attributes,
            sku,
            barcode: None,
            price_difference: 0,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    /// Check if variant is deleted
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Check if variant is available (active and not deleted)
    pub fn is_available(&self) -> bool {
        self.is_active && !self.is_deleted()
    }

    /// Get display name (SKU + attributes summary)
    pub fn display_name(&self) -> String {
        if let Some(attrs) = self.variant_attributes.as_object() {
            let attrs_str: Vec<String> =
                attrs.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
            format!("{} ({})", self.sku, attrs_str.join(", "))
        } else {
            self.sku.clone()
        }
    }

    /// Mark as deleted (soft delete)
    pub fn mark_deleted(&mut self) {
        self.deleted_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Update timestamps
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Activate this variant
    pub fn activate(&mut self) {
        self.is_active = true;
        self.touch();
    }

    /// Deactivate this variant
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.touch();
    }
}

/// Response DTO for product variant with joined parent product info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ProductVariantResponse {
    pub variant_id: Uuid,
    pub tenant_id: Uuid,
    pub parent_product_id: Uuid,
    pub variant_attributes: serde_json::Value,
    pub sku: String,
    pub barcode: Option<String>,
    pub price_difference: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Joined fields from parent product
    pub parent_product_name: Option<String>,
    pub parent_product_sku: Option<String>,
}

impl From<ProductVariant> for ProductVariantResponse {
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

impl ProductVariantResponse {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_variant() -> ProductVariant {
        ProductVariant::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "TEST-VAR-001".to_string(),
            json!({"color": "red", "size": "L"}),
        )
    }

    #[test]
    fn test_variant_new_creates_with_correct_defaults() {
        let tenant_id = Uuid::new_v4();
        let parent_product_id = Uuid::new_v4();
        let variant = ProductVariant::new(
            tenant_id,
            parent_product_id,
            "VAR-001".to_string(),
            json!({"color": "blue"}),
        );

        assert_eq!(variant.tenant_id, tenant_id);
        assert_eq!(variant.parent_product_id, parent_product_id);
        assert_eq!(variant.sku, "VAR-001");
        assert!(variant.is_active);
        assert_eq!(variant.price_difference, 0);
        assert!(variant.barcode.is_none());
        assert!(variant.deleted_at.is_none());
    }

    #[test]
    fn test_variant_new_generates_uuid_v7() {
        let variant = create_test_variant();
        assert_eq!(
            variant.variant_id.get_version(),
            Some(uuid::Version::SortRand),
            "Variant should use UUID v7"
        );
    }

    #[test]
    fn test_is_deleted_when_not_deleted() {
        let variant = create_test_variant();
        assert!(!variant.is_deleted());
    }

    #[test]
    fn test_is_deleted_when_deleted() {
        let mut variant = create_test_variant();
        variant.deleted_at = Some(Utc::now());
        assert!(variant.is_deleted());
    }

    #[test]
    fn test_is_available_all_conditions_true() {
        let variant = create_test_variant();
        assert!(variant.is_available());
    }

    #[test]
    fn test_is_available_when_inactive() {
        let mut variant = create_test_variant();
        variant.is_active = false;
        assert!(!variant.is_available());
    }

    #[test]
    fn test_is_available_when_deleted() {
        let mut variant = create_test_variant();
        variant.deleted_at = Some(Utc::now());
        assert!(!variant.is_available());
    }

    #[test]
    fn test_display_name_with_attributes() {
        let variant = create_test_variant();
        let display = variant.display_name();
        assert!(display.contains("TEST-VAR-001"));
        assert!(display.contains("color"));
        assert!(display.contains("size"));
    }

    #[test]
    fn test_mark_deleted_sets_timestamp() {
        let mut variant = create_test_variant();
        let before = Utc::now();
        variant.mark_deleted();
        let after = Utc::now();

        assert!(variant.deleted_at.is_some());
        let deleted_at = variant.deleted_at.unwrap();
        assert!(deleted_at >= before && deleted_at <= after);
    }

    #[test]
    fn test_activate_and_deactivate() {
        let mut variant = create_test_variant();

        variant.deactivate();
        assert!(!variant.is_active);

        variant.activate();
        assert!(variant.is_active);
    }
}
