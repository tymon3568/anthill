use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domains::inventory::dto::common::validate_zone_type;
use crate::domains::inventory::BaseEntity;

/// Warehouse zone domain entity representing zones within warehouses
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseZone {
    /// Primary key using UUID v7 (timestamp-based)
    pub zone_id: Uuid,

    /// Multi-tenancy: All queries must filter by tenant_id
    pub tenant_id: Uuid,

    /// Warehouse relationship
    pub warehouse_id: Uuid,

    /// Zone identifiers
    #[validate(length(min = 1, max = 50))]
    pub zone_code: String,

    #[validate(length(min = 1, max = 255))]
    pub zone_name: String,

    pub description: Option<String>,

    /// Zone classification
    #[validate(custom(function = "validate_zone_type"))]
    pub zone_type: String,

    /// Zone properties
    pub zone_attributes: Option<serde_json::Value>,

    /// Capacity information
    pub capacity_info: Option<serde_json::Value>,

    /// Status
    pub is_active: bool,

    /// Audit fields
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl BaseEntity for WarehouseZone {
    fn id(&self) -> Uuid {
        self.zone_id
    }

    fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    fn code(&self) -> &str {
        &self.zone_code
    }

    fn name(&self) -> &str {
        &self.zone_name
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

impl WarehouseZone {
    /// Create a new warehouse zone
    pub fn new(
        tenant_id: Uuid,
        warehouse_id: Uuid,
        zone_code: String,
        zone_name: String,
        zone_type: String,
    ) -> Self {
        Self {
            zone_id: Uuid::now_v7(),
            tenant_id,
            warehouse_id,
            zone_code,
            zone_name,
            description: None,
            zone_type,
            zone_attributes: None,
            capacity_info: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    /// Get zone type display name
    pub fn zone_type_display(&self) -> &'static str {
        match self.zone_type.as_str() {
            "storage" => "Storage Area",
            "picking" => "Picking Zone",
            "quarantine" => "Quarantine Area",
            "receiving" => "Receiving Dock",
            "shipping" => "Shipping Area",
            "bulk" => "Bulk Storage",
            "damaged" => "Damaged Goods",
            "returns" => "Returns Processing",
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
    pub struct WarehouseZoneResponse {
        /// Primary key using UUID v7 (timestamp-based)
        pub zone_id: Uuid,

        /// Multi-tenancy: All queries must filter by tenant_id
        pub tenant_id: Uuid,

        /// Warehouse relationship
        pub warehouse_id: Uuid,

        /// Zone identifiers
        pub zone_code: String,
        pub zone_name: String,
        pub description: Option<String>,

        /// Zone classification
        pub zone_type: String,

        /// Zone properties
        pub zone_attributes: Option<serde_json::Value>,

        /// Capacity information
        pub capacity_info: Option<serde_json::Value>,

        /// Status
        pub is_active: bool,

        /// Audit fields
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    impl From<WarehouseZone> for WarehouseZoneResponse {
        fn from(zone: WarehouseZone) -> Self {
            Self {
                zone_id: zone.zone_id,
                tenant_id: zone.tenant_id,
                warehouse_id: zone.warehouse_id,
                zone_code: zone.zone_code,
                zone_name: zone.zone_name,
                description: zone.description,
                zone_type: zone.zone_type,
                zone_attributes: zone.zone_attributes,
                capacity_info: zone.capacity_info,
                is_active: zone.is_active,
                created_at: zone.created_at,
                updated_at: zone.updated_at,
            }
        }
    }
}
