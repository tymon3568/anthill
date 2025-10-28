use axum::routing::{delete, get, post};
use axum::{http::{header, HeaderValue}, Router};
use shared_auth::enforcer::{create_enforcer, SharedEnforcer};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use user_service_api::{handlers, AppState};
use user_service_infra::auth::{
    AuthServiceImpl, PgSessionRepository, PgTenantRepository, PgUserRepository,
};

mod openapi;

#[derive(Clone)]
pub struct AuthzState {
    enforcer: SharedEnforcer,
    jwt_secret: String,
}

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
    let config = shared_config::Config::from_env().expect("Failed to load configuration");

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

    // TODO: Re-enable authorization state
    // let authz_state = AuthzState {
    //     enforcer: enforcer.clone(),
    //     jwt_secret: config.jwt_secret.clone(),
    // };

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
        // Admin routes for role management
        .route(
            "/api/v1/admin/policies",
            post(handlers::add_policy).delete(handlers::remove_policy),
        )
        .route(
            "/api/v1/admin/users/:user_id/roles",
            post(handlers::assign_role_to_user).delete(handlers::revoke_role_from_user),
        );
        // TODO: Re-enable authorization middleware
        // .layer(axum::middleware::from_fn_with_state(
        //     authz_state,
        //     shared_auth::middleware::casbin_middleware,
        // ));

    // Combine all API routes
    let api_routes = public_routes.merge(protected_routes).with_state(state);

    // Build application with routes and Swagger UI
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .merge(api_routes)
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()))
        // Security headers
        .layer(SetResponseHeaderLayer::if_not_present(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline';"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::HeaderName::from_static("x-permitted-cross-domain-policies"),
            HeaderValue::from_static("none"),
        ))
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
