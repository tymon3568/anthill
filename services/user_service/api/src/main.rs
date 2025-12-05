use axum::extract::{DefaultBodyLimit, FromRef};
use axum::http::Method;
use axum::routing::{delete, get, post, put};
use axum::{
    http::{header, HeaderValue},
    Extension, Router,
};
use shared_auth::enforcer::{create_enforcer, SharedEnforcer};
use shared_kanidm_client::{KanidmClient, KanidmConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use user_service_api::{
    admin_handlers, handlers, permission_handlers, profile_handlers, AppState, ProfileAppState,
};
use user_service_infra::auth::{
    AuthServiceImpl, PgSessionRepository, PgTenantRepository, PgUserProfileRepository,
    PgUserRepository, ProfileServiceImpl,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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

    // Validate CORS configuration for production
    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    let rust_env = std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
    let is_production = app_env == "production" || rust_env == "production";

    if is_production && config.get_cors_origins().is_empty() {
        panic!(
            "CORS_ORIGINS must be configured in production environment. \
             Set CORS_ORIGINS=https://your-domain.com,https://admin.your-domain.com"
        );
    }

    tracing::info!("✅ Configuration loaded");

    // Initialize database connection pool
    let db_pool = shared_db::init_pool(&config.database_url, 5)
        .await
        .expect("Failed to connect to database");

    tracing::info!("✅ Database connected");

    // Initialize Casbin enforcer
    let enforcer = create_enforcer(&config.database_url, Some(&config.casbin_model_path))
        .await
        .expect("Failed to initialize Casbin enforcer");

    tracing::info!("✅ Casbin enforcer initialized");

    // Initialize Kanidm client (optional - falls back to dev mode if not configured)
    let kanidm_client =
        if let (Some(url), Some(client_id), Some(client_secret), Some(redirect_uri)) = (
            config.kanidm_url.as_ref(),
            config.kanidm_client_id.as_ref(),
            config.kanidm_client_secret.as_ref(),
            config.kanidm_redirect_url.as_ref(),
        ) {
            let kanidm_config = KanidmConfig {
                kanidm_url: url.clone(),
                client_id: client_id.clone(),
                client_secret: client_secret.clone(),
                redirect_uri: redirect_uri.clone(),
                scopes: vec![
                    "openid".to_string(),
                    "profile".to_string(),
                    "email".to_string(),
                    "groups".to_string(),
                ],
                skip_jwt_verification: false,
                allowed_issuers: vec![url.clone()],
                expected_audience: Some(client_id.clone()),
            };

            KanidmClient::new(kanidm_config).expect("Failed to initialize Kanidm client")
        } else {
            // Check if we're in production - dev mode is not allowed
            let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
            let rust_env = std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());

            if app_env == "production" || rust_env == "production" {
                panic!(
                    "Kanidm configuration is required in production environment. \
                 Set KANIDM_URL, KANIDM_CLIENT_ID, KANIDM_CLIENT_SECRET, \
                 and KANIDM_REDIRECT_URL environment variables."
                );
            }

            tracing::warn!("⚠️ Kanidm configuration not found - using dev mode (legacy JWT only)");

            // Create a dummy Kanidm client for dev mode
            let dev_config = KanidmConfig {
                kanidm_url: "http://localhost:8300".to_string(),
                client_id: "dev".to_string(),
                client_secret: "dev".to_string(),
                redirect_uri: "http://localhost:8000/oauth/callback".to_string(),
                scopes: vec!["openid".to_string()],
                skip_jwt_verification: true, // DEV MODE ONLY
                allowed_issuers: vec!["http://localhost:8300".to_string()],
                expected_audience: Some("dev".to_string()),
            };

            KanidmClient::new(dev_config).expect("Failed to initialize dev Kanidm client")
        };

    tracing::info!("✅ Kanidm client initialized");

    // Initialize repositories
    let user_repo = PgUserRepository::new(db_pool.clone());
    let tenant_repo = PgTenantRepository::new(db_pool.clone());
    let session_repo = PgSessionRepository::new(db_pool.clone());
    let profile_repo = PgUserProfileRepository::new(db_pool.clone());

    // Initialize services
    let auth_service = AuthServiceImpl::new(
        user_repo.clone(),
        tenant_repo.clone(),
        session_repo,
        config.jwt_secret.clone(),
        config.jwt_expiration,
        config.jwt_refresh_expiration,
    );

    let profile_service =
        ProfileServiceImpl::new(Arc::new(profile_repo), Arc::new(user_repo.clone()));

    // Create application states
    let state = AppState {
        auth_service: Arc::new(auth_service),
        enforcer: enforcer.clone(),
        jwt_secret: config.jwt_secret.clone(),
        kanidm_client: kanidm_client.clone(),
        user_repo: Some(Arc::new(user_repo)),
        tenant_repo: Some(Arc::new(tenant_repo)),
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

    impl FromRef<CombinedState>
        for AppState<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>
    {
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

    impl shared_auth::extractors::KanidmClientProvider for CombinedState {
        fn get_kanidm_client(&self) -> &KanidmClient {
            &self.app.kanidm_client
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
        .layer(Extension(combined_state.app.clone()))
        .route(
            "/api/v1/auth/register",
            post(
                handlers::register::<
                    AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>,
                >,
            ),
        )
        .route(
            "/api/v1/auth/login",
            post(
                handlers::login::<
                    AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>,
                >,
            ),
        )
        .route(
            "/api/v1/auth/refresh",
            post(
                handlers::refresh_token::<
                    AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>,
                >,
            ),
        )
        .route(
            "/api/v1/auth/logout",
            post(
                handlers::logout::<
                    AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>,
                >,
            ),
        );

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        .layer(Extension(combined_state.app.clone()))
        .route("/api/v1/users", get(handlers::list_users::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>))
        .route("/api/v1/users/{user_id}", get(handlers::get_user::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>))
        // Permission checking routes
        .route("/api/v1/users/permissions/check", get(permission_handlers::check_permission::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>))
        .route("/api/v1/users/permissions", get(permission_handlers::get_user_permissions::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>))
        .route("/api/v1/users/roles", get(permission_handlers::get_user_roles::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>))
        .route("/api/v1/users/tenant/validate", get(permission_handlers::validate_tenant_access::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>))
        // Low-level policy management (for advanced use cases)
        .route(
            "/api/v1/admin/policies",
            post(handlers::add_policy::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>).delete(handlers::remove_policy::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>),
        );

    // Admin role and permission management routes
    let admin_routes = Router::new()
        .layer(Extension(combined_state.app.clone()))
        // Role management
        .route("/api/v1/admin/roles",
            post(admin_handlers::create_role::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
            .get(admin_handlers::list_roles::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        )
        .route("/api/v1/admin/roles/{role_name}",
            put(admin_handlers::update_role::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
            .delete(admin_handlers::delete_role::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        )
        // User role assignment (additional GET endpoint)
        .route("/api/v1/admin/users/{user_id}/roles",
            get(admin_handlers::get_user_roles::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        )
        // New user role management endpoints
        .route("/api/v1/admin/users/{user_id}/roles/assign",
            post(admin_handlers::assign_role_to_user::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        )
        .route("/api/v1/admin/users/{user_id}/roles/{role_name}/remove",
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
        .layer(Extension(combined_state.profile.clone()))
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
        .route("/api/v1/users/profiles/{user_id}",
            get(profile_handlers::get_public_profile::<ProfileServiceImpl>)
        )
        .route("/api/v1/users/profiles/{user_id}/verification",
            put(profile_handlers::update_verification::<ProfileServiceImpl>)
        );

    // Combine all API routes with single unified state
    let api_routes = public_routes
        .merge(protected_routes)
        .merge(admin_routes)
        .merge(profile_routes);

    // Build application with routes and Swagger UI
    let app = Router::new()
        .layer(Extension(combined_state))
        .route("/health", get(handlers::health_check))
        .merge(api_routes)
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()))
        // CORS configuration
        .layer({
            let origins = config.get_cors_origins();
            let mut cors = CorsLayer::new()
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                ])
                .allow_headers([
                    header::CONTENT_TYPE,
                    header::AUTHORIZATION,
                ]);

            if origins.is_empty() {
                cors = cors.allow_origin(AllowOrigin::any());
                cors = cors.allow_credentials(false); // Explicitly disable credentials for wildcard origins
            } else {
                // Validate that wildcard is not in configured origins
                if origins.iter().any(|o| o == "*") {
                    panic!(
                        "CORS configuration error: wildcard origin '*' cannot be used with credentials. \
                         Either remove '*' and specify exact origins, or leave CORS_ORIGINS empty for development."
                    );
                }

                let header_values: Result<Vec<HeaderValue>, _> = origins
                    .into_iter()
                    .map(|origin| {
                        HeaderValue::from_str(&origin).map_err(|e| {
                            format!("Invalid CORS origin '{}': {}", origin, e)
                        })
                    })
                    .collect();

                match header_values {
                    Ok(values) => {
                        cors = cors.allow_origin(AllowOrigin::list(values));
                        // Only allow credentials when specific origins are configured
                        cors = cors.allow_credentials(true);
                    }
                    Err(e) => panic!("CORS configuration error: {}", e),
                }
            }

            cors
        })
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
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
