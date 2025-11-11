use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Warehouse domain entity representing a warehouse in the hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Warehouse {
    /// Primary key using UUID v7 (timestamp-based)
    pub warehouse_id: Uuid,

    /// Multi-tenancy: All queries must filter by tenant_id
    pub tenant_id: Uuid,

    /// Warehouse identifiers
    #[validate(length(min = 1, max = 50))]
    pub warehouse_code: String,

    #[validate(length(min = 1, max = 255))]
    pub warehouse_name: String,

    pub description: Option<String>,

    /// Warehouse classification
    #[validate(custom(function = "validate_warehouse_type"))]
    pub warehouse_type: String,

    /// Hierarchy support (unlimited depth)
    pub parent_warehouse_id: Option<Uuid>,

    /// Location and contact information
    pub address: Option<serde_json::Value>,
    pub contact_info: Option<serde_json::Value>,

    /// Capacity and operational data
    pub capacity_info: Option<serde_json::Value>,

    /// Status
    pub is_active: bool,

    /// Audit fields
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Validation functions
fn validate_warehouse_type(warehouse_type: &str) -> Result<(), validator::ValidationError> {
    match warehouse_type {
        "main" | "transit" | "quarantine" | "distribution" | "retail" | "satellite" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_warehouse_type")),
    }
}

impl Warehouse {
    /// Create a new warehouse
    pub fn new(
        tenant_id: Uuid,
        warehouse_code: String,
        warehouse_name: String,
        warehouse_type: String,
    ) -> Self {
        Self {
            warehouse_id: Uuid::now_v7(),
            tenant_id,
            warehouse_code,
            warehouse_name,
            description: None,
            warehouse_type,
            parent_warehouse_id: None,
            address: None,
            contact_info: None,
            capacity_info: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    /// Check if warehouse is deleted
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Check if warehouse is active
    pub fn is_active(&self) -> bool {
        self.is_active && !self.is_deleted()
    }

    /// Get display name (code + name)
    pub fn display_name(&self) -> String {
        format!("{} ({})", self.warehouse_name, self.warehouse_code)
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

    /// Check if this is a root warehouse (no parent)
    pub fn is_root(&self) -> bool {
        self.parent_warehouse_id.is_none()
    }

    /// Get warehouse type display name
    pub fn warehouse_type_display(&self) -> &'static str {
        match self.warehouse_type.as_str() {
            "main" => "Main Warehouse",
            "transit" => "Transit Hub",
            "quarantine" => "Quarantine Area",
            "distribution" => "Distribution Center",
            "retail" => "Retail Store",
            "satellite" => "Satellite Location",
            _ => "Unknown",
        }
    }
}

#[cfg(feature = "openapi")]
mod openapi {
    use super::*;
    use utoipa::ToSchema;

    #[derive(ToSchema)]
    #[schema(rename_all = "camelCase")]
    #[allow(dead_code)]
    pub struct WarehouseResponse {
        /// Primary key using UUID v7 (timestamp-based)
        pub warehouse_id: Uuid,

        /// Multi-tenancy: All queries must filter by tenant_id
        pub tenant_id: Uuid,

        /// Warehouse identifiers
        pub warehouse_code: String,
        pub warehouse_name: String,
        pub description: Option<String>,

        /// Warehouse classification
        pub warehouse_type: String,

        /// Hierarchy support (unlimited depth)
        pub parent_warehouse_id: Option<Uuid>,

        /// Location and contact information
        pub address: Option<serde_json::Value>,
        pub contact_info: Option<serde_json::Value>,

        /// Capacity and operational data
        pub capacity_info: Option<serde_json::Value>,

        /// Status
        pub is_active: bool,

        /// Audit fields
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    impl From<Warehouse> for WarehouseResponse {
        fn from(warehouse: Warehouse) -> Self {
            Self {
                warehouse_id: warehouse.warehouse_id,
                tenant_id: warehouse.tenant_id,
                warehouse_code: warehouse.warehouse_code,
                warehouse_name: warehouse.warehouse_name,
                description: warehouse.description,
                warehouse_type: warehouse.warehouse_type,
                parent_warehouse_id: warehouse.parent_warehouse_id,
                address: warehouse.address,
                contact_info: warehouse.contact_info,
                capacity_info: warehouse.capacity_info,
                is_active: warehouse.is_active,
                created_at: warehouse.created_at,
                updated_at: warehouse.updated_at,
            }
        }
    }
}
