// Library exports for integration tests
pub mod admin_handlers;
pub mod extractors;
pub mod handlers;
pub mod invitation_handlers;
pub mod openapi;
pub mod permission_handlers;
pub mod profile_handlers;
pub mod verification_handlers;

// Re-export commonly used types for tests
pub use handlers::AppState;
pub use profile_handlers::ProfileAppState;

use axum::routing::{delete, get, post};
use axum::{Extension, Router};
use shared_auth::{create_enforcer, AuthzState};
use shared_config::Config;
use shared_db::PgPool;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use user_service_infra::auth::{
    AuthServiceImpl, InvitationServiceImpl, PgInvitationRepository, PgSessionRepository,
    PgTenantRepository, PgUserRepository,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

type ConcreteAuthService =
    AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>;

type AppRouter = Router;

/// Create test app with database pool and config (for integration tests)
pub async fn get_app(db_pool: PgPool, config: &Config) -> AppRouter {
    // Initialize Casbin enforcer
    let enforcer = create_enforcer(&config.database_url, Some(&config.casbin_model_path))
        .await
        .expect("Failed to initialize Casbin enforcer");

    // Initialize repositories
    let user_repo = PgUserRepository::new(db_pool.clone());
    let tenant_repo = PgTenantRepository::new(db_pool.clone());
    let session_repo = PgSessionRepository::new(db_pool.clone());
    let invitation_repo = PgInvitationRepository::new(db_pool.clone());

    // Initialize auth service
    let auth_service = AuthServiceImpl::new(
        user_repo.clone(),
        tenant_repo.clone(),
        session_repo,
        config.jwt_secret.clone(),
        config.jwt_expiration,
        config.jwt_refresh_expiration,
    );

    // Initialize invitation service
    let invitation_service = InvitationServiceImpl::new(
        invitation_repo,
        user_repo.clone(),
        enforcer.clone(),
        config.invitation_expiry_hours,
        config.invitation_max_attempts,
    );

    // Create app state
    let state = AppState {
        auth_service: Arc::new(auth_service),
        enforcer,
        jwt_secret: config.jwt_secret.clone(),
        user_repo: Some(Arc::new(user_repo)),
        tenant_repo: Some(Arc::new(tenant_repo)),
        invitation_service: Some(Arc::new(invitation_service)),
        config: config.clone(),
    };

    create_router(&state)
}

/// Create router from app state (for testing)
pub fn create_router(
    state: &AppState<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>,
) -> AppRouter {
    let authz_state = AuthzState {
        enforcer: state.enforcer.clone(),
        jwt_secret: state.jwt_secret.clone(),
    };

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/api/v1/auth/register", post(handlers::register::<ConcreteAuthService>))
        .route("/api/v1/auth/login", post(handlers::login::<ConcreteAuthService>))
        .route("/api/v1/auth/refresh", post(handlers::refresh_token::<ConcreteAuthService>))
        .route("/api/v1/auth/logout", post(handlers::logout::<ConcreteAuthService>))
        .route(
            "/api/v1/auth/accept-invite",
            post(invitation_handlers::accept_invitation::<ConcreteAuthService>),
        )
        .layer(Extension(state.clone()));

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        .route("/api/v1/users", get(handlers::list_users::<ConcreteAuthService>))
        .route("/api/v1/users/{user_id}", get(handlers::get_user::<ConcreteAuthService>))
        // Invitation management (admin only)
        .route("/api/v1/admin/users/invite", post(invitation_handlers::create_invitation::<ConcreteAuthService>))
        .route("/api/v1/admin/users/invitations", get(invitation_handlers::list_invitations::<ConcreteAuthService>))
        .route("/api/v1/admin/users/invitations/{invitation_id}", delete(invitation_handlers::revoke_invitation::<ConcreteAuthService>))
        .route("/api/v1/admin/users/invitations/{invitation_id}/resend", post(invitation_handlers::resend_invitation::<ConcreteAuthService>))
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
