use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domains::inventory::dto::common::validate_warehouse_type;
use crate::domains::inventory::BaseEntity;
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

impl BaseEntity for Warehouse {
    fn id(&self) -> Uuid {
        self.warehouse_id
    }

    fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    fn code(&self) -> &str {
        &self.warehouse_code
    }

    fn name(&self) -> &str {
        &self.warehouse_name
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a test warehouse
    fn create_test_warehouse() -> Warehouse {
        Warehouse::new(
            Uuid::new_v4(),
            "WH-001".to_string(),
            "Main Warehouse".to_string(),
            "main".to_string(),
        )
    }

    // =========================================================================
    // Warehouse Creation Tests
    // =========================================================================

    #[test]
    fn test_warehouse_new_creates_with_correct_defaults() {
        let tenant_id = Uuid::new_v4();
        let warehouse = Warehouse::new(
            tenant_id,
            "WH-001".to_string(),
            "Test Warehouse".to_string(),
            "main".to_string(),
        );

        assert_eq!(warehouse.tenant_id, tenant_id);
        assert_eq!(warehouse.warehouse_code, "WH-001");
        assert_eq!(warehouse.warehouse_name, "Test Warehouse");
        assert_eq!(warehouse.warehouse_type, "main");
        assert!(warehouse.is_active);
        assert!(warehouse.parent_warehouse_id.is_none());
        assert!(warehouse.description.is_none());
        assert!(warehouse.address.is_none());
        assert!(warehouse.contact_info.is_none());
        assert!(warehouse.capacity_info.is_none());
        assert!(warehouse.deleted_at.is_none());
    }

    #[test]
    fn test_warehouse_new_generates_uuid_v7() {
        let warehouse = create_test_warehouse();
        // Use uuid crate's API to verify version instead of string parsing
        assert_eq!(
            warehouse.warehouse_id.get_version(),
            Some(uuid::Version::SortRand),
            "Warehouse should use UUID v7"
        );
    }

    // =========================================================================
    // is_root Tests
    // =========================================================================

    #[test]
    fn test_is_root_when_no_parent() {
        let warehouse = create_test_warehouse();
        assert!(warehouse.is_root());
    }

    #[test]
    fn test_is_root_when_has_parent() {
        let mut warehouse = create_test_warehouse();
        warehouse.parent_warehouse_id = Some(Uuid::new_v4());
        assert!(!warehouse.is_root());
    }

    // =========================================================================
    // warehouse_type_display Tests
    // =========================================================================

    #[test]
    fn test_warehouse_type_display_main() {
        let mut warehouse = create_test_warehouse();
        warehouse.warehouse_type = "main".to_string();
        assert_eq!(warehouse.warehouse_type_display(), "Main Warehouse");
    }

    #[test]
    fn test_warehouse_type_display_transit() {
        let mut warehouse = create_test_warehouse();
        warehouse.warehouse_type = "transit".to_string();
        assert_eq!(warehouse.warehouse_type_display(), "Transit Hub");
    }

    #[test]
    fn test_warehouse_type_display_quarantine() {
        let mut warehouse = create_test_warehouse();
        warehouse.warehouse_type = "quarantine".to_string();
        assert_eq!(warehouse.warehouse_type_display(), "Quarantine Area");
    }

    #[test]
    fn test_warehouse_type_display_distribution() {
        let mut warehouse = create_test_warehouse();
        warehouse.warehouse_type = "distribution".to_string();
        assert_eq!(warehouse.warehouse_type_display(), "Distribution Center");
    }

    #[test]
    fn test_warehouse_type_display_retail() {
        let mut warehouse = create_test_warehouse();
        warehouse.warehouse_type = "retail".to_string();
        assert_eq!(warehouse.warehouse_type_display(), "Retail Store");
    }

    #[test]
    fn test_warehouse_type_display_satellite() {
        let mut warehouse = create_test_warehouse();
        warehouse.warehouse_type = "satellite".to_string();
        assert_eq!(warehouse.warehouse_type_display(), "Satellite Location");
    }

    #[test]
    fn test_warehouse_type_display_unknown() {
        let mut warehouse = create_test_warehouse();
        warehouse.warehouse_type = "custom_type".to_string();
        assert_eq!(warehouse.warehouse_type_display(), "Unknown");
    }

    // =========================================================================
    // BaseEntity Trait Tests
    // =========================================================================

    #[test]
    fn test_base_entity_id() {
        let warehouse = create_test_warehouse();
        assert_eq!(warehouse.id(), warehouse.warehouse_id);
    }

    #[test]
    fn test_base_entity_tenant_id() {
        let warehouse = create_test_warehouse();
        assert_eq!(BaseEntity::tenant_id(&warehouse), warehouse.tenant_id);
    }

    #[test]
    fn test_base_entity_code() {
        let warehouse = create_test_warehouse();
        assert_eq!(warehouse.code(), "WH-001");
    }

    #[test]
    fn test_base_entity_name() {
        let warehouse = create_test_warehouse();
        assert_eq!(BaseEntity::name(&warehouse), "Main Warehouse");
    }

    #[test]
    fn test_base_entity_description_none() {
        let warehouse = create_test_warehouse();
        assert!(BaseEntity::description(&warehouse).is_none());
    }

    #[test]
    fn test_base_entity_description_some() {
        let mut warehouse = create_test_warehouse();
        warehouse.description = Some("Test description".to_string());
        assert_eq!(BaseEntity::description(&warehouse), Some("Test description"));
    }

    #[test]
    fn test_base_entity_is_active() {
        let warehouse = create_test_warehouse();
        assert!(BaseEntity::is_active(&warehouse));
    }

    #[test]
    fn test_base_entity_is_deleted_false() {
        let warehouse = create_test_warehouse();
        assert!(!BaseEntity::is_deleted(&warehouse));
    }

    #[test]
    fn test_base_entity_is_deleted_true() {
        let mut warehouse = create_test_warehouse();
        warehouse.deleted_at = Some(Utc::now());
        assert!(BaseEntity::is_deleted(&warehouse));
    }

    #[test]
    fn test_base_entity_is_active_status() {
        let warehouse = create_test_warehouse();
        assert!(warehouse.is_active_status());
    }

    #[test]
    fn test_base_entity_is_active_status_when_deleted() {
        let mut warehouse = create_test_warehouse();
        warehouse.deleted_at = Some(Utc::now());
        assert!(!warehouse.is_active_status());
    }

    #[test]
    fn test_base_entity_is_active_status_when_inactive() {
        let mut warehouse = create_test_warehouse();
        warehouse.is_active = false;
        assert!(!warehouse.is_active_status());
    }

    #[test]
    fn test_base_entity_display_name() {
        let warehouse = create_test_warehouse();
        assert_eq!(BaseEntity::display_name(&warehouse), "Main Warehouse (WH-001)");
    }

    #[test]
    fn test_base_entity_mark_deleted() {
        let mut warehouse = create_test_warehouse();
        let before = Utc::now();
        warehouse.mark_deleted();

        assert!(warehouse.deleted_at.is_some());
        assert!(warehouse.deleted_at.unwrap() >= before);
    }

    #[test]
    fn test_base_entity_touch() {
        let mut warehouse = create_test_warehouse();
        let original = warehouse.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        warehouse.touch();

        assert!(warehouse.updated_at > original);
    }

    // Note: validate_warehouse_type tests are in domains/inventory/dto/common.rs
    // to avoid duplication and centralize validation testing
}
