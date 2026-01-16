use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    Json,
};
use shared_auth::casbin::{CoreApi, MgmtApi};
use shared_auth::extractors::RequireAdmin;
use shared_error::AppError;
use std::collections::HashMap;
use user_service_core::domains::auth::domain::service::AuthService;
use user_service_core::domains::auth::dto::admin_dto::*;
use uuid::Uuid;
use validator::Validate;

use crate::handlers::{bump_tenant_authz_version, bump_user_authz_version, AppState};

// ============================================================================
// Role Management Handlers
// ============================================================================

/// Create a custom role with permissions
#[utoipa::path(
    post,
    path = "/api/v1/admin/roles",
    tag = "admin-roles",
    operation_id = "admin_create_role",
    request_body = CreateRoleReq,
    responses(
        (status = 201, description = "Role created successfully", body = CreateRoleResp),
        (status = 400, description = "Invalid request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden - Admin only", body = String),
        (status = 409, description = "Role already exists", body = String),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_role<S: AuthService>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
    Json(payload): Json<CreateRoleReq>,
) -> Result<(StatusCode, Json<CreateRoleResp>), AppError> {
    // Validate request
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let tenant_id = admin_user.tenant_id;
    let role_name = payload.role_name.trim().to_lowercase();

    // Prevent creating system roles
    if role_name == "admin" || role_name == "user" {
        return Err(AppError::Forbidden("Cannot create system roles (admin, user)".to_string()));
    }

    // Require at least one permission
    if payload.permissions.is_empty() {
        return Err(AppError::ValidationError(
            "Role must have at least one permission".to_string(),
        ));
    }

    // Acquire write lock first to prevent race conditions
    let mut enforcer = state.enforcer.write().await;

    // Check if role already exists by checking if it has any policies
    let existing_policies =
        enforcer.get_filtered_policy(0, vec![role_name.clone(), tenant_id.to_string()]);

    if !existing_policies.is_empty() {
        return Err(AppError::Conflict(format!("Role '{}' already exists", role_name)));
    }

    // Add all permissions for the role
    let mut added_count = 0;

    for (index, permission) in payload.permissions.iter().enumerate() {
        let policy = vec![
            role_name.clone(),
            tenant_id.to_string(),
            permission.resource.clone(),
            permission.action.clone(),
        ];

        let added = enforcer
            .add_policy(policy)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to add policy: {}", e)))?;

        if !added {
            // If first permission fails to add, role already exists (race condition)
            if index == 0 {
                return Err(AppError::Conflict(format!("Role '{}' already exists", role_name)));
            }
            // Skip duplicate permissions for subsequent additions
        } else {
            added_count += 1;
        }
    }

    // Save policies to database
    enforcer
        .save_policy()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to save policies: {}", e)))?;

    drop(enforcer);

    // Bump tenant authz version to invalidate existing tokens
    bump_tenant_authz_version(&state, tenant_id, "create_role").await;

    Ok((
        StatusCode::CREATED,
        Json(CreateRoleResp {
            role_name: role_name.clone(),
            description: payload.description,
            permissions_count: added_count,
            message: format!(
                "Role '{}' created successfully with {} permissions",
                role_name, added_count
            ),
        }),
    ))
}

/// List all roles in the tenant
#[utoipa::path(
    get,
    path = "/api/v1/admin/roles",
    tag = "admin-roles",
    operation_id = "admin_list_roles",
    responses(
        (status = 200, description = "List of roles", body = RoleListResp),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden - Admin only", body = String),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_roles<S: AuthService>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
) -> Result<Json<RoleListResp>, AppError> {
    let tenant_id = admin_user.tenant_id;

    let enforcer = state.enforcer.read().await;

    // Get all policies for this tenant
    let all_policies = enforcer.get_policy();

    // Get all grouping policies (user-role assignments) for this tenant
    let all_groupings = enforcer.get_grouping_policy();

    drop(enforcer);

    // Filter policies by tenant and group by role
    let mut role_permissions_map: HashMap<String, Vec<PermissionInfo>> = HashMap::new();

    for policy in all_policies {
        // Policy format: [role, tenant_id, resource, action]
        if policy.len() >= 4 && policy[1] == tenant_id.to_string() {
            let role = policy[0].clone();
            let permission = PermissionInfo {
                resource: policy[2].clone(),
                action: policy[3].clone(),
            };

            role_permissions_map
                .entry(role)
                .or_default()
                .push(permission);
        }
    }

    // Count users for each role
    let mut role_user_count: HashMap<String, usize> = HashMap::new();

    for grouping in all_groupings {
        // Grouping format: [user_id, role, tenant_id]
        if grouping.len() >= 3 && grouping[2] == tenant_id.to_string() {
            let role = grouping[1].clone();
            *role_user_count.entry(role).or_insert(0) += 1;
        }
    }

    // Build role list
    let mut roles: Vec<RoleInfo> = role_permissions_map
        .into_iter()
        .map(|(role_name, permissions)| {
            let user_count = role_user_count.get(&role_name).copied().unwrap_or(0);
            RoleInfo {
                role_name: role_name.clone(),
                description: None, // TODO: Store descriptions in a separate table if needed
                permissions,
                user_count,
            }
        })
        .collect();

    roles.sort_by(|a, b| a.role_name.cmp(&b.role_name));

    let total = roles.len();

    Ok(Json(RoleListResp { roles, total }))
}

/// Update role permissions
#[utoipa::path(
    put,
    path = "/api/v1/admin/roles/{role_name}",
    tag = "admin-roles",
    operation_id = "admin_update_role",
    params(
        ("role_name" = String, Path, description = "Role name to update"),
    ),
    request_body = UpdateRoleReq,
    responses(
        (status = 200, description = "Role updated successfully", body = UpdateRoleResp),
        (status = 400, description = "Invalid request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden - Admin only", body = String),
        (status = 404, description = "Role not found", body = String),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_role<S: AuthService>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
    Path(role_name): Path<String>,
    Json(payload): Json<UpdateRoleReq>,
) -> Result<Json<UpdateRoleResp>, AppError> {
    // Validate request
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let tenant_id = admin_user.tenant_id;
    let role_name = role_name.trim().to_lowercase();

    // Prevent modifying system roles
    if role_name == "admin" || role_name == "user" {
        return Err(AppError::Forbidden("Cannot modify system roles (admin, user)".to_string()));
    }

    // Require at least one permission
    if payload.permissions.is_empty() {
        return Err(AppError::ValidationError(
            "Role must have at least one permission".to_string(),
        ));
    }

    let mut enforcer = state.enforcer.write().await;

    // Check if role exists
    let existing_policies =
        enforcer.get_filtered_policy(0, vec![role_name.clone(), tenant_id.to_string()]);

    if existing_policies.is_empty() {
        return Err(AppError::NotFound(format!("Role '{}' not found", role_name)));
    }

    // Backup existing policies for rollback
    let backup_policies = existing_policies.clone();

    // Remove all existing policies for this role in this tenant
    for policy in &existing_policies {
        enforcer
            .remove_policy(policy.clone())
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to remove policy: {}", e)))?;
    }

    // Add new permissions
    let mut added_count = 0;

    for permission in &payload.permissions {
        let policy = vec![
            role_name.clone(),
            tenant_id.to_string(),
            permission.resource.clone(),
            permission.action.clone(),
        ];

        match enforcer.add_policy(policy).await {
            Ok(added) => {
                if added {
                    added_count += 1;
                }
            },
            Err(e) => {
                // Rollback: Restore original policies
                for old_policy in &backup_policies {
                    let _ = enforcer.add_policy(old_policy.clone()).await;
                }
                let _ = enforcer.save_policy().await;
                return Err(AppError::InternalError(format!(
                    "Failed to add policy, rolled back: {}",
                    e
                )));
            },
        }
    }

    // Save changes
    if let Err(e) = enforcer.save_policy().await {
        // Rollback: Restore original policies if save fails
        let current =
            enforcer.get_filtered_policy(0, vec![role_name.clone(), tenant_id.to_string()]);
        for policy in current {
            let _ = enforcer.remove_policy(policy).await;
        }
        for old_policy in &backup_policies {
            let _ = enforcer.add_policy(old_policy.clone()).await;
        }
        let _ = enforcer.save_policy().await;
        return Err(AppError::InternalError(format!(
            "Failed to save policies, rolled back: {}",
            e
        )));
    }

    drop(enforcer);

    // Bump tenant authz version to invalidate existing tokens
    bump_tenant_authz_version(&state, tenant_id, "update_role").await;

    Ok(Json(UpdateRoleResp {
        role_name: role_name.clone(),
        permissions_count: added_count,
        message: format!("Role '{}' updated with {} permissions", role_name, added_count),
    }))
}

/// Delete a custom role
#[utoipa::path(
    delete,
    path = "/api/v1/admin/roles/{role_name}",
    tag = "admin-roles",
    operation_id = "admin_delete_role",
    params(
        ("role_name" = String, Path, description = "Role name to delete"),
    ),
    responses(
        (status = 200, description = "Role deleted successfully", body = DeleteRoleResp),
        (status = 400, description = "Cannot delete system role", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden - Admin only", body = String),
        (status = 404, description = "Role not found", body = String),
        (status = 409, description = "Role is assigned to users", body = String),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_role<S: AuthService>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
    Path(role_name): Path<String>,
) -> Result<Json<DeleteRoleResp>, AppError> {
    let tenant_id = admin_user.tenant_id;
    let role_name = role_name.trim().to_lowercase();

    // Prevent deleting system roles
    if role_name == "admin" || role_name == "user" {
        return Err(AppError::Forbidden("Cannot delete system roles (admin, user)".to_string()));
    }

    let mut enforcer = state.enforcer.write().await;

    // Check if role exists
    let existing_policies =
        enforcer.get_filtered_policy(0, vec![role_name.clone(), tenant_id.to_string()]);

    if existing_policies.is_empty() {
        return Err(AppError::NotFound(format!("Role '{}' not found", role_name)));
    }

    // Check if role is assigned to any users
    let user_assignments =
        enforcer.get_filtered_grouping_policy(1, vec![role_name.clone(), tenant_id.to_string()]);

    if !user_assignments.is_empty() {
        return Err(AppError::Conflict(format!(
            "Cannot delete role '{}': it is assigned to {} user(s)",
            role_name,
            user_assignments.len()
        )));
    }

    // Remove all policies for this role
    for policy in existing_policies {
        enforcer
            .remove_policy(policy)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to remove policy: {}", e)))?;
    }

    // Save changes
    enforcer
        .save_policy()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to save policies: {}", e)))?;

    drop(enforcer);

    // Bump tenant authz version to invalidate existing tokens
    bump_tenant_authz_version(&state, tenant_id, "delete_role").await;

    Ok(Json(DeleteRoleResp {
        role_name: role_name.clone(),
        message: format!("Role '{}' deleted successfully", role_name),
    }))
}

// ============================================================================
// User Role Assignment Handlers
// ============================================================================

/// Assign a role to a user
#[utoipa::path(
    post,
    path = "/api/v1/admin/users/{user_id}/roles",
    tag = "admin-users",
    operation_id = "admin_assign_user_role",
    params(
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    request_body = AssignUserRoleReq,
    responses(
        (status = 200, description = "Role assigned successfully", body = AssignUserRoleResp),
        (status = 400, description = "Invalid request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden - Admin only", body = String),
        (status = 404, description = "User or role not found", body = String),
        (status = 409, description = "Role already assigned", body = String),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn assign_role_to_user<S: AuthService>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<AssignUserRoleReq>,
) -> Result<Json<AssignUserRoleResp>, AppError> {
    // Validate request
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let tenant_id = admin_user.tenant_id;
    let role_name = payload.role_name.trim().to_lowercase();

    // Verify user exists and belongs to admin's tenant
    state.auth_service.get_user(user_id, tenant_id).await?;

    let mut enforcer = state.enforcer.write().await;

    // Verify role exists in this tenant
    let role_policies =
        enforcer.get_filtered_policy(0, vec![role_name.clone(), tenant_id.to_string()]);

    if role_policies.is_empty() {
        return Err(AppError::NotFound(format!("Role '{}' not found", role_name)));
    }

    // Add grouping policy (user -> role -> tenant)
    let grouping = vec![
        user_id.to_string(),
        role_name.clone(),
        tenant_id.to_string(),
    ];

    let added = enforcer
        .add_grouping_policy(grouping)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to add grouping policy: {}", e)))?;

    if !added {
        return Err(AppError::Conflict(format!("User already has role '{}'", role_name)));
    }

    // Save changes
    enforcer
        .save_policy()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to save policy: {}", e)))?;

    drop(enforcer);

    // Bump user authz version to invalidate existing tokens for this user
    bump_user_authz_version(&state, user_id, tenant_id, "assign_role_to_user").await;

    Ok(Json(AssignUserRoleResp {
        user_id,
        role_name: role_name.clone(),
        message: format!("Role '{}' assigned to user successfully", role_name),
    }))
}

/// Remove a role from a user
#[utoipa::path(
    delete,
    path = "/api/v1/admin/users/{user_id}/roles/{role_name}",
    tag = "admin-users",
    operation_id = "admin_remove_user_role",
    params(
        ("user_id" = Uuid, Path, description = "User ID"),
        ("role_name" = String, Path, description = "Role name to remove"),
    ),
    responses(
        (status = 200, description = "Role removed successfully", body = RemoveUserRoleResp),
        (status = 400, description = "Cannot remove user's only role", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden - Admin only", body = String),
        (status = 404, description = "User or role assignment not found", body = String),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn remove_role_from_user<S: AuthService>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
    Path((user_id, role_name)): Path<(Uuid, String)>,
) -> Result<Json<RemoveUserRoleResp>, AppError> {
    let tenant_id = admin_user.tenant_id;
    let role_name = role_name.trim().to_lowercase();

    // Verify user exists and belongs to admin's tenant
    state.auth_service.get_user(user_id, tenant_id).await?;

    let mut enforcer = state.enforcer.write().await;

    // Get all roles for this user in this tenant
    let user_roles = enforcer.get_filtered_grouping_policy(0, vec![user_id.to_string()]);
    let tenant_roles: Vec<_> = user_roles
        .into_iter()
        .filter(|g| g.len() >= 3 && g[2] == tenant_id.to_string())
        .collect();

    // Check if user has the role we're trying to remove (before checking count)
    if !tenant_roles
        .iter()
        .any(|g| g.get(1).map(|r| r == &role_name).unwrap_or(false))
    {
        return Err(AppError::NotFound(format!("User does not have role '{}'", role_name)));
    }

    // Prevent removing user's last role
    if tenant_roles.len() <= 1 {
        return Err(AppError::ValidationError(
            "Cannot remove user's only role. Users must have at least one role.".to_string(),
        ));
    }

    // Remove grouping policy
    let grouping = vec![
        user_id.to_string(),
        role_name.clone(),
        tenant_id.to_string(),
    ];

    let removed = enforcer
        .remove_grouping_policy(grouping)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to remove grouping policy: {}", e)))?;

    // Save changes (removed check should always be true since we verified above)
    if removed {
        enforcer
            .save_policy()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to save policy: {}", e)))?;
    }

    drop(enforcer);

    // Bump user authz version to invalidate existing tokens for this user
    if removed {
        bump_user_authz_version(&state, user_id, tenant_id, "remove_role_from_user").await;
    }

    Ok(Json(RemoveUserRoleResp {
        user_id,
        role_name: role_name.clone(),
        message: format!("Role '{}' removed from user successfully", role_name),
    }))
}

/// Get all roles assigned to a user
#[utoipa::path(
    get,
    path = "/api/v1/admin/users/{user_id}/roles",
    tag = "admin-users",
    operation_id = "admin_get_user_roles",
    params(
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "User's roles", body = UserRolesResp),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden - Admin only", body = String),
        (status = 404, description = "User not found", body = String),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user_roles<S: AuthService>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserRolesResp>, AppError> {
    let tenant_id = admin_user.tenant_id;

    // Verify user exists and belongs to admin's tenant
    state.auth_service.get_user(user_id, tenant_id).await?;

    let enforcer = state.enforcer.read().await;

    // Get all roles for this user in this tenant
    let user_roles = enforcer.get_filtered_grouping_policy(0, vec![user_id.to_string()]);

    let roles: Vec<String> = user_roles
        .into_iter()
        .filter(|g| g.len() >= 3 && g[2] == tenant_id.to_string())
        .map(|g| g[1].clone())
        .collect();

    drop(enforcer);

    Ok(Json(UserRolesResp { user_id, roles }))
}

// ============================================================================
// Permission Listing Handler
// ============================================================================

/// List all available permissions in the system
#[utoipa::path(
    get,
    path = "/api/v1/admin/permissions",
    tag = "admin-permissions",
    operation_id = "admin_list_permissions",
    responses(
        (status = 200, description = "List of available permissions", body = PermissionListResp),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden - Admin only", body = String),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_permissions<S: AuthService>(
    RequireAdmin(_admin_user): RequireAdmin,
    Extension(_state): Extension<AppState<S>>,
) -> Result<Json<PermissionListResp>, AppError> {
    // Define available permissions for the system
    // In a production system, this could be loaded from a configuration file or database
    let permissions = vec![
        AvailablePermission {
            resource: "users".to_string(),
            actions: vec![
                "read".to_string(),
                "write".to_string(),
                "delete".to_string(),
            ],
            description: "Manage user accounts and profiles".to_string(),
        },
        AvailablePermission {
            resource: "products".to_string(),
            actions: vec![
                "read".to_string(),
                "write".to_string(),
                "delete".to_string(),
                "import".to_string(),
            ],
            description: "Manage product catalog and inventory".to_string(),
        },
        AvailablePermission {
            resource: "orders".to_string(),
            actions: vec![
                "read".to_string(),
                "write".to_string(),
                "delete".to_string(),
                "approve".to_string(),
                "fulfill".to_string(),
            ],
            description: "Manage customer orders and fulfillment".to_string(),
        },
        AvailablePermission {
            resource: "inventory".to_string(),
            actions: vec![
                "read".to_string(),
                "write".to_string(),
                "adjust".to_string(),
                "transfer".to_string(),
            ],
            description: "Manage stock levels and transfers".to_string(),
        },
        AvailablePermission {
            resource: "integrations".to_string(),
            actions: vec![
                "read".to_string(),
                "write".to_string(),
                "delete".to_string(),
                "sync".to_string(),
            ],
            description: "Manage third-party integrations".to_string(),
        },
        AvailablePermission {
            resource: "payments".to_string(),
            actions: vec![
                "read".to_string(),
                "write".to_string(),
                "refund".to_string(),
            ],
            description: "Manage payment transactions".to_string(),
        },
        AvailablePermission {
            resource: "reports".to_string(),
            actions: vec!["read".to_string(), "export".to_string()],
            description: "View and export analytics reports".to_string(),
        },
        AvailablePermission {
            resource: "settings".to_string(),
            actions: vec!["read".to_string(), "write".to_string()],
            description: "Manage tenant settings and configuration".to_string(),
        },
    ];

    let total = permissions.len();

    Ok(Json(PermissionListResp { permissions, total }))
}

// ============================================================================
// Admin User Management Handlers
// ============================================================================

/// Create a new user in the admin's tenant
///
/// Creates a user with the specified role (default: "user").
/// The user is always created in the admin's tenant (tenant isolation enforced via JWT).
/// Password is hashed with bcrypt before storage.
///
/// ## Authorization
/// - Admin-only endpoint (requires admin role via Casbin)
///
/// ## Role Assignment
/// - Default role is "user" if not specified
/// - Cannot create users with "owner" role (owner is only assigned during tenant bootstrap)
/// - Custom roles must exist in the tenant (validated at creation time)
///
/// ## Single Role Invariant
/// This endpoint enforces the Option D (Single Custom Role) pattern:
/// - Each user has exactly one role
/// - Role is stored in `users.role` field
/// - Casbin grouping policy is created for authorization
#[utoipa::path(
    post,
    path = "/api/v1/admin/users",
    tag = "admin-users",
    operation_id = "admin_create_user",
    request_body = AdminCreateUserReq,
    responses(
        (status = 201, description = "User created successfully", body = AdminCreateUserResp),
        (status = 400, description = "Invalid request (email, password, or role validation failed)", body = String),
        (status = 401, description = "Unauthorized - Missing or invalid JWT", body = String),
        (status = 403, description = "Forbidden - Admin only or cannot create owner role", body = String),
        (status = 409, description = "User already exists in tenant", body = String),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn admin_create_user<S: AuthService>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
    Json(payload): Json<AdminCreateUserReq>,
) -> Result<(StatusCode, Json<AdminCreateUserResp>), AppError> {
    // Validate request body
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let admin_tenant_id = admin_user.tenant_id;

    // Validate custom role existence in Casbin policies (per AUTHORIZATION_RBAC_STRATEGY.md)
    // System roles (admin, user) are always valid; custom roles must have policies defined
    if let Some(role) = &payload.role {
        if !matches!(role.as_str(), "admin" | "user") {
            // For custom roles, check if any policies exist for this role in the tenant
            // This uses in-memory Casbin policies (no DB query per AUTHORIZATION_RBAC_STRATEGY.md)
            // Policy format: p = sub(role), dom(tenant_id), obj(resource), act(action)
            // Index 0 = role, Index 1 = tenant_id
            let enforcer = state.enforcer.read().await;
            let role_policies =
                enforcer.get_filtered_policy(0, vec![role.clone(), admin_tenant_id.to_string()]);

            if role_policies.is_empty() {
                return Err(AppError::ValidationError(format!(
                    "Role '{}' does not exist in this tenant. Custom roles must have policies defined before users can be assigned to them.",
                    role
                )));
            }
        }
    }

    // Delegate to service layer (handles password hashing, tenant isolation, role validation)
    let response = state
        .auth_service
        .admin_create_user(admin_tenant_id, payload)
        .await?;

    // Add Casbin grouping policy for the new user
    // This ensures the user has the correct role for authorization
    let mut enforcer = state.enforcer.write().await;
    let grouping_result = enforcer
        .add_grouping_policy(vec![
            response.user_id.to_string(),
            response.role.clone(),
            response.tenant_id.to_string(),
        ])
        .await;

    // Handle Casbin policy addition failure with compensating delete
    let grouping_added = match grouping_result {
        Ok(added) => added,
        Err(e) => {
            tracing::error!(
                user_id = %response.user_id,
                role = %response.role,
                tenant_id = %response.tenant_id,
                error = %e,
                "Failed to add Casbin grouping policy for new user, attempting compensating delete"
            );

            // Drop the enforcer lock before async call to avoid blocking other tasks
            drop(enforcer);

            // Compensating action: delete the user we just created to maintain consistency
            if let Err(delete_err) = state
                .auth_service
                .internal_delete_user(response.user_id, response.tenant_id)
                .await
            {
                tracing::error!(
                    user_id = %response.user_id,
                    tenant_id = %response.tenant_id,
                    error = %delete_err,
                    "Failed to perform compensating delete after Casbin policy failure"
                );
            }

            return Err(AppError::InternalError(format!("Failed to set user role policy: {}", e)));
        },
    };

    // Persist the policy to database
    if grouping_added {
        if let Err(e) = enforcer.save_policy().await {
            tracing::error!(
                user_id = %response.user_id,
                role = %response.role,
                tenant_id = %response.tenant_id,
                error = %e,
                "Failed to save Casbin policy, attempting compensating delete"
            );

            // Try to remove the in-memory policy we just added
            let _ = enforcer
                .remove_grouping_policy(vec![
                    response.user_id.to_string(),
                    response.role.clone(),
                    response.tenant_id.to_string(),
                ])
                .await;

            // Drop the enforcer lock before async call to avoid blocking other tasks
            drop(enforcer);

            // Compensating action: delete the user
            if let Err(delete_err) = state
                .auth_service
                .internal_delete_user(response.user_id, response.tenant_id)
                .await
            {
                tracing::error!(
                    user_id = %response.user_id,
                    tenant_id = %response.tenant_id,
                    error = %delete_err,
                    "Failed to perform compensating delete after save_policy failure"
                );
            }

            return Err(AppError::InternalError(format!("Failed to save user role policy: {}", e)));
        }

        tracing::info!(
            user_id = %response.user_id,
            role = %response.role,
            tenant_id = %response.tenant_id,
            "Added and persisted Casbin grouping policy for new user"
        );
    }

    Ok((StatusCode::CREATED, Json(response)))
}

// ============================================================================
// User Lifecycle Management Handlers
// ============================================================================

/// Suspend a user (admin operation)
#[utoipa::path(
    post,
    path = "/api/v1/admin/users/{user_id}/suspend",
    tag = "admin-users",
    operation_id = "admin_suspend_user",
    params(
        ("user_id" = Uuid, Path, description = "User ID to suspend"),
    ),
    request_body = SuspendUserReq,
    responses(
        (status = 200, description = "User suspended successfully", body = SuspendUserResp),
        (status = 400, description = "Bad request - user already suspended", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden - cannot suspend tenant owner", body = String),
        (status = 404, description = "User not found", body = String),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn suspend_user<S: AuthService>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<SuspendUserReq>,
) -> Result<Json<SuspendUserResp>, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let response = state
        .auth_service
        .admin_suspend_user(admin_user.tenant_id, admin_user.user_id, user_id, payload.reason)
        .await?;

    // Bump user authz version to invalidate existing tokens for suspended user
    bump_user_authz_version(&state, user_id, admin_user.tenant_id, "suspend_user").await;

    Ok(Json(response))
}

/// Unsuspend a user (admin operation)
#[utoipa::path(
    post,
    path = "/api/v1/admin/users/{user_id}/unsuspend",
    tag = "admin-users",
    operation_id = "admin_unsuspend_user",
    params(
        ("user_id" = Uuid, Path, description = "User ID to unsuspend"),
    ),
    responses(
        (status = 200, description = "User unsuspended successfully", body = UnsuspendUserResp),
        (status = 400, description = "Bad request - user is not suspended", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden - Admin only", body = String),
        (status = 404, description = "User not found", body = String),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn unsuspend_user<S: AuthService>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UnsuspendUserResp>, AppError> {
    let response = state
        .auth_service
        .admin_unsuspend_user(admin_user.tenant_id, user_id)
        .await?;

    // Bump user authz version (recommended for clean security boundary)
    bump_user_authz_version(&state, user_id, admin_user.tenant_id, "unsuspend_user").await;

    Ok(Json(response))
}

/// Soft delete a user (admin operation)
#[utoipa::path(
    delete,
    path = "/api/v1/admin/users/{user_id}",
    tag = "admin-users",
    operation_id = "admin_delete_user",
    params(
        ("user_id" = Uuid, Path, description = "User ID to delete"),
    ),
    responses(
        (status = 200, description = "User deleted successfully", body = DeleteUserResp),
        (status = 400, description = "Bad request - user already deleted", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden - cannot delete tenant owner", body = String),
        (status = 404, description = "User not found", body = String),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_user<S: AuthService>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<DeleteUserResp>, AppError> {
    let response = state
        .auth_service
        .admin_delete_user(admin_user.tenant_id, admin_user.user_id, user_id)
        .await?;

    // Bump user authz version to invalidate existing tokens for deleted user
    bump_user_authz_version(&state, user_id, admin_user.tenant_id, "delete_user").await;

    Ok(Json(response))
}

/// Reset user password (admin operation)
#[utoipa::path(
    post,
    path = "/api/v1/admin/users/{user_id}/reset-password",
    tag = "admin-users",
    operation_id = "admin_reset_user_password",
    params(
        ("user_id" = Uuid, Path, description = "User ID whose password to reset"),
    ),
    request_body = AdminResetPasswordReq,
    responses(
        (status = 200, description = "Password reset successfully", body = AdminResetPasswordResp),
        (status = 400, description = "Invalid password", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 403, description = "Forbidden - Admin only", body = String),
        (status = 404, description = "User not found", body = String),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn reset_user_password<S: AuthService>(
    RequireAdmin(admin_user): RequireAdmin,
    Extension(state): Extension<AppState<S>>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<AdminResetPasswordReq>,
) -> Result<Json<AdminResetPasswordResp>, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let response = state
        .auth_service
        .admin_reset_password(
            admin_user.tenant_id,
            user_id,
            payload.new_password,
            payload.force_logout,
        )
        .await?;

    // Bump user authz version to invalidate existing tokens after password reset
    bump_user_authz_version(&state, user_id, admin_user.tenant_id, "reset_user_password").await;

    Ok(Json(response))
}
