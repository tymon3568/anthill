use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use regex::Regex;

static ROLE_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9_]*$").unwrap());

// ============================================================================
// Role Management DTOs
// ============================================================================

/// Request to create a custom role
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateRoleReq {
    /// Role name (e.g., "inventory_manager", "sales_staff")
    #[validate(length(min = 1, max = 100))]
    #[validate(regex(
        path = "ROLE_NAME_REGEX",
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
    #[validate(regex(
        path = "ROLE_NAME_REGEX",
        message = "Role name must be lowercase alphanumeric with underscores"
    ))]
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

// ============================================================================
// Admin User Management DTOs
// ============================================================================

/// System roles that are protected and cannot be created via admin create user
pub const SYSTEM_ROLES: &[&str] = &["owner", "admin", "user"];

/// Request to create a new user in the admin's tenant
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AdminCreateUserReq {
    /// Email address for the new user (must be unique within the tenant)
    #[validate(email(message = "Invalid email format"))]
    #[schema(example = "newuser@example.com")]
    pub email: String,

    /// Password for the new user (min 8 characters)
    /// Admin may set a temporary password that the user should change on first login
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    #[schema(example = "TempPass123!", min_length = 8)]
    pub password: String,

    /// Full name of the user (optional)
    #[validate(length(max = 255, message = "Full name too long"))]
    #[schema(example = "Jane Smith")]
    pub full_name: Option<String>,

    /// Role to assign to the user (default: "user")
    /// Must be a valid role in the tenant (system role or custom role)
    /// Note: Creating users with "owner" role is not allowed via this endpoint
    #[validate(length(min = 1, max = 100))]
    #[validate(regex(
        path = "ROLE_NAME_REGEX",
        message = "Role name must be lowercase alphanumeric with underscores"
    ))]
    #[schema(example = "user")]
    pub role: Option<String>,
}

/// Response for admin user creation
#[derive(Debug, Serialize, ToSchema)]
pub struct AdminCreateUserResp {
    /// Created user's ID (UUID v7)
    pub user_id: Uuid,

    /// Tenant ID the user belongs to
    pub tenant_id: Uuid,

    /// User's email address
    #[schema(example = "newuser@example.com")]
    pub email: String,

    /// User's full name (if provided)
    #[schema(example = "Jane Smith")]
    pub full_name: Option<String>,

    /// Assigned role
    #[schema(example = "user")]
    pub role: String,

    /// Account creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Success message
    #[schema(example = "User created successfully")]
    pub message: String,
}
