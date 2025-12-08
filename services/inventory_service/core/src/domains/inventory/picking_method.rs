use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domains::inventory::dto::common::validate_picking_method_type;
use crate::domains::inventory::BaseEntity;

/// Picking method domain entity representing a picking strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PickingMethod {
    /// Primary key using UUID v7 (timestamp-based)
    pub method_id: Uuid,

    /// Multi-tenancy: All queries must filter by tenant_id
    pub tenant_id: Uuid,

    /// Method metadata
    #[validate(length(min = 1, max = 200))]
    pub name: String,

    pub description: Option<String>,

    /// Picking method type
    #[validate(custom(function = "validate_picking_method_type"))]
    pub method_type: String,

    /// Warehouse this method applies to
    pub warehouse_id: Uuid,

    /// Method configuration (JSON for flexible settings)
    pub config: serde_json::Value,

    /// Status flags
    pub is_active: bool,
    pub is_default: bool,

    /// Audit fields
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl BaseEntity for PickingMethod {
    fn id(&self) -> Uuid {
        self.method_id
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
        self.description.as_deref()
    }

    fn is_active(&self) -> bool {
        self.is_active
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

impl PickingMethod {
    /// Create a new picking method
    pub fn new(
        tenant_id: Uuid,
        name: String,
        method_type: String,
        warehouse_id: Uuid,
        config: serde_json::Value,
    ) -> Self {
        Self {
            method_id: Uuid::now_v7(),
            tenant_id,
            name,
            description: None,
            method_type,
            warehouse_id,
            config,
            is_active: true,
            is_default: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    /// Get method type display name
    pub fn method_type_display(&self) -> &'static str {
        let method_type = self.method_type.to_ascii_lowercase();
        match method_type.as_str() {
            "batch" => "Batch Picking",
            "cluster" => "Cluster Picking",
            "wave" => "Wave Picking",
            _ => "Unknown",
        }
    }

    /// Check if this method supports a specific optimization criteria
    pub fn supports_criteria(&self, criteria: &str) -> bool {
        // Check config for supported criteria: accept either an array of strings or a single string
        if let Some(supported_criteria) = self.config.get("supported_criteria") {
            if let Some(s) = supported_criteria.as_str() {
                return s == criteria;
            } else if let Some(array) = supported_criteria.as_array() {
                return array.iter().any(|c| c.as_str() == Some(criteria));
            }
        }
        false
    }

    /// Get method-specific configuration value
    pub fn get_config_value(&self, key: &str) -> Option<&serde_json::Value> {
        self.config.get(key)
    }
}
