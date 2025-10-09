use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "User Service API",
        version = "0.1.0",
        description = "Authentication, user management, and tenant management API",
        contact(
            name = "API Support",
            email = "support@example.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development"),
        (url = "https://api.example.com", description = "Production")
    ),
    paths(
        crate::handlers::health_check,
        crate::handlers::register,
        crate::handlers::login,
        crate::handlers::refresh_token,
        crate::handlers::list_users,
    ),
    components(schemas(
        crate::models::ErrorResp,
        crate::models::HealthResp,
        crate::models::RegisterReq,
        crate::models::LoginReq,
        crate::models::AuthResp,
        crate::models::UserInfo,
        crate::models::RefreshReq,
        crate::models::UserListResp,
    )),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "User management endpoints"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

use utoipa::Modify;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build()
                ),
            )
        }
    }
}
