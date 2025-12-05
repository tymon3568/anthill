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
use shared_auth::enforcer::create_enforcer;
use shared_auth::AuthzState;
use shared_config::Config;
use shared_kanidm_client::{KanidmClient, KanidmConfig};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use user_service_infra::auth::{
    AuthServiceImpl, PgSessionRepository, PgTenantRepository, PgUserRepository,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

type ConcreteAuthService =
    AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>;

type AppRouter = Router;

/// Create Kanidm client from configuration
fn create_kanidm_client(config: &Config) -> KanidmClient {
    let kanidm_config = KanidmConfig {
        kanidm_url: config
            .kanidm_url
            .clone()
            .unwrap_or_else(|| "http://localhost:8300".to_string()),
        client_id: config
            .kanidm_client_id
            .clone()
            .unwrap_or_else(|| "dev".to_string()),
        client_secret: config
            .kanidm_client_secret
            .clone()
            .unwrap_or_else(|| "dev".to_string()),
        redirect_uri: config
            .kanidm_redirect_url
            .clone()
            .unwrap_or_else(|| "http://localhost:8000/oauth/callback".to_string()),
        scopes: vec!["openid".to_string()],
        skip_jwt_verification: true, // DEV/TEST MODE ONLY - should be false in production
        allowed_issuers: vec![config
            .kanidm_url
            .clone()
            .unwrap_or_else(|| "http://localhost:8300".to_string())],
        expected_audience: config.kanidm_client_id.clone(),
    };

    KanidmClient::new(kanidm_config).expect("Failed to create Kanidm client")
}

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
        .layer(Extension(state.clone()))
        .route("/api/v1/auth/register", post(handlers::register::<ConcreteAuthService>))
        .route("/api/v1/auth/login", post(handlers::login::<ConcreteAuthService>))
        .route("/api/v1/auth/refresh", post(handlers::refresh_token::<ConcreteAuthService>))
        .route("/api/v1/auth/logout", post(handlers::logout::<ConcreteAuthService>))
        // OAuth2 endpoints
        .route("/api/v1/auth/oauth/authorize", post(oauth_handlers::oauth_authorize::<ConcreteAuthService>))
        .route("/api/v1/auth/oauth/callback", post(oauth_handlers::oauth_callback::<ConcreteAuthService>))
        .route("/api/v1/auth/oauth/refresh", post(oauth_handlers::oauth_refresh::<ConcreteAuthService>));

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        .layer(Extension(state.clone()))
        .route("/api/v1/users", get(handlers::list_users::<ConcreteAuthService>))
        .route("/api/v1/users/{user_id}", get(handlers::get_user::<ConcreteAuthService>))
        // Low-level policy management (for advanced use cases)
        .route(
            "/api/v1/admin/policies",
            post(handlers::add_policy::<ConcreteAuthService>).delete(handlers::remove_policy::<ConcreteAuthService>),
        )
        .layer(shared_auth::CasbinAuthLayer::new(authz_state));

    // Combine all API routes
    let api_routes = public_routes.merge(protected_routes);

    // Build application with routes and Swagger UI
    Router::new()
        .layer(Extension(state.clone()))
        .route("/health", get(handlers::health_check))
        .merge(api_routes)
        .merge(
            Router::from(
                SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()),
            )
            .layer(Extension(state.clone())),
        )
        .layer(TraceLayer::new_for_http())
}

pub async fn get_app(db_pool: PgPool, config: &Config) -> AppRouter {
    // Initialize Casbin enforcer
    // Try multiple paths to find the model file
    let model_paths = [
        "shared/auth/model.conf",             // From workspace root
        "../../../shared/auth/model.conf",    // From services/user_service/api
        "../../../../shared/auth/model.conf", // From target/debug
    ];

    let model_path = model_paths
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .copied()
        .unwrap_or("shared/auth/model.conf");

    let enforcer = create_enforcer(&config.database_url, Some(model_path))
        .await
        .expect("Failed to initialize Casbin enforcer");

    // Initialize repositories
    let user_repo = PgUserRepository::new(db_pool.clone());
    let tenant_repo = PgTenantRepository::new(db_pool.clone());
    let session_repo = PgSessionRepository::new(db_pool.clone());

    // Initialize service
    let auth_service = AuthServiceImpl::new(
        user_repo.clone(),
        tenant_repo.clone(),
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
        kanidm_client: create_kanidm_client(config),
        user_repo: Some(Arc::new(user_repo)),
        tenant_repo: Some(Arc::new(tenant_repo)),
    };

    let authz_state = AuthzState {
        enforcer: enforcer.clone(),
        jwt_secret: config.jwt_secret.clone(),
        kanidm_client: create_kanidm_client(config),
    };

    // Public routes (no auth required)
    let public_routes = Router::new()
        .layer(Extension(state.clone()))
        .route("/health", get(handlers::health_check))
        .route("/api/v1/auth/register", post(handlers::register::<ConcreteAuthService>))
        .route("/api/v1/auth/login", post(handlers::login::<ConcreteAuthService>))
        .route("/api/v1/auth/refresh", post(handlers::refresh_token::<ConcreteAuthService>))
        .route("/api/v1/auth/logout", post(handlers::logout::<ConcreteAuthService>))
        // OAuth2 endpoints
        .route("/api/v1/auth/oauth/authorize", post(oauth_handlers::oauth_authorize::<ConcreteAuthService>))
        .route("/api/v1/auth/oauth/callback", post(oauth_handlers::oauth_callback::<ConcreteAuthService>))
        .route("/api/v1/auth/oauth/refresh", post(oauth_handlers::oauth_refresh::<ConcreteAuthService>));

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        .layer(Extension(state.clone()))
        .route("/api/v1/users", get(handlers::list_users::<ConcreteAuthService>))
        .route("/api/v1/users/{user_id}", get(handlers::get_user::<ConcreteAuthService>))
        // Low-level policy management (for advanced use cases)
        .route(
            "/api/v1/admin/policies",
            post(handlers::add_policy::<ConcreteAuthService>).delete(handlers::remove_policy::<ConcreteAuthService>),
        )
        .layer(shared_auth::CasbinAuthLayer::new(authz_state));

    // Swagger UI
    let swagger_routes = Router::from(
        SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()),
    )
    .layer(Extension(state.clone()));

    // Build application by merging routers
    Router::new()
        .layer(Extension(state))
        .merge(public_routes)
        .merge(protected_routes)
        .merge(swagger_routes)
        .layer(TraceLayer::new_for_http())
}
