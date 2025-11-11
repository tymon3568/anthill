use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Request DTO for creating a new warehouse
#[derive(Debug, Clone, Deserialize, Validate)]
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
    #[validate(custom(function = "validate_warehouse_type_req"))]
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
    #[validate(custom(function = "validate_zone_type_req"))]
    pub zone_type: String,

    /// Zone attributes
    pub zone_attributes: Option<serde_json::Value>,

    /// Capacity information
    pub capacity_info: Option<serde_json::Value>,
}

/// Request DTO for creating a warehouse location
#[derive(Debug, Clone, Deserialize, Validate)]
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
    #[validate(custom(function = "validate_location_type_req"))]
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

/// Validation functions for request DTOs
fn validate_warehouse_type_req(warehouse_type: &str) -> Result<(), validator::ValidationError> {
    match warehouse_type {
        "main" | "transit" | "quarantine" | "distribution" | "retail" | "satellite" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_warehouse_type")),
    }
}

fn validate_zone_type_req(zone_type: &str) -> Result<(), validator::ValidationError> {
    match zone_type {
        "storage" | "picking" | "quarantine" | "receiving" | "shipping" | "bulk" | "damaged"
        | "returns" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_zone_type")),
    }
}

fn validate_location_type_req(location_type: &str) -> Result<(), validator::ValidationError> {
    match location_type {
        "bin" | "shelf" | "pallet" | "floor" | "rack" | "container" | "bulk" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_location_type")),
    }
}
