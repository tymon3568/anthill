mod handlers;
mod openapi;
mod extractors;

use axum::{
    routing::{get, post},
    Router,
    Extension,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use user_service_infra::auth::{PgUserRepository, PgTenantRepository, PgSessionRepository, AuthServiceImpl};
use handlers::AppState;
use shared_auth::enforcer::create_enforcer;

#[tokio::main]
async fn main() {
    // Export OpenAPI spec if feature is enabled
    #[cfg(feature = "export-spec")]
    {
        openapi::export_spec().expect("Failed to export OpenAPI spec");
        tracing::info!("📄 OpenAPI spec exported to shared/openapi/user.yaml");
    }

    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    tracing::info!("🚀 User Service Starting...");
    
    // Load configuration
    let config = shared_config::Config::from_env()
        .expect("Failed to load configuration");
    
    tracing::info!("✅ Configuration loaded");
    
    // Initialize database connection pool
    let db_pool = shared_db::init_pool(&config.database_url, 5)
        .await
        .expect("Failed to connect to database");
    
    tracing::info!("✅ Database connected");
    
    // Initialize Casbin enforcer
    let enforcer = create_enforcer(&config.database_url, None)
        .await
        .expect("Failed to initialize Casbin enforcer");
    
    tracing::info!("✅ Casbin enforcer initialized");
    
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
    
    tracing::info!("✅ Services initialized");
    
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
        .layer(Extension(state.enforcer.clone()))
        .layer(Extension(state.jwt_secret.clone()));
    
    // Combine all API routes
    let api_routes = public_routes
        .merge(protected_routes)
        .with_state(state);
    
    // Build application with routes and Swagger UI
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .merge(api_routes)
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http());
    
    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("🚀 User Service listening on http://{}", addr);
    tracing::info!("📚 Swagger UI available at http://{}/docs", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
