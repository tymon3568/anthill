// Library exports for integration tests
pub mod admin_handlers;
pub mod extractors;
pub mod handlers;
pub mod oauth_handlers;
pub mod openapi;
pub mod permission_handlers;
pub mod profile_handlers;

// Re-export commonly used types for tests
pub use handlers::AppState;
pub use profile_handlers::ProfileAppState;

use axum::routing::{get, post};
use axum::{Extension, Router};
use shared_auth::AuthzState;
use tower_http::trace::TraceLayer;
use user_service_infra::auth::{
    AuthServiceImpl, PgSessionRepository, PgTenantRepository, PgUserRepository,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

type ConcreteAuthService =
    AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>;

type AppRouter = Router;

/// Create router from app state (for testing)
pub fn create_router(
    state: &AppState<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>,
) -> AppRouter {
    let authz_state = AuthzState {
        enforcer: state.enforcer.clone(),
        jwt_secret: state.jwt_secret.clone(),
        kanidm_client: state.kanidm_client.clone(),
    };

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/api/v1/auth/register", post(handlers::register::<ConcreteAuthService>))
        .route("/api/v1/auth/login", post(handlers::login::<ConcreteAuthService>))
        .route("/api/v1/auth/refresh", post(handlers::refresh_token::<ConcreteAuthService>))
        .route("/api/v1/auth/logout", post(handlers::logout::<ConcreteAuthService>))
        // OAuth2 endpoints
        .route("/api/v1/auth/oauth/authorize", post(oauth_handlers::oauth_authorize::<ConcreteAuthService>))
        .route("/api/v1/auth/oauth/callback", post(oauth_handlers::oauth_callback::<ConcreteAuthService>))
        .route("/api/v1/auth/oauth/refresh", post(oauth_handlers::oauth_refresh::<ConcreteAuthService>))
        .layer(Extension(state.clone()));

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        .route("/api/v1/users", get(handlers::list_users::<ConcreteAuthService>))
        .route("/api/v1/users/{user_id}", get(handlers::get_user::<ConcreteAuthService>))
        // Low-level policy management (for advanced use cases)
        .route(
            "/api/v1/admin/policies",
            post(handlers::add_policy::<ConcreteAuthService>).delete(handlers::remove_policy::<ConcreteAuthService>),
        )
        .layer(Extension(state.clone()))
        .layer(shared_auth::CasbinAuthLayer::new(authz_state));

    // Combine all API routes
    let api_routes = public_routes.merge(protected_routes);

    // Build application with routes and Swagger UI
    Router::new()
        .route("/health", get(handlers::health_check))
        .merge(api_routes)
        .merge(
            Router::from(
                SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()),
            )
            .layer(Extension(state.clone())),
        )
        .layer(Extension(state.clone()))
        .layer(TraceLayer::new_for_http())
}
