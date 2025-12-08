use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::domains::inventory::dto::common::validate_product_type;

/// Product tracking method for inventory management
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ProductTrackingMethod {
    /// No tracking required
    None,
    /// Lot/batch tracking
    Lot,
    /// Serial number tracking
    Serial,
}

impl Default for ProductTrackingMethod {
    fn default() -> Self {
        Self::None
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
