use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domains::inventory::dto::common::{
    validate_location_type, validate_warehouse_type, validate_zone_type,
};

/// Request DTO for creating a new warehouse
#[derive(Debug, Clone, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CreateWarehouseRequest {
    /// Warehouse code (unique per tenant)
    #[validate(length(min = 1, max = 50))]
    pub warehouse_code: String,

    /// Warehouse name
    #[validate(length(min = 1, max = 255))]
    pub warehouse_name: String,

    /// Optional description
    #[validate(length(max = 1000))]
    pub description: Option<String>,

    /// Warehouse type
    #[validate(custom(function = "validate_warehouse_type"))]
    pub warehouse_type: String,

    /// Parent warehouse ID for hierarchy (optional)
    pub parent_warehouse_id: Option<Uuid>,

    /// Address information
    pub address: Option<serde_json::Value>,

    /// Contact information
    pub contact_info: Option<serde_json::Value>,

    /// Capacity information
    pub capacity_info: Option<serde_json::Value>,
}

/// Request DTO for creating a warehouse zone
#[derive(Debug, Clone, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CreateWarehouseZoneRequest {
    /// Zone code (unique per warehouse)
    #[validate(length(min = 1, max = 50))]
    pub zone_code: String,

    /// Zone name
    #[validate(length(min = 1, max = 255))]
    pub zone_name: String,

    /// Optional description
    #[validate(length(max = 1000))]
    pub description: Option<String>,

    /// Zone type
    #[validate(custom(function = "validate_zone_type"))]
    pub zone_type: String,

    /// Zone attributes
    pub zone_attributes: Option<serde_json::Value>,

    /// Capacity information
    pub capacity_info: Option<serde_json::Value>,
}

/// Request DTO for creating a warehouse location
#[derive(Debug, Clone, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CreateWarehouseLocationRequest {
    /// Zone ID (optional - location can exist without zone)
    pub zone_id: Option<Uuid>,

    /// Location code (unique per warehouse)
    #[validate(length(min = 1, max = 100))]
    pub location_code: String,

    /// Optional location name
    #[validate(length(max = 255))]
    pub location_name: Option<String>,

    /// Optional description
    #[validate(length(max = 1000))]
    pub description: Option<String>,

    /// Location type
    #[validate(custom(function = "validate_location_type"))]
    pub location_type: String,

    /// Physical coordinates
    pub coordinates: Option<serde_json::Value>,

    /// Dimensions
    pub dimensions: Option<serde_json::Value>,

    /// Capacity information
    pub capacity_info: Option<serde_json::Value>,

    /// Location attributes
    pub location_attributes: Option<serde_json::Value>,
}

/// Warehouse response DTO
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct WarehouseResponse {
    /// Warehouse ID
    pub warehouse_id: Uuid,

    /// Tenant ID
    pub tenant_id: Uuid,

    /// Warehouse identifiers
    pub warehouse_code: String,
    pub warehouse_name: String,
    pub description: Option<String>,

    /// Warehouse type
    pub warehouse_type: String,

    /// Hierarchy
    pub parent_warehouse_id: Option<Uuid>,

    /// Location and contact info
    pub address: Option<serde_json::Value>,
    pub contact_info: Option<serde_json::Value>,

    /// Capacity info
    pub capacity_info: Option<serde_json::Value>,

    /// Status
    pub is_active: bool,

    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::domains::inventory::warehouse::Warehouse> for WarehouseResponse {
    fn from(warehouse: crate::domains::inventory::warehouse::Warehouse) -> Self {
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

/// Warehouse zone response DTO
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct WarehouseZoneResponse {
    /// Zone ID
    pub zone_id: Uuid,

    /// Tenant ID
    pub tenant_id: Uuid,

    /// Warehouse ID
    pub warehouse_id: Uuid,

    /// Zone identifiers
    pub zone_code: String,
    pub zone_name: String,
    pub description: Option<String>,

    /// Zone type
    pub zone_type: String,

    /// Zone properties
    pub zone_attributes: Option<serde_json::Value>,

    /// Capacity info
    pub capacity_info: Option<serde_json::Value>,

    /// Status
    pub is_active: bool,

    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Warehouse location response DTO
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct WarehouseLocationResponse {
    /// Location ID
    pub location_id: Uuid,

    /// Tenant ID
    pub tenant_id: Uuid,

    /// Warehouse and zone IDs
    pub warehouse_id: Uuid,
    pub zone_id: Option<Uuid>,

    /// Location identifiers
    pub location_code: String,
    pub location_name: Option<String>,
    pub description: Option<String>,

    /// Location type
    pub location_type: String,

    /// Physical properties
    pub coordinates: Option<serde_json::Value>,
    pub dimensions: Option<serde_json::Value>,

    /// Capacity and attributes
    pub capacity_info: Option<serde_json::Value>,
    pub location_attributes: Option<serde_json::Value>,

    /// Status
    pub is_active: bool,

    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Warehouse tree node for hierarchical representation
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct WarehouseTreeNode {
    /// Warehouse data
    #[serde(flatten)]
    pub warehouse: WarehouseResponse,

    /// Child warehouses
    pub children: Vec<WarehouseTreeNode>,

    /// Zones in this warehouse
    pub zones: Vec<WarehouseZoneWithLocations>,
}

/// Warehouse zone with its locations
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct WarehouseZoneWithLocations {
    /// Zone data
    #[serde(flatten)]
    pub zone: WarehouseZoneResponse,

    /// Locations in this zone
    pub locations: Vec<WarehouseLocationResponse>,
}

/// Warehouse hierarchy/tree response
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct WarehouseTreeResponse {
    /// Root warehouses (those without parent)
    pub roots: Vec<WarehouseTreeNode>,

    /// Total count of warehouses
    pub total_warehouses: u32,

    /// Total count of zones
    pub total_zones: u32,

    /// Total count of locations
    pub total_locations: u32,
}

impl From<crate::domains::inventory::warehouse_zone::WarehouseZone> for WarehouseZoneResponse {
    fn from(zone: crate::domains::inventory::warehouse_zone::WarehouseZone) -> Self {
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

impl From<crate::domains::inventory::warehouse_location::WarehouseLocation>
    for WarehouseLocationResponse
{
    fn from(location: crate::domains::inventory::warehouse_location::WarehouseLocation) -> Self {
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
