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
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum ProductTrackingMethod {
    /// No tracking required
    #[default]
    None,
    /// Lot/batch tracking
    Lot,
    /// Serial number tracking
    Serial,
}

/// Barcode type for product identification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, sqlx::Type)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum BarcodeType {
    /// EAN-13 (European Article Number)
    #[default]
    Ean13,
    /// UPC-A (Universal Product Code)
    UpcA,
    /// ISBN (International Standard Book Number)
    Isbn,
    /// Custom barcode format
    Custom,
}

impl std::fmt::Display for BarcodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ean13 => write!(f, "ean13"),
            Self::UpcA => write!(f, "upc_a"),
            Self::Isbn => write!(f, "isbn"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for BarcodeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ean13" => Ok(Self::Ean13),
            "upc_a" => Ok(Self::UpcA),
            "isbn" => Ok(Self::Isbn),
            "custom" => Ok(Self::Custom),
            _ => Err(format!("Invalid barcode type: {}", s)),
        }
    }
}

impl BarcodeType {
    /// Validate a barcode string against this barcode type's format
    ///
    /// # Arguments
    /// * `barcode` - The barcode string to validate
    ///
    /// # Returns
    /// * `Ok(())` if the barcode is valid for this type
    /// * `Err(String)` with a description of why the barcode is invalid
    pub fn validate_barcode(&self, barcode: &str) -> Result<(), String> {
        match self {
            Self::Ean13 => Self::validate_ean13(barcode),
            Self::UpcA => Self::validate_upc_a(barcode),
            Self::Isbn => Self::validate_isbn(barcode),
            Self::Custom => Ok(()), // Custom barcodes accept any format
        }
    }

    /// Validate EAN-13 barcode format
    /// EAN-13 must be exactly 13 digits with a valid check digit
    fn validate_ean13(barcode: &str) -> Result<(), String> {
        // Must be exactly 13 digits
        if barcode.len() != 13 {
            return Err(format!("EAN-13 must be exactly 13 digits, got {} digits", barcode.len()));
        }

        // Must be all digits
        if !barcode.chars().all(|c| c.is_ascii_digit()) {
            return Err("EAN-13 must contain only digits".to_string());
        }

        // Validate check digit
        if !Self::validate_ean_check_digit(barcode) {
            return Err("Invalid EAN-13 check digit".to_string());
        }

        Ok(())
    }

    /// Validate UPC-A barcode format
    /// UPC-A must be exactly 12 digits with a valid check digit
    fn validate_upc_a(barcode: &str) -> Result<(), String> {
        // Must be exactly 12 digits
        if barcode.len() != 12 {
            return Err(format!("UPC-A must be exactly 12 digits, got {} digits", barcode.len()));
        }

        // Must be all digits
        if !barcode.chars().all(|c| c.is_ascii_digit()) {
            return Err("UPC-A must contain only digits".to_string());
        }

        // Validate check digit using the same algorithm as EAN
        // (UPC-A is actually a subset of EAN-13 with leading 0)
        if !Self::validate_upc_check_digit(barcode) {
            return Err("Invalid UPC-A check digit".to_string());
        }

        Ok(())
    }

    /// Validate ISBN barcode format
    /// Accepts both ISBN-10 and ISBN-13 formats
    fn validate_isbn(barcode: &str) -> Result<(), String> {
        // Remove any hyphens or spaces
        let cleaned: String = barcode
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-')
            .collect();

        match cleaned.len() {
            10 => Self::validate_isbn10(&cleaned),
            13 => Self::validate_isbn13(&cleaned),
            _ => Err(format!(
                "ISBN must be 10 or 13 characters (excluding hyphens), got {}",
                cleaned.len()
            )),
        }
    }

    /// Validate ISBN-10 check digit
    fn validate_isbn10(isbn: &str) -> Result<(), String> {
        let chars: Vec<char> = isbn.chars().collect();

        // First 9 must be digits, last can be digit or X
        for (i, c) in chars.iter().enumerate() {
            if i < 9 {
                if !c.is_ascii_digit() {
                    return Err("ISBN-10 first 9 characters must be digits".to_string());
                }
            } else if !c.is_ascii_digit() && *c != 'X' && *c != 'x' {
                return Err("ISBN-10 check digit must be a digit or X".to_string());
            }
        }

        // Calculate check digit
        let mut sum = 0u32;
        for (i, c) in chars.iter().enumerate() {
            let value = if *c == 'X' || *c == 'x' {
                10
            } else {
                c.to_digit(10).unwrap()
            };
            sum += value * (10 - i as u32);
        }

        if sum % 11 != 0 {
            return Err("Invalid ISBN-10 check digit".to_string());
        }

        Ok(())
    }

    /// Validate ISBN-13 check digit (same as EAN-13)
    fn validate_isbn13(isbn: &str) -> Result<(), String> {
        // ISBN-13 must start with 978 or 979
        if !isbn.starts_with("978") && !isbn.starts_with("979") {
            return Err("ISBN-13 must start with 978 or 979".to_string());
        }

        // Must be all digits
        if !isbn.chars().all(|c| c.is_ascii_digit()) {
            return Err("ISBN-13 must contain only digits".to_string());
        }

        // Validate check digit (same algorithm as EAN-13)
        if !Self::validate_ean_check_digit(isbn) {
            return Err("Invalid ISBN-13 check digit".to_string());
        }

        Ok(())
    }

    /// Validate EAN/ISBN-13 check digit
    /// Uses the standard modulo 10 algorithm with weights 1 and 3
    fn validate_ean_check_digit(barcode: &str) -> bool {
        let digits: Vec<u32> = barcode.chars().filter_map(|c| c.to_digit(10)).collect();

        if digits.len() != 13 {
            return false;
        }

        let mut sum = 0;
        for (i, digit) in digits.iter().enumerate() {
            if i % 2 == 0 {
                sum += digit;
            } else {
                sum += digit * 3;
            }
        }

        sum % 10 == 0
    }

    /// Validate UPC-A check digit
    /// Uses the standard modulo 10 algorithm with weights 3 and 1
    fn validate_upc_check_digit(barcode: &str) -> bool {
        let digits: Vec<u32> = barcode.chars().filter_map(|c| c.to_digit(10)).collect();

        if digits.len() != 12 {
            return false;
        }

        let mut sum = 0;
        for (i, digit) in digits.iter().enumerate() {
            if i % 2 == 0 {
                sum += digit * 3;
            } else {
                sum += digit;
            }
        }

        sum % 10 == 0
    }
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

    /// Barcode for product identification (EAN-13, UPC-A, ISBN, or custom)
    #[validate(length(max = 50))]
    pub barcode: Option<String>,

    /// Type of barcode (ean13, upc_a, isbn, custom)
    pub barcode_type: Option<BarcodeType>,

    /// Category for product organization
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
            barcode: None,
            barcode_type: None,
            category_id: None,
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

        /// Barcode for product identification
        pub barcode: Option<String>,

        /// Type of barcode
        pub barcode_type: Option<BarcodeType>,

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
                barcode: product.barcode,
                barcode_type: product.barcode_type,
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
