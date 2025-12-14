use user_service_core::domains::auth::dto::admin_dto::*;
use user_service_core::domains::auth::dto::auth_dto::*;
use utoipa::OpenApi;

/// OpenAPI documentation for User Service
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::health_check,
        crate::handlers::register,
        crate::handlers::login,
        crate::handlers::refresh_token,
        crate::handlers::logout,
        crate::handlers::list_users,
        crate::handlers::get_user,
        // Low-level policy management
        crate::handlers::add_policy,
        crate::handlers::remove_policy,
        // TODO: OAuth2 endpoints - add after feature "openapi" enables in core
        // crate::oauth_handlers::oauth_authorize,
        // crate::oauth_handlers::oauth_callback,
        // crate::oauth_handlers::oauth_refresh,
        // Admin role management endpoints (new comprehensive implementations)
        crate::admin_handlers::create_role,
        crate::admin_handlers::list_roles,
        crate::admin_handlers::update_role,
        crate::admin_handlers::delete_role,
        crate::admin_handlers::assign_role_to_user,
        crate::admin_handlers::remove_role_from_user,
        crate::admin_handlers::get_user_roles,
        crate::admin_handlers::list_permissions,
    ),
    components(
        schemas(
            HealthResp,
            RegisterReq,
            LoginReq,
            RefreshReq,
            AuthResp,
            UserInfo,
            UserListResp,
            ErrorResp,
            crate::handlers::CreatePolicyReq,
            crate::handlers::DeletePolicyReq,
            // TODO: OAuth2 DTOs - add after feature "openapi" enables in core
            // OAuth2AuthorizeReq,
            // OAuth2AuthorizeResp,
            // OAuth2CallbackReq,
            // OAuth2CallbackResp,
            // OAuth2RefreshReq,
            // OAuth2RefreshResp,
            // KanidmUserInfo,
            // TenantInfo,
            // Admin DTOs (comprehensive role management)
            CreateRoleReq,
            CreateRoleResp,
            RoleListResp,
            RoleInfo,
            PermissionInfo,
            PermissionReq,
            UpdateRoleReq,
            UpdateRoleResp,
            DeleteRoleResp,
            AssignUserRoleReq,
            AssignUserRoleResp,
            RemoveUserRoleResp,
            UserRolesResp,
            PermissionListResp,
            AvailablePermission,
            AddPolicyReq,
            RemovePolicyReq,
            PolicyResp,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "oauth", description = "OAuth2 authentication with Kanidm"),
        (name = "users", description = "User management endpoints"),
        (name = "admin", description = "Admin-only endpoints for role and policy management"),
        (name = "admin-roles", description = "Role management endpoints (admin only)"),
        (name = "admin-users", description = "User role assignment endpoints (admin only)"),
        (name = "admin-permissions", description = "Permission management endpoints (admin only)"),
    ),
    info(
        title = "User Service API",
        version = "0.1.0",
        description = "Multi-tenant user authentication and management service",
        contact(
            name = "Anthill Team",
            email = "tymon3568@gmail.com"
        ),
    ),
    servers(
        (url = "http://localhost:8000", description = "Local development server"),
    ),
)]
pub struct ApiDoc;

/// Export OpenAPI spec to YAML file (only with --features export-spec)
#[cfg(feature = "export-spec")]
#[allow(dead_code)]
pub fn export_spec() -> std::io::Result<()> {
    use std::path::Path;

    let openapi = ApiDoc::openapi();
    let yaml = serde_yaml::to_string(&openapi).expect("Failed to serialize OpenAPI to YAML");

    let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../shared/openapi/user.yaml"));

    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, yaml)?;

    println!("cargo:warning=OpenAPI spec exported to {:?}", path);
    Ok(())
}
