use axum::routing::{get, post, put, delete};
use axum::{http::{header, HeaderValue}, Router};
use axum::extract::{DefaultBodyLimit, FromRef};
use shared_auth::enforcer::{create_enforcer, SharedEnforcer};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use user_service_api::{handlers, profile_handlers, admin_handlers, AppState, ProfileAppState};
use user_service_infra::auth::{
    AuthServiceImpl, PgSessionRepository, PgTenantRepository, PgUserRepository,
    ProfileServiceImpl, PgUserProfileRepository,
};

mod openapi;

#[derive(Clone)]
#[allow(dead_code)] // TODO: Re-enable when authorization middleware is activated
pub struct AuthzState {
    enforcer: SharedEnforcer,
    jwt_secret: String,
}

#[tokio::main]
async fn main() {
    // Export OpenAPI spec if feature is enabled
    #[cfg(feature = "export-spec")]
    {
        user_service_api::openapi::export_spec().expect("Failed to export OpenAPI spec");
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
    let profile_repo = PgUserProfileRepository::new(db_pool.clone());

    // Initialize services
    let auth_service = AuthServiceImpl::new(
        user_repo.clone(),
        tenant_repo,
        session_repo,
        config.jwt_secret.clone(),
        config.jwt_expiration,
        config.jwt_refresh_expiration,
    );
    
    let profile_service = ProfileServiceImpl::new(
        Arc::new(profile_repo),
        Arc::new(user_repo),
    );

    // Create application states
    let state = AppState {
        auth_service: Arc::new(auth_service),
        enforcer: enforcer.clone(),
        jwt_secret: config.jwt_secret.clone(),
    };
    
    let profile_state = ProfileAppState {
        profile_service: Arc::new(profile_service),
        jwt_secret: config.jwt_secret.clone(),
    };

    // Combined state for unified router (Axum best practice)
    #[derive(Clone)]
    struct CombinedState {
        app: AppState<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>,
        profile: ProfileAppState<ProfileServiceImpl>,
    }

    impl FromRef<CombinedState> for AppState<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>> {
        fn from_ref(state: &CombinedState) -> Self {
            state.app.clone()
        }
    }

    impl FromRef<CombinedState> for ProfileAppState<ProfileServiceImpl> {
        fn from_ref(state: &CombinedState) -> Self {
            state.profile.clone()
        }
    }

    impl shared_auth::extractors::JwtSecretProvider for CombinedState {
        fn get_jwt_secret(&self) -> &str {
            &self.app.jwt_secret
        }
    }

    let combined_state = CombinedState {
        app: state.clone(),
        profile: profile_state.clone(),
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
        // Low-level policy management (for advanced use cases)
        .route(
            "/api/v1/admin/policies",
            post(handlers::add_policy).delete(handlers::remove_policy),
        );
    
    // Admin role and permission management routes
    let admin_routes = Router::new()
        // Role management
        .route("/api/v1/admin/roles",
            post(admin_handlers::create_role::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
            .get(admin_handlers::list_roles::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        )
        .route("/api/v1/admin/roles/:role_name",
            put(admin_handlers::update_role::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
            .delete(admin_handlers::delete_role::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        )
        // User role assignment (additional GET endpoint)
        .route("/api/v1/admin/users/:user_id/roles",
            get(admin_handlers::get_user_roles::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        )
        // New user role management endpoints
        .route("/api/v1/admin/users/:user_id/roles/assign",
            post(admin_handlers::assign_role_to_user::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        )
        .route("/api/v1/admin/users/:user_id/roles/:role_name/remove",
            delete(admin_handlers::remove_role_from_user::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        )
        // Permission listing
        .route("/api/v1/admin/permissions",
            get(admin_handlers::list_permissions::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        );
    
        // TODO: Re-enable authorization middleware
        // .layer(axum::middleware::from_fn_with_state(
        //     authz_state,
        //     shared_auth::middleware::casbin_middleware,
        // ));
    
    // Profile routes (require authentication)
    let profile_routes = Router::new()
        .route("/api/v1/users/profile", 
            get(profile_handlers::get_profile::<ProfileServiceImpl>)
            .put(profile_handlers::update_profile::<ProfileServiceImpl>)
        )
        // Avatar upload with 5MB body limit to prevent 413 before handler validation
        .route("/api/v1/users/profile/avatar",
            post(profile_handlers::upload_avatar::<ProfileServiceImpl>)
                .layer(DefaultBodyLimit::max(5 * 1024 * 1024)) // 5MB
        )
        .route("/api/v1/users/profile/visibility", 
            put(profile_handlers::update_visibility::<ProfileServiceImpl>)
        )
        .route("/api/v1/users/profile/completeness", 
            get(profile_handlers::get_completeness::<ProfileServiceImpl>)
        )
        .route("/api/v1/users/profiles/search", 
            post(profile_handlers::search_profiles::<ProfileServiceImpl>)
        )
        .route("/api/v1/users/profiles/:user_id", 
            get(profile_handlers::get_public_profile::<ProfileServiceImpl>)
        )
        .route("/api/v1/users/profiles/:user_id/verification", 
            put(profile_handlers::update_verification::<ProfileServiceImpl>)
        );

    // Combine all API routes with single unified state
    let api_routes = public_routes
        .merge(protected_routes)
        .merge(admin_routes)
        .merge(profile_routes);

    // Build application with routes and Swagger UI
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .merge(api_routes)
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()))
        .with_state(combined_state)
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
