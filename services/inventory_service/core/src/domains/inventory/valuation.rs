anthill-windsurf/services/inventory_service/core/src/domains/inventory/valuation.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supported inventory valuation methods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ValuationMethod {
    Fifo,
    Avco,
    Standard,
}

/// Inventory valuation entity representing current valuation for a product
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Valuation {
    /// Primary key using UUID v7
    pub valuation_id: Uuid,

    /// Multi-tenancy
    pub tenant_id: Uuid,

    /// Product relationship
    pub product_id: Uuid,

    /// Valuation method
    pub valuation_method: ValuationMethod,

    /// Current valuation data
    pub current_unit_cost: Option<i64>, // In cents, for FIFO/AVCO
    pub total_quantity: i64,
    pub total_value: i64, // In cents

    /// Standard cost (only used when method = 'standard')
    pub standard_cost: Option<i64>, // In cents

    /// Metadata
    pub last_updated: DateTime<Utc>,
    pub updated_by: Option<Uuid>, // User who last updated
}

impl Valuation {
    /// Create a new valuation
    pub fn new(
        tenant_id: Uuid,
        product_id: Uuid,
        valuation_method: ValuationMethod,
    ) -> Self {
        Self {
            valuation_id: Uuid::now_v7(),
            tenant_id,
            product_id,
            valuation_method,
            current_unit_cost: None,
            total_quantity: 0,
            total_value: 0,
            standard_cost: None,
            last_updated: Utc::now(),
            updated_by: None,
        }
    }

    /// Update valuation with new data
    pub fn update(&mut self, unit_cost: Option<i64>, quantity: i64, value: i64, updated_by: Option<Uuid>) {
        self.current_unit_cost = unit_cost;
        self.total_quantity = quantity;
        self.total_value = value;
        self.last_updated = Utc::now();
        self.updated_by = updated_by;
    }

    /// Set standard cost
    pub fn set_standard_cost(&mut self, cost: i64, updated_by: Option<Uuid>) {
        self.standard_cost = Some(cost);
        self.last_updated = Utc::now();
        self.updated_by = updated_by;
    }

    /// Change valuation method
    pub fn change_method(&mut self, new_method: ValuationMethod, updated_by: Option<Uuid>) {
        self.valuation_method = new_method;
        self.last_updated = Utc::now();
        self.updated_by = updated_by;
    }

    /// Get current unit cost based on method
    pub fn get_current_unit_cost(&self) -> Option<i64> {
        match self.valuation_method {
            ValuationMethod::Fifo | ValuationMethod::Avco => self.current_unit_cost,
            ValuationMethod::Standard => self.standard_cost,
        }
    }
}

/// Cost layer entity for FIFO valuation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationLayer {
    /// Primary key using UUID v7
    pub layer_id: Uuid,

    /// Multi-tenancy
    pub tenant_id: Uuid,

    /// Product relationship
    pub product_id: Uuid,

    /// Layer details
    pub quantity: i64, // Remaining quantity in this layer
    pub unit_cost: i64, // Cost per unit in cents
    pub total_value: i64, // Calculated total in cents

    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ValuationLayer {
    /// Create a new cost layer
    pub fn new(tenant_id: Uuid, product_id: Uuid, quantity: i64, unit_cost: i64) -> Self {
        let total_value = quantity * unit_cost;
        Self {
            layer_id: Uuid::now_v7(),
            tenant_id,
            product_id,
            quantity,
            unit_cost,
            total_value,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Consume quantity from this layer (for FIFO)
    pub fn consume(&mut self, quantity_to_consume: i64) -> i64 {
        if quantity_to_consume >= self.quantity {
            let consumed = self.quantity;
            self.quantity = 0;
            self.total_value = 0;
            self.updated_at = Utc::now();
            consumed
        } else {
            self.quantity -= quantity_to_consume;
            self.total_value = self.quantity * self.unit_cost;
            self.updated_at = Utc::now();
            quantity_to_consume
        }
    }

    /// Check if layer is empty
    pub fn is_empty(&self) -> bool {
        self.quantity == 0
    }
}

/// Historical valuation record for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationHistory {
    /// Primary key
    pub history_id: Uuid,

    /// Reference to valuation
    pub valuation_id: Uuid,

    /// Historical data snapshot
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub valuation_method: ValuationMethod,
    pub unit_cost: Option<i64>,
    pub total_quantity: i64,
    pub total_value: i64,
    pub standard_cost: Option<i64>,

    /// Change metadata
    pub changed_at: DateTime<Utc>,
    pub changed_by: Option<Uuid>,
    pub change_reason: Option<String>,
}

impl ValuationHistory {
    /// Create a new history record
    pub fn new(
        valuation_id: Uuid,
        tenant_id: Uuid,
        product_id: Uuid,
        valuation_method: ValuationMethod,
        unit_cost: Option<i64>,
        total_quantity: i64,
        total_value: i64,
        standard_cost: Option<i64>,
        changed_by: Option<Uuid>,
        change_reason: Option<String>,
    ) -> Self {
        Self {
            history_id: Uuid::now_v7(),
            valuation_id,
            tenant_id,
            product_id,
            valuation_method,
            unit_cost,
            total_quantity,
            total_value,
            standard_cost,
            changed_at: Utc::now(),
            changed_by,
            change_reason,
        }
    }
}

#[cfg(feature = "openapi")]
mod openapi {
    use super::*;
    use utoipa::ToSchema;

    #[derive(ToSchema)]
    #[allow(dead_code)]
    pub struct ValuationResponse {
        pub valuation_id: Uuid,
        pub tenant_id: Uuid,
        pub product_id: Uuid,
        pub valuation_method: ValuationMethod,
        pub current_unit_cost: Option<i64>,
        pub total_quantity: i64,
        pub total_value: i64,
        pub standard_cost: Option<i64>,
        pub last_updated: DateTime<Utc>,
    }

    impl From<Valuation> for ValuationResponse {
        fn from(valuation: Valuation) -> Self {
            Self {
                valuation_id: valuation.valuation_id,
                tenant_id: valuation.tenant_id,
                product_id: valuation.product_id,
                valuation_method: valuation.valuation_method,
                current_unit_cost: valuation.current_unit_cost,
                total_quantity: valuation.total_quantity,
                total_value: valuation.total_value,
                standard_cost: valuation.standard_cost,
                last_updated: valuation.last_updated,
            }
        }
    }

    #[derive(ToSchema)]
    #[allow(dead_code)]
    pub struct ValuationLayerResponse {
        pub layer_id: Uuid,
        pub tenant_id: Uuid,
        pub product_id: Uuid,
        pub quantity: i64,
        pub unit_cost: i64,
        pub total_value: i64,
        pub created_at: DateTime<Utc>,
    }

    impl From<ValuationLayer> for ValuationLayerResponse {
        fn from(layer: ValuationLayer) -> Self {
            Self {
                layer_id: layer.layer_id,
                tenant_id: layer.tenant_id,
                product_id: layer.product_id,
                quantity: layer.quantity,
                unit_cost: layer.unit_cost,
                total_value: layer.total_value,
                created_at: layer.created_at,
            }
        }
    }
}
