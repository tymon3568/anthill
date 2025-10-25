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
        crate::handlers::add_policy,
        crate::handlers::remove_policy,
        crate::handlers::assign_role_to_user,
        crate::handlers::revoke_role_from_user,
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
            crate::handlers::AssignRoleReq,
            crate::handlers::RevokeRoleReq,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "User management endpoints"),
        (name = "admin", description = "Admin-only endpoints for role and policy management"),
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
        (url = "http://localhost:3000", description = "Local development server"),
    ),
)]
pub struct ApiDoc;

/// Export OpenAPI spec to YAML file (only with --features export-spec)
#[cfg(feature = "export-spec")]
pub fn export_spec() -> std::io::Result<()> {
    use std::path::Path;

    let openapi = ApiDoc::openapi();
    let yaml = serde_yaml::to_string(&openapi).expect("Failed to serialize OpenAPI to YAML");

    let path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../shared/openapi/user.yaml"
    ));

    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, yaml)?;

    println!("cargo:warning=OpenAPI spec exported to {:?}", path);
    Ok(())
}
