use user_service_core::domains::auth::dto::admin_dto::*;
use user_service_core::domains::auth::dto::auth_dto::*;
use user_service_core::domains::auth::dto::email_verification_dto::*;
use user_service_core::domains::auth::dto::invitation_dto::*;
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
        // Admin user management
        crate::admin_handlers::admin_create_user,
        // Admin user lifecycle management
        crate::admin_handlers::suspend_user,
        crate::admin_handlers::unsuspend_user,
        crate::admin_handlers::delete_user,
        crate::admin_handlers::reset_user_password,
        // Invitation management
        crate::invitation_handlers::create_invitation,
        crate::invitation_handlers::accept_invitation,
        crate::invitation_handlers::list_invitations,
        crate::invitation_handlers::revoke_invitation,
        crate::invitation_handlers::resend_invitation,
        // Email verification
        crate::verification_handlers::verify_email,
        crate::verification_handlers::resend_verification,
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
            // Admin user management DTOs
            AdminCreateUserReq,
            AdminCreateUserResp,
            // Admin user lifecycle management DTOs
            SuspendUserReq,
            SuspendUserResp,
            UnsuspendUserResp,
            DeleteUserResp,
            AdminResetPasswordReq,
            AdminResetPasswordResp,
            // Invitation DTOs
            CreateInvitationRequest,
            CreateInvitationResponse,
            AcceptInvitationRequest,
            AcceptInvitationResponse,
            InvitationListItem,
            InvitedByInfo,
            ListInvitationsQuery,
            ListInvitationsResponse,
            // Email verification DTOs
            VerifyEmailReq,
            VerifyEmailResp,
            ResendVerificationReq,
            ResendVerificationResp,
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
        (name = "invitations", description = "User invitation management endpoints"),
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
