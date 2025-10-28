// Library exports for integration tests
pub mod extractors;
pub mod handlers;
pub mod openapi;
pub mod profile_handlers;

// Re-export commonly used types for tests
pub use handlers::AppState;
pub use profile_handlers::ProfileAppState;

use axum::error_handling::HandleErrorLayer;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use shared_auth::enforcer::{create_enforcer, SharedEnforcer};
use shared_auth::middleware::AuthError;
use shared_config::Config;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use user_service_infra::auth::{
    AuthServiceImpl, PgSessionRepository, PgTenantRepository, PgUserRepository,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AuthzState {
    pub enforcer: SharedEnforcer,
    pub jwt_secret: String,
}

pub async fn get_app(db_pool: PgPool, config: &Config) -> Router {
    // Initialize Casbin enforcer
    let enforcer = create_enforcer(&config.database_url, None)
        .await
        .expect("Failed to initialize Casbin enforcer");

    // Initialize repositories
    let user_repo = PgUserRepository::new(db_pool.clone());
    let tenant_repo = PgTenantRepository::new(db_pool.clone());
    let session_repo = PgSessionRepository::new(db_pool.clone());

    // Initialize service
    let auth_service = AuthServiceImpl::new(
        user_repo,
        tenant_repo,
        session_repo,
        config.jwt_secret.clone(),
        config.jwt_expiration,
        config.jwt_refresh_expiration,
    );

    // Create application state
    let state = AppState {
        auth_service: Arc::new(auth_service),
        enforcer: enforcer.clone(),
        jwt_secret: config.jwt_secret.clone(),
    };

    let authz_state = AuthzState {
        enforcer: enforcer.clone(),
        jwt_secret: config.jwt_secret.clone(),
    };

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/api/v1/auth/register", post(handlers::register))
        .route("/api/v1/auth/login", post(handlers::login))
        .route("/api/v1/auth/refresh", post(handlers::refresh_token))
        .route("/api/v1/auth/logout", post(handlers::logout));

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        .route("/api/v1/users", get(handlers::list_users))
        .route("/api/v1/users/:user_id", get(handlers::get_user))
        // Admin routes for role management
        .route(
            "/api/v1/admin/policies",
            post(handlers::add_policy).delete(handlers::remove_policy),
        )
        .route(
            "/api/v1/admin/users/:user_id/roles",
            post(handlers::assign_role_to_user).delete(handlers::revoke_role_from_user),
        )
        .layer(axum::middleware::from_fn_with_state(
            authz_state,
            shared_auth::middleware::casbin_middleware,
        ))
        .layer(HandleErrorLayer::new(|e: AuthError| async move {
            e.into_response()
        })); // Combine all API routes
    let api_routes = public_routes.merge(protected_routes).with_state(state);

    // Build application with routes and Swagger UI
    Router::new()
        .route("/health", get(handlers::health_check))
        .merge(api_routes)
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
}
