use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// Role Management DTOs
// ============================================================================

/// Request to create a custom role
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateRoleReq {
    /// Role name (e.g., "inventory_manager", "sales_staff")
    #[validate(length(min = 1, max = 100))]
    #[validate(regex(
        path = "RE_ROLE_NAME",
        message = "Role name must be lowercase alphanumeric with underscores"
    ))]
    pub role_name: String,

    /// Human-readable role description
    #[validate(length(max = 500))]
    pub description: Option<String>,

    /// List of permissions for this role
    #[validate(length(min = 1))]
    pub permissions: Vec<PermissionReq>,
}

/// Permission definition for a role
#[derive(Debug, Deserialize, Serialize, Validate, ToSchema, Clone)]
pub struct PermissionReq {
    /// Resource (e.g., "products", "orders", "inventory")
    #[validate(length(min = 1, max = 100))]
    pub resource: String,

    /// Action (e.g., "read", "write", "delete", "approve")
    #[validate(length(min = 1, max = 50))]
    pub action: String,
}

lazy_static::lazy_static! {
    static ref RE_ROLE_NAME: regex::Regex = regex::Regex::new(r"^[a-z0-9_]+$").unwrap();
}

/// Response for role creation
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateRoleResp {
    pub role_name: String,
    pub description: Option<String>,
    pub permissions_count: usize,
    pub message: String,
}

/// Response for listing roles
#[derive(Debug, Serialize, ToSchema)]
pub struct RoleListResp {
    pub roles: Vec<RoleInfo>,
    pub total: usize,
}

/// Information about a role
#[derive(Debug, Serialize, ToSchema)]
pub struct RoleInfo {
    pub role_name: String,
    pub description: Option<String>,
    pub permissions: Vec<PermissionInfo>,
    pub user_count: usize,
}

/// Permission information
#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionInfo {
    pub resource: String,
    pub action: String,
}

/// Request to update role permissions
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateRoleReq {
    /// Updated description
    #[validate(length(max = 500))]
    pub description: Option<String>,

    /// New list of permissions (replaces existing)
    #[validate(length(min = 1))]
    pub permissions: Vec<PermissionReq>,
}

/// Response for role update
#[derive(Debug, Serialize, ToSchema)]
pub struct UpdateRoleResp {
    pub role_name: String,
    pub permissions_count: usize,
    pub message: String,
}

/// Response for role deletion
#[derive(Debug, Serialize, ToSchema)]
pub struct DeleteRoleResp {
    pub role_name: String,
    pub message: String,
}

// ============================================================================
// User Role Assignment DTOs
// ============================================================================

/// Request to assign role to user
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AssignUserRoleReq {
    /// Role name to assign
    #[validate(length(min = 1, max = 100))]
    pub role_name: String,
}

/// Response for role assignment
#[derive(Debug, Serialize, ToSchema)]
pub struct AssignUserRoleResp {
    pub user_id: Uuid,
    pub role_name: String,
    pub message: String,
}

/// Response for role removal
#[derive(Debug, Serialize, ToSchema)]
pub struct RemoveUserRoleResp {
    pub user_id: Uuid,
    pub role_name: String,
    pub message: String,
}

/// Response for listing user's roles
#[derive(Debug, Serialize, ToSchema)]
pub struct UserRolesResp {
    pub user_id: Uuid,
    pub roles: Vec<String>,
}

// ============================================================================
// Permission Management DTOs
// ============================================================================

/// Response for listing all available permissions
#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionListResp {
    pub permissions: Vec<AvailablePermission>,
    pub total: usize,
}

/// Available permission in the system
#[derive(Debug, Serialize, ToSchema)]
pub struct AvailablePermission {
    pub resource: String,
    pub actions: Vec<String>,
    pub description: String,
}

// ============================================================================
// Policy Management DTOs (Direct Casbin manipulation)
// ============================================================================

/// Request to add a policy directly
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AddPolicyReq {
    #[validate(length(min = 1, max = 255))]
    pub role: String,

    #[validate(length(min = 1, max = 255))]
    pub resource: String,

    #[validate(length(min = 1, max = 255))]
    pub action: String,
}

/// Request to remove a policy
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RemovePolicyReq {
    #[validate(length(min = 1))]
    pub role: String,

    #[validate(length(min = 1))]
    pub resource: String,

    #[validate(length(min = 1))]
    pub action: String,
}

/// Response for policy operations
#[derive(Debug, Serialize, ToSchema)]
pub struct PolicyResp {
    pub message: String,
    pub role: String,
    pub resource: String,
    pub action: String,
}
