use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domains::inventory::dto::common::validate_location_type;
use crate::domains::inventory::BaseEntity;

/// Warehouse location domain entity representing storage positions within warehouses
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseLocation {
    /// Primary key using UUID v7 (timestamp-based)
    pub location_id: Uuid,

    /// Multi-tenancy: All queries must filter by tenant_id
    pub tenant_id: Uuid,

    /// Warehouse and zone relationships
    pub warehouse_id: Uuid,
    pub zone_id: Option<Uuid>,

    /// Location identifiers
    #[validate(length(min = 1, max = 100))]
    pub location_code: String,

    pub location_name: Option<String>,
    pub description: Option<String>,

    /// Location classification
    #[validate(custom(function = "validate_location_type"))]
    pub location_type: String,

    /// Physical coordinates and dimensions
    pub coordinates: Option<serde_json::Value>,
    pub dimensions: Option<serde_json::Value>,

    /// Capacity and operational data
    pub capacity_info: Option<serde_json::Value>,

    /// Location properties
    pub location_attributes: Option<serde_json::Value>,

    /// Status
    pub is_active: bool,

    /// Audit fields
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl BaseEntity for WarehouseLocation {
    fn id(&self) -> Uuid {
        self.location_id
    }

    fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    fn code(&self) -> &str {
        &self.location_code
    }

    fn name(&self) -> &str {
        self.location_name.as_deref().unwrap_or(&self.location_code)
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

impl WarehouseLocation {
    /// Create a new warehouse location
    pub fn new(
        tenant_id: Uuid,
        warehouse_id: Uuid,
        location_code: String,
        location_type: String,
    ) -> Self {
        Self {
            location_id: Uuid::now_v7(),
            tenant_id,
            warehouse_id,
            zone_id: None,
            location_code,
            location_name: None,
            description: None,
            location_type,
            coordinates: None,
            dimensions: None,
            capacity_info: None,
            location_attributes: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    /// Get location type display name
    pub fn location_type_display(&self) -> &'static str {
        match self.location_type.as_str() {
            "bin" => "Storage Bin",
            "shelf" => "Shelf",
            "pallet" => "Pallet Position",
            "floor" => "Floor Space",
            "rack" => "Rack Position",
            "container" => "Container",
            "bulk" => "Bulk Storage",
            _ => "Unknown",
        }
    }

    /// Check if location has zone assignment
    pub fn has_zone(&self) -> bool {
        self.zone_id.is_some()
    }
}

#[cfg(feature = "openapi")]
mod openapi {
    use super::*;
    use utoipa::ToSchema;

    #[derive(ToSchema)]
    #[schema(rename_all = "camelCase")]
    #[allow(dead_code)]
    pub struct WarehouseLocationResponse {
        /// Primary key using UUID v7 (timestamp-based)
        pub location_id: Uuid,

        /// Multi-tenancy: All queries must filter by tenant_id
        pub tenant_id: Uuid,

        /// Warehouse and zone relationships
        pub warehouse_id: Uuid,
        pub zone_id: Option<Uuid>,

        /// Location identifiers
        pub location_code: String,
        pub location_name: Option<String>,
        pub description: Option<String>,

        /// Location classification
        pub location_type: String,

        /// Physical coordinates and dimensions
        pub coordinates: Option<serde_json::Value>,
        pub dimensions: Option<serde_json::Value>,

        /// Capacity and operational data
        pub capacity_info: Option<serde_json::Value>,

        /// Location properties
        pub location_attributes: Option<serde_json::Value>,

        /// Status
        pub is_active: bool,

        /// Audit fields
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    impl From<WarehouseLocation> for WarehouseLocationResponse {
        fn from(location: WarehouseLocation) -> Self {
            Self {
                location_id: location.location_id,
                tenant_id: location.tenant_id,
                warehouse_id: location.warehouse_id,
                zone_id: location.zone_id,
                location_code: location.location_code,
                location_name: location.location_name,
                description: location.description,
                location_type: location.location_type,
                coordinates: location.coordinates,
                dimensions: location.dimensions,
                capacity_info: location.capacity_info,
                location_attributes: location.location_attributes,
                is_active: location.is_active,
                created_at: location.created_at,
                updated_at: location.updated_at,
            }
        }
    }
}
