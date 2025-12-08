use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domains::inventory::dto::common::validate_removal_strategy_type;
use crate::domains::inventory::BaseEntity;

/// Removal strategy domain entity representing a stock removal strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RemovalStrategy {
    /// Primary key using UUID v7 (timestamp-based)
    pub strategy_id: Uuid,

    /// Multi-tenancy: All queries must filter by tenant_id
    pub tenant_id: Uuid,

    /// Strategy metadata
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    /// Removal strategy type
    #[validate(custom(function = "validate_removal_strategy_type"))]
    pub strategy_type: String,

    /// Scope: warehouse-wide or product-specific
    pub warehouse_id: Option<Uuid>,
    pub product_id: Option<Uuid>,

    /// Configuration
    pub active: bool,
    pub config: serde_json::Value,

    /// Audit fields
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
}

impl BaseEntity for RemovalStrategy {
    fn id(&self) -> Uuid {
        self.strategy_id
    }

    fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    fn code(&self) -> &str {
        &self.name
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        None // No description field
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }

    /// Mark as deleted (soft delete)
    fn mark_deleted(&mut self) {
        self.deleted_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Update timestamps
    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl RemovalStrategy {
    /// Create a new removal strategy
    pub fn new(
        tenant_id: Uuid,
        name: String,
        strategy_type: String,
        warehouse_id: Option<Uuid>,
        product_id: Option<Uuid>,
        config: serde_json::Value,
        created_by: Uuid,
    ) -> Self {
        Self {
            strategy_id: Uuid::now_v7(),
            tenant_id,
            name,
            strategy_type,
            warehouse_id,
            product_id,
            active: true,
            config,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            created_by,
            updated_by: None,
        }
    }

    /// Get strategy type display name
    pub fn strategy_type_display(&self) -> &'static str {
        match self.strategy_type.as_str() {
            "fifo" => "First In, First Out",
            "lifo" => "Last In, First Out",
            "fefo" => "First Expired, First Out",
            "closest_location" => "Closest Location",
            "least_packages" => "Least Packages",
            _ => "Unknown",
        }
    }

    /// Check if strategy applies to specific warehouse
    #[allow(clippy::unnecessary_map_or)]
    pub fn applies_to_warehouse(&self, warehouse_id: Uuid) -> bool {
        self.warehouse_id.map_or(true, |w_id| w_id == warehouse_id)
    }

    /// Check if strategy applies to specific product
    #[allow(clippy::unnecessary_map_or)]
    pub fn applies_to_product(&self, product_id: Uuid) -> bool {
        self.product_id.map_or(true, |p_id| p_id == product_id)
    }

    /// Get strategy-specific configuration value
    pub fn get_config_value(&self, key: &str) -> Option<&serde_json::Value> {
        self.config.get(key)
    }

    /// Get FEFO buffer days (default 0)
    pub fn fefo_buffer_days(&self) -> i32 {
        self.get_config_value("fefo_buffer_days")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32
    }

    /// Get location priorities for closest_location strategy
    pub fn location_priorities(&self) -> Vec<String> {
        self.get_config_value("location_priorities")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }
}
