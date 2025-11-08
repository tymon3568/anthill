use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use shared_auth::casbin::{CoreApi, MgmtApi};
use shared_auth::extractors::AuthUser;
use shared_error::AppError;
use user_service_core::domains::auth::domain::service::AuthService;

use crate::handlers::AppState;

/// Query parameters for permission checking
#[derive(Debug, Deserialize)]
pub struct PermissionCheckQuery {
    pub resource: String,
    pub action: String,
}

/// Check if the current user has permission for a specific resource and action
#[utoipa::path(
    get,
    path = "/api/v1/users/permissions/check",
    tag = "permissions",
    operation_id = "check_user_permission",
    params(
        ("resource" = String, Query, description = "Resource to check permission for"),
        ("action" = String, Query, description = "Action to check permission for"),
    ),
    responses(
        (status = 200, description = "Permission check result", body = PermissionCheckResp),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn check_permission<S: AuthService>(
    auth_user: AuthUser,
    State(state): State<AppState<S>>,
    Query(query): Query<PermissionCheckQuery>,
) -> Result<Json<PermissionCheckResp>, AppError> {
    let enforcer = state.enforcer.read().await;

    // Check permission using Casbin enforcer
    // Format: subject, tenant, resource, action
    let allowed = enforcer
        .enforce((
            &auth_user.user_id.to_string(),
            &auth_user.tenant_id.to_string(),
            &query.resource,
            &query.action,
        ))
        .map_err(|e| AppError::InternalError(format!("Permission check failed: {}", e)))?;

    drop(enforcer);

    Ok(Json(PermissionCheckResp { allowed }))
}

/// Get all permissions for the current user
#[utoipa::path(
    get,
    path = "/api/v1/users/permissions",
    tag = "permissions",
    operation_id = "get_user_permissions",
    responses(
        (status = 200, description = "User permissions", body = UserPermissionsResp),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user_permissions<S: AuthService>(
    auth_user: AuthUser,
    State(state): State<AppState<S>>,
) -> Result<Json<UserPermissionsResp>, AppError> {
    let enforcer = state.enforcer.read().await;

    // Get all policies that apply to this user
    let _all_policies = enforcer.get_policy();

    // Get user's roles in this tenant
    let user_groupings =
        enforcer.get_filtered_grouping_policy(0, vec![auth_user.user_id.to_string()]);
    let user_roles: Vec<String> = user_groupings
        .into_iter()
        .filter(|g| g.len() >= 3 && g[2] == auth_user.tenant_id.to_string())
        .map(|g| g[1].clone())
        .collect();

    // Collect all permissions from user's roles in this tenant
    let mut permissions: Vec<String> = Vec::new();

    for role in &user_roles {
        // Get policies for this role in this tenant
        let role_policies =
            enforcer.get_filtered_policy(0, vec![role.clone(), auth_user.tenant_id.to_string()]);

        for policy in role_policies {
            if policy.len() >= 4 {
                // Format: resource:action
                let permission = format!("{}:{}", policy[2], policy[3]);
                if !permissions.contains(&permission) {
                    permissions.push(permission);
                }
            }
        }
    }

    drop(enforcer);

    Ok(Json(UserPermissionsResp { permissions }))
}

/// Get all roles and groups for the current user
#[utoipa::path(
    get,
    path = "/api/v1/users/roles",
    tag = "permissions",
    operation_id = "get_user_roles",
    responses(
        (status = 200, description = "User roles and groups", body = UserRolesResp),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user_roles<S: AuthService>(
    auth_user: AuthUser,
    State(state): State<AppState<S>>,
) -> Result<Json<UserRolesResp>, AppError> {
    let enforcer = state.enforcer.read().await;

    // Get user's roles in this tenant
    let user_groupings =
        enforcer.get_filtered_grouping_policy(0, vec![auth_user.user_id.to_string()]);
    let roles: Vec<String> = user_groupings
        .into_iter()
        .filter(|g| g.len() >= 3 && g[2] == auth_user.tenant_id.to_string())
        .map(|g| g[1].clone())
        .collect();

    // For now, groups are the same as roles (Kanidm groups map to roles)
    // In the future, this could include additional group memberships
    let groups = roles.clone();

    drop(enforcer);

    Ok(Json(UserRolesResp { roles, groups }))
}

/// Validate tenant access for the current user
#[utoipa::path(
    get,
    path = "/api/v1/users/tenant/validate",
    tag = "permissions",
    operation_id = "validate_tenant_access",
    responses(
        (status = 200, description = "Tenant access validation", body = TenantAccessResp),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - No access to tenant"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn validate_tenant_access<S: AuthService>(
    auth_user: AuthUser,
    State(_state): State<AppState<S>>,
) -> Result<Json<TenantAccessResp>, AppError> {
    // Since the AuthUser extractor already validates tenant access,
    // if we reach this point, the user has valid tenant access
    Ok(Json(TenantAccessResp {
        tenant_id: auth_user.tenant_id,
        has_access: true,
        message: "User has valid access to tenant".to_string(),
    }))
}

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PermissionCheckResp {
    pub allowed: bool,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UserPermissionsResp {
    pub permissions: Vec<String>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UserRolesResp {
    pub roles: Vec<String>,
    pub groups: Vec<String>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct TenantAccessResp {
    pub tenant_id: uuid::Uuid,
    pub has_access: bool,
    pub message: String,
}
