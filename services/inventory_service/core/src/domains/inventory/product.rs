use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::domains::inventory::dto::common::validate_product_type;

/// Product tracking method for inventory management
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, sqlx::Type)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "product_tracking_method", rename_all = "snake_case")]
pub enum ProductTrackingMethod {
    /// No tracking required
    #[default]
    None,
    /// Lot/batch tracking
    Lot,
    /// Serial number tracking
    Serial,
}

impl std::fmt::Display for ProductTrackingMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Lot => write!(f, "lot"),
            Self::Serial => write!(f, "serial"),
        }
    }
}

impl std::str::FromStr for ProductTrackingMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "lot" => Ok(Self::Lot),
            "serial" => Ok(Self::Serial),
            _ => Err(format!("Invalid tracking method: {}", s)),
        }
    }
}

/// Product domain entity representing the Item Master
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    /// Primary key using UUID v7 (timestamp-based)
    pub product_id: Uuid,

    /// Multi-tenancy: All queries must filter by tenant_id
    pub tenant_id: Uuid,

    /// Core product identifiers
    #[validate(length(min = 1, max = 100))]
    pub sku: String,

    #[validate(length(min = 1, max = 255))]
    pub name: String,

    pub description: Option<String>,

    /// Product classification
    #[validate(custom(function = "validate_product_type"))]
    pub product_type: String,

    pub item_group_id: Option<Uuid>,

    /// Inventory tracking
    pub track_inventory: bool,

    pub tracking_method: ProductTrackingMethod,

    /// Units of measure
    pub default_uom_id: Option<Uuid>,

    /// Pricing (stored in smallest currency unit: cents/xu)
    pub sale_price: Option<i64>,
    pub cost_price: Option<i64>,

    #[validate(length(min = 3, max = 3))]
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
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Product {
    /// Create a new product
    pub fn new(
        tenant_id: Uuid,
        sku: String,
        name: String,
        product_type: String,
        currency_code: String,
    ) -> Self {
        Self {
            product_id: Uuid::now_v7(),
            tenant_id,
            sku,
            name,
            description: None,
            product_type,
            item_group_id: None,
            track_inventory: true,
            tracking_method: ProductTrackingMethod::None,
            default_uom_id: None,
            sale_price: None,
            cost_price: None,
            currency_code,
            weight_grams: None,
            dimensions: None,
            attributes: None,
            is_active: true,
            is_sellable: true,
            is_purchaseable: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    /// Check if product is deleted
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Check if product is available for sale
    pub fn is_available_for_sale(&self) -> bool {
        self.is_active && self.is_sellable && !self.is_deleted()
    }

    /// Check if product is available for purchase
    pub fn is_available_for_purchase(&self) -> bool {
        self.is_active && self.is_purchaseable && !self.is_deleted()
    }

    /// Get display name (name + sku)
    pub fn display_name(&self) -> String {
        format!("{} ({})", self.name, self.sku)
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
}

#[cfg(feature = "openapi")]
mod openapi {
    use super::*;
    use utoipa::ToSchema;

    #[derive(ToSchema)]
    #[schema(rename_all = "camelCase")]
    #[allow(dead_code)]
    pub struct ProductResponse {
        /// Primary key using UUID v7 (timestamp-based)
        pub product_id: Uuid,

        /// Multi-tenancy: All queries must filter by tenant_id
        pub tenant_id: Uuid,

        /// Core product identifiers
        pub sku: String,
        pub name: String,
        pub description: Option<String>,

        /// Product classification
        pub product_type: String,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use std::thread::sleep;
    use std::time::Duration;

    /// Helper function to create a test product with default values
    fn create_test_product() -> Product {
        Product::new(
            Uuid::new_v4(),
            "TEST-SKU-001".to_string(),
            "Test Product".to_string(),
            "goods".to_string(),
            "USD".to_string(),
        )
    }

    // =========================================================================
    // ProductTrackingMethod Enum Tests
    // =========================================================================

    #[test]
    fn test_tracking_method_display() {
        assert_eq!(ProductTrackingMethod::None.to_string(), "none");
        assert_eq!(ProductTrackingMethod::Lot.to_string(), "lot");
        assert_eq!(ProductTrackingMethod::Serial.to_string(), "serial");
    }

    #[test]
    fn test_tracking_method_from_str_valid() {
        assert_eq!(ProductTrackingMethod::from_str("none").unwrap(), ProductTrackingMethod::None);
        assert_eq!(ProductTrackingMethod::from_str("lot").unwrap(), ProductTrackingMethod::Lot);
        assert_eq!(
            ProductTrackingMethod::from_str("serial").unwrap(),
            ProductTrackingMethod::Serial
        );
    }

    #[test]
    fn test_tracking_method_from_str_invalid() {
        let result = ProductTrackingMethod::from_str("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid tracking method"));
    }

    #[test]
    fn test_tracking_method_default() {
        assert_eq!(ProductTrackingMethod::default(), ProductTrackingMethod::None);
    }

    // =========================================================================
    // Product Creation Tests
    // =========================================================================

    #[test]
    fn test_product_new_creates_with_correct_defaults() {
        let tenant_id = Uuid::new_v4();
        let product = Product::new(
            tenant_id,
            "SKU-001".to_string(),
            "Widget".to_string(),
            "goods".to_string(),
            "USD".to_string(),
        );

        assert_eq!(product.tenant_id, tenant_id);
        assert_eq!(product.sku, "SKU-001");
        assert_eq!(product.name, "Widget");
        assert_eq!(product.product_type, "goods");
        assert_eq!(product.currency_code, "USD");
        assert!(product.is_active);
        assert!(product.is_sellable);
        assert!(product.is_purchaseable);
        assert!(product.track_inventory);
        assert_eq!(product.tracking_method, ProductTrackingMethod::None);
        assert!(product.description.is_none());
        assert!(product.deleted_at.is_none());
    }

    #[test]
    fn test_product_new_generates_uuid_v7() {
        let product = create_test_product();
        // Use uuid crate's API to verify version instead of string parsing
        assert_eq!(
            product.product_id.get_version(),
            Some(uuid::Version::SortRand),
            "Product should use UUID v7"
        );
    }

    // =========================================================================
    // Product Business Logic Tests
    // =========================================================================

    #[test]
    fn test_is_deleted_when_not_deleted() {
        let product = create_test_product();
        assert!(!product.is_deleted());
    }

    #[test]
    fn test_is_deleted_when_deleted() {
        let mut product = create_test_product();
        product.deleted_at = Some(Utc::now());
        assert!(product.is_deleted());
    }

    #[test]
    fn test_is_available_for_sale_all_conditions_true() {
        let product = create_test_product();
        assert!(product.is_available_for_sale());
    }

    #[test]
    fn test_is_available_for_sale_when_inactive() {
        let mut product = create_test_product();
        product.is_active = false;
        assert!(!product.is_available_for_sale());
    }

    #[test]
    fn test_is_available_for_sale_when_not_sellable() {
        let mut product = create_test_product();
        product.is_sellable = false;
        assert!(!product.is_available_for_sale());
    }

    #[test]
    fn test_is_available_for_sale_when_deleted() {
        let mut product = create_test_product();
        product.deleted_at = Some(Utc::now());
        assert!(!product.is_available_for_sale());
    }

    #[test]
    fn test_is_available_for_purchase_all_conditions_true() {
        let product = create_test_product();
        assert!(product.is_available_for_purchase());
    }

    #[test]
    fn test_is_available_for_purchase_when_inactive() {
        let mut product = create_test_product();
        product.is_active = false;
        assert!(!product.is_available_for_purchase());
    }

    #[test]
    fn test_is_available_for_purchase_when_not_purchaseable() {
        let mut product = create_test_product();
        product.is_purchaseable = false;
        assert!(!product.is_available_for_purchase());
    }

    #[test]
    fn test_is_available_for_purchase_when_deleted() {
        let mut product = create_test_product();
        product.deleted_at = Some(Utc::now());
        assert!(!product.is_available_for_purchase());
    }

    #[test]
    fn test_display_name_format() {
        let mut product = create_test_product();
        product.name = "Test Widget".to_string();
        product.sku = "WDG-001".to_string();
        assert_eq!(product.display_name(), "Test Widget (WDG-001)");
    }

    #[test]
    fn test_mark_deleted_sets_timestamp() {
        let mut product = create_test_product();
        let before = Utc::now();
        product.mark_deleted();
        let after = Utc::now();

        assert!(product.deleted_at.is_some());
        let deleted_at = product.deleted_at.unwrap();
        assert!(deleted_at >= before && deleted_at <= after);
    }

    #[test]
    fn test_mark_deleted_updates_updated_at() {
        let mut product = create_test_product();
        let original_updated_at = product.updated_at;

        // Small delay to ensure timestamp difference
        sleep(Duration::from_millis(10));
        product.mark_deleted();

        assert!(product.updated_at > original_updated_at);
    }

    #[test]
    fn test_touch_updates_timestamp() {
        let mut product = create_test_product();
        let original_updated_at = product.updated_at;

        // Small delay to ensure timestamp difference
        sleep(Duration::from_millis(10));
        product.touch();

        assert!(product.updated_at > original_updated_at);
    }

    // =========================================================================
    // Edge Cases and Combined Conditions
    // =========================================================================

    #[test]
    fn test_product_all_optional_fields_none() {
        let product = create_test_product();
        assert!(product.description.is_none());
        assert!(product.item_group_id.is_none());
        assert!(product.default_uom_id.is_none());
        assert!(product.sale_price.is_none());
        assert!(product.cost_price.is_none());
        assert!(product.weight_grams.is_none());
        assert!(product.dimensions.is_none());
        assert!(product.attributes.is_none());
    }

    #[test]
    fn test_product_with_pricing() {
        let mut product = create_test_product();
        product.sale_price = Some(1999); // $19.99 in cents
        product.cost_price = Some(999); // $9.99 in cents

        assert_eq!(product.sale_price, Some(1999));
        assert_eq!(product.cost_price, Some(999));
    }

    #[test]
    fn test_product_service_type() {
        let product = Product::new(
            Uuid::new_v4(),
            "SVC-001".to_string(),
            "Consulting Service".to_string(),
            "service".to_string(),
            "USD".to_string(),
        );

        assert_eq!(product.product_type, "service");
    }

    #[test]
    fn test_product_consumable_type() {
        let product = Product::new(
            Uuid::new_v4(),
            "CSM-001".to_string(),
            "Office Supplies".to_string(),
            "consumable".to_string(),
            "USD".to_string(),
        );

        assert_eq!(product.product_type, "consumable");
    }
}
