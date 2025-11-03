// Library exports for integration tests
pub mod admin_handlers;
pub mod extractors;
pub mod handlers;
pub mod oauth_handlers;
pub mod openapi;
pub mod profile_handlers;

// Re-export commonly used types for tests
pub use handlers::AppState;
pub use profile_handlers::ProfileAppState;

use axum::routing::{get, post};
use axum::Router;
use shared_auth::enforcer::create_enforcer;
use shared_auth::AuthzState;
use shared_config::Config;
use shared_kanidm_client::{KanidmClient, KanidmConfig};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use user_service_core::domains::auth::domain::service::AuthService;
use user_service_infra::auth::{
    AuthServiceImpl, PgSessionRepository, PgTenantRepository, PgUserRepository,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Create a dev Kanidm client for testing (JWT validation disabled)
fn create_dev_kanidm_client() -> KanidmClient {
    let config = KanidmConfig {
        kanidm_url: "http://localhost:8300".to_string(),
        client_id: "dev".to_string(),
        client_secret: "dev".to_string(),
        redirect_uri: "http://localhost:3000/oauth/callback".to_string(),
        scopes: vec!["openid".to_string()],
        skip_jwt_verification: true, // DEV/TEST MODE ONLY
        allowed_issuers: vec!["http://localhost:8300".to_string()],
        expected_audience: Some("dev".to_string()),
    };
    
    KanidmClient::new(config).expect("Failed to create dev Kanidm client")
}

/// Create router from app state (for testing)
pub fn create_router<S: AuthService + 'static>(state: AppState<S>) -> Router {
    let authz_state = AuthzState {
        enforcer: state.enforcer.clone(),
        jwt_secret: state.jwt_secret.clone(),
    };

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/api/v1/auth/register", post(handlers::register::<S>))
        .route("/api/v1/auth/login", post(handlers::login::<S>))
        .route("/api/v1/auth/refresh", post(handlers::refresh_token::<S>))
        .route("/api/v1/auth/logout", post(handlers::logout::<S>))
        // OAuth2 endpoints
        .route("/api/v1/auth/oauth/authorize", post(oauth_handlers::oauth_authorize::<S>))
        .route("/api/v1/auth/oauth/callback", post(oauth_handlers::oauth_callback::<S>))
        .route("/api/v1/auth/oauth/refresh", post(oauth_handlers::oauth_refresh::<S>));

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        .route("/api/v1/users", get(handlers::list_users::<S>))
        .route("/api/v1/users/{user_id}", get(handlers::get_user::<S>))
        // Low-level policy management (for advanced use cases)
        .route(
            "/api/v1/admin/policies",
            post(handlers::add_policy::<S>).delete(handlers::remove_policy::<S>),
        )
        .layer(shared_auth::CasbinAuthLayer::new(authz_state));

    // Combine all API routes
    let api_routes = public_routes.merge(protected_routes).with_state(state);

    // Build application with routes and Swagger UI
    Router::new()
        .route("/health", get(handlers::health_check))
        .merge(api_routes)
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
}

pub async fn get_app(db_pool: PgPool, config: &Config) -> Router {
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
        kanidm_client: create_dev_kanidm_client(),
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
        .route("/api/v1/auth/logout", post(handlers::logout))
        // OAuth2 endpoints
        .route("/api/v1/auth/oauth/authorize", post(oauth_handlers::oauth_authorize))
        .route("/api/v1/auth/oauth/callback", post(oauth_handlers::oauth_callback))
        .route("/api/v1/auth/oauth/refresh", post(oauth_handlers::oauth_refresh));

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        .route("/api/v1/users", get(handlers::list_users))
        .route("/api/v1/users/{user_id}", get(handlers::get_user))
        // Low-level policy management (for advanced use cases)
        .route(
            "/api/v1/admin/policies",
            post(handlers::add_policy).delete(handlers::remove_policy),
        )
        .layer(shared_auth::CasbinAuthLayer::new(authz_state));

    // Combine all API routes
    let api_routes = public_routes.merge(protected_routes).with_state(state);

    // Build application with routes and Swagger UI
    Router::new()
        .route("/health", get(handlers::health_check))
        .merge(api_routes)
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
}
