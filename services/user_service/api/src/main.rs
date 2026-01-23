use axum::extract::DefaultBodyLimit;
use axum::http::Method;
use axum::routing::{delete, get, post, put};
use axum::{
    http::{header, HeaderValue},
    Extension, Router,
};
use shared_auth::enforcer::create_enforcer;
use shared_auth::middleware::AuthzState;
use shared_rate_limit::{RateLimitConfig, RateLimitEndpoint, RateLimitLayer, RateLimitState};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use user_service_api::{
    admin_handlers, handlers, invitation_handlers, password_reset_handlers, permission_handlers,
    profile_handlers, rate_limiter::InvitationRateLimiter, verification_handlers, AppState,
    ProfileAppState,
};
use user_service_core::domains::auth::domain::authz_version_repository::AuthzVersionRepository;
use user_service_infra::auth::{
    AuthServiceImpl, EmailSender, EmailVerificationServiceImpl, InvitationServiceImpl,
    PasswordResetServiceImpl, PgEmailVerificationRepository, PgInvitationRepository,
    PgPasswordResetRepository, PgSessionRepository, PgTenantRepository, PgUserProfileRepository,
    PgUserRepository, ProfileServiceImpl, RedisAuthzVersionRepository, SmtpConfig, SmtpEmailSender,
};
use user_service_infra::storage::{StorageClient, StorageConfig};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod openapi;

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

    // Initialize storage client for file uploads (RustFS)
    let storage_client = match StorageConfig::from_env() {
        Ok(storage_config) => match StorageClient::new(storage_config).await {
            Ok(client) => {
                tracing::info!("✅ Storage client initialized (RustFS)");
                Some(Arc::new(client))
            },
            Err(e) => {
                tracing::warn!("⚠️ Failed to initialize storage client: {} - avatar uploads will use placeholder URLs", e);
                None
            },
        },
        Err(e) => {
            tracing::warn!(
                "⚠️ Storage configuration not found: {} - avatar uploads will use placeholder URLs",
                e
            );
            None
        },
    };

    let profile_service = if let Some(storage) = storage_client {
        ProfileServiceImpl::with_storage(
            Arc::new(profile_repo),
            Arc::new(user_repo.clone()),
            storage,
        )
    } else {
        ProfileServiceImpl::new(Arc::new(profile_repo), Arc::new(user_repo.clone()))
    };

    // Initialize invitation service
    let invitation_repo = PgInvitationRepository::new(db_pool.clone());
    let invitation_service = InvitationServiceImpl::new(
        invitation_repo,
        user_repo.clone(),
        enforcer.clone(),
        config.invitation_expiry_hours,
        config.invitation_max_attempts,
        config.invitation_max_per_admin_per_day,
    );

    // Initialize email verification service
    let verification_repo = PgEmailVerificationRepository::new(db_pool.clone());

    // Initialize SMTP email sender
    let smtp_config = SmtpConfig {
        host: config.smtp_host.clone().unwrap_or_default(),
        port: config.smtp_port,
        username: config.smtp_username.clone(),
        password: config.smtp_password.clone(),
        from_email: config.smtp_from_email.clone(),
        from_name: config.smtp_from_name.clone(),
        use_tls: config.smtp_tls,
    };
    let email_sender = Arc::new(
        SmtpEmailSender::new(smtp_config.clone()).expect("Failed to initialize SMTP email sender"),
    );

    if email_sender.is_available() {
        tracing::info!("✅ SMTP email sender initialized");
    } else {
        tracing::warn!("⚠️ SMTP not configured - emails will be logged but not sent");
    }

    let email_verification_service = EmailVerificationServiceImpl::new(
        Arc::new(verification_repo),
        Arc::new(user_repo.clone()),
        config.verification_base_url.clone(),
        config.verification_expiry_hours,
        config.rate_limit_resend_max,
        (config.rate_limit_resend_window / 60) as i64, // Convert seconds to minutes
        email_sender.clone(),
    );
    let email_verification_service = Arc::new(email_verification_service);

    // Initialize AuthZ version repository (optional - for immediate-effect permission changes)
    let authz_version_repo: Option<Arc<dyn AuthzVersionRepository>> =
        if let Some(redis_url) = &config.redis_url {
            tracing::info!("✅ Initializing AuthZ version repository with Redis");
            Some(Arc::new(RedisAuthzVersionRepository::new(db_pool.clone(), redis_url).await))
        } else {
            tracing::info!("ℹ️ Redis not configured - AuthZ versioning disabled");
            None
        };

    // Initialize password reset service
    let password_reset_repo = PgPasswordResetRepository::new(db_pool.clone());
    let password_reset_session_repo = PgSessionRepository::new(db_pool.clone());
    let password_reset_service = PasswordResetServiceImpl::new(
        Arc::new(password_reset_repo),
        Arc::new(user_repo.clone()),
        Arc::new(password_reset_session_repo),
        config.verification_base_url.clone(), // Reuse verification base URL for reset links
        1,                                    // 1 hour expiry for reset tokens
        config.rate_limit_forgot_max,
        (config.rate_limit_forgot_window / 60) as i64, // Convert seconds to minutes
        email_sender.clone(),
        8, // Minimum password length
    );
    let password_reset_service = Arc::new(password_reset_service);

    // Create application states
    let state = AppState {
        auth_service: Arc::new(auth_service),
        enforcer: enforcer.clone(),
        jwt_secret: config.jwt_secret.clone(),
        user_repo: Some(Arc::new(user_repo)),
        tenant_repo: Some(Arc::new(tenant_repo)),
        invitation_service: Some(Arc::new(invitation_service)),
        email_verification_service: Some(email_verification_service.clone()),
        authz_version_repo,
        config: config.clone(),
        invitation_rate_limiter: Arc::new(InvitationRateLimiter::default()),
        email_sender: Some(email_sender.clone()),
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

    impl shared_auth::extractors::JwtSecretProvider for CombinedState {
        fn get_jwt_secret(&self) -> &str {
            &self.app.jwt_secret
        }
    }

    let combined_state = CombinedState {
        app: state.clone(),
        profile: profile_state.clone(),
    };

    // Authorization state for Casbin middleware
    let authz_state = AuthzState {
        enforcer: enforcer.clone(),
        jwt_secret: config.jwt_secret.clone(),
    };

    tracing::info!("✅ Services initialized");

    // Initialize rate limiting
    let rate_limit_config = RateLimitConfig {
        redis_url: config.redis_url.clone(),
        login_max_attempts: config.rate_limit_login_max,
        login_window_seconds: config.rate_limit_login_window,
        register_max_attempts: config.rate_limit_register_max,
        register_window_seconds: config.rate_limit_register_window,
        forgot_password_max: config.rate_limit_forgot_max,
        forgot_password_window: config.rate_limit_forgot_window,
        accept_invite_max: config.rate_limit_accept_invite_max,
        accept_invite_window: config.rate_limit_accept_invite_window,
        refresh_max: config.rate_limit_refresh_max,
        refresh_window: config.rate_limit_refresh_window,
        resend_verification_max: config.rate_limit_resend_max,
        resend_verification_window: config.rate_limit_resend_window,
        file_upload_max: config.rate_limit_file_upload_max,
        file_upload_window: config.rate_limit_file_upload_window,
        lockout_threshold: config.rate_limit_lockout_threshold,
        lockout_duration_seconds: config.rate_limit_lockout_duration,
        enabled: config.rate_limit_enabled,
        trusted_ips: config.rate_limit_trusted_ips.clone(),
        trust_proxy_headers: config.rate_limit_trust_proxy_headers,
        proxy_count: config.rate_limit_proxy_count,
        ..Default::default()
    };
    let rate_limit_state = RateLimitState::from_config(rate_limit_config).await;

    if config.rate_limit_enabled {
        tracing::info!(
            "✅ Rate limiting enabled (login: {}/{}s, register: {}/{}s)",
            config.rate_limit_login_max,
            config.rate_limit_login_window,
            config.rate_limit_register_max,
            config.rate_limit_register_window
        );
    } else {
        tracing::warn!("⚠️ Rate limiting is DISABLED");
    }

    // Public routes (no auth required) with rate limiting
    // Register route with rate limiting
    let register_route = Router::new()
        .route(
            "/api/v1/auth/register",
            post(
                handlers::register::<
                    AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>,
                >,
            ),
        )
        .layer(Extension(combined_state.app.clone()))
        .layer(RateLimitLayer::new(rate_limit_state.clone(), RateLimitEndpoint::Register));

    // Login route with rate limiting
    let login_route = Router::new()
        .route(
            "/api/v1/auth/login",
            post(
                handlers::login::<
                    AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>,
                >,
            ),
        )
        .layer(Extension(combined_state.app.clone()))
        .layer(RateLimitLayer::new(rate_limit_state.clone(), RateLimitEndpoint::Login));

    // Refresh route with rate limiting
    let refresh_route = Router::new()
        .route(
            "/api/v1/auth/refresh",
            post(
                handlers::refresh_token::<
                    AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>,
                >,
            ),
        )
        .layer(Extension(combined_state.app.clone()))
        .layer(RateLimitLayer::new(rate_limit_state.clone(), RateLimitEndpoint::Refresh));

    // Accept invite route with rate limiting
    let accept_invite_route = Router::new()
        .route(
            "/api/v1/auth/accept-invite",
            post(
                invitation_handlers::accept_invitation::<
                    AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>,
                >,
            ),
        )
        .layer(Extension(combined_state.app.clone()))
        .layer(RateLimitLayer::new(rate_limit_state.clone(), RateLimitEndpoint::AcceptInvite));

    // Logout route (no rate limiting needed)
    let logout_route = Router::new()
        .route(
            "/api/v1/auth/logout",
            post(
                handlers::logout::<
                    AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>,
                >,
            ),
        )
        .layer(Extension(combined_state.app.clone()));

    // Check tenant slug availability (public endpoint, no auth required)
    let check_tenant_slug_route = Router::new()
        .route(
            "/api/v1/auth/check-tenant-slug",
            get(handlers::check_tenant_slug::<
                AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>,
            >),
        )
        .layer(Extension(combined_state.app.clone()));

    // Email verification routes with rate limiting
    let verify_email_route = Router::new()
        .route(
            "/api/v1/auth/verify-email",
            post(
                verification_handlers::verify_email::<
                    EmailVerificationServiceImpl<PgEmailVerificationRepository, PgUserRepository>,
                >,
            ),
        )
        .layer(Extension(email_verification_service.clone()));

    let resend_verification_route = Router::new()
        .route(
            "/api/v1/auth/resend-verification",
            post(
                verification_handlers::resend_verification::<
                    EmailVerificationServiceImpl<PgEmailVerificationRepository, PgUserRepository>,
                >,
            ),
        )
        .layer(Extension(email_verification_service.clone()))
        .layer(RateLimitLayer::new(
            rate_limit_state.clone(),
            RateLimitEndpoint::ResendVerification,
        ));

    // Password reset routes with rate limiting
    let forgot_password_route = Router::new()
        .route(
            "/api/v1/auth/forgot-password",
            post(
                password_reset_handlers::forgot_password::<
                    PasswordResetServiceImpl<
                        PgPasswordResetRepository,
                        PgUserRepository,
                        PgSessionRepository,
                    >,
                >,
            ),
        )
        .layer(Extension(password_reset_service.clone()))
        .layer(RateLimitLayer::new(rate_limit_state.clone(), RateLimitEndpoint::ForgotPassword));

    let reset_password_route = Router::new()
        .route(
            "/api/v1/auth/reset-password",
            post(
                password_reset_handlers::reset_password::<
                    PasswordResetServiceImpl<
                        PgPasswordResetRepository,
                        PgUserRepository,
                        PgSessionRepository,
                    >,
                >,
            ),
        )
        .layer(Extension(password_reset_service.clone()));

    let validate_reset_token_route = Router::new()
        .route(
            "/api/v1/auth/validate-reset-token",
            post(
                password_reset_handlers::validate_reset_token::<
                    PasswordResetServiceImpl<
                        PgPasswordResetRepository,
                        PgUserRepository,
                        PgSessionRepository,
                    >,
                >,
            ),
        )
        .layer(Extension(password_reset_service.clone()));

    // Combine public routes
    let public_routes = Router::new()
        .merge(register_route)
        .merge(login_route)
        .merge(refresh_route)
        .merge(accept_invite_route)
        .merge(logout_route)
        .merge(check_tenant_slug_route)
        .merge(verify_email_route)
        .merge(resend_verification_route)
        .merge(forgot_password_route)
        .merge(reset_password_route)
        .merge(validate_reset_token_route);

    // Protected routes (require authentication)
    let protected_routes = Router::new()
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
        )
        .layer(Extension(combined_state.app.clone()))
        .layer(shared_auth::CasbinAuthLayer::new(authz_state.clone()))
        .layer(Extension(authz_state.clone()));

    // Admin role and permission management routes
    let admin_routes = Router::new()
        // Admin user management
        .route("/api/v1/admin/users",
            post(admin_handlers::admin_create_user::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
            .get(handlers::list_users::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        )
        // Admin user lifecycle management
        .route("/api/v1/admin/users/{user_id}/suspend",
            post(admin_handlers::suspend_user::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        )
        .route("/api/v1/admin/users/{user_id}/unsuspend",
            post(admin_handlers::unsuspend_user::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        )
        .route("/api/v1/admin/users/{user_id}",
            delete(admin_handlers::delete_user::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        )
        .route("/api/v1/admin/users/{user_id}/reset-password",
            post(admin_handlers::reset_user_password::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        )
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
        )
        // Invitation management
        .route("/api/v1/admin/users/invite",
            post(invitation_handlers::create_invitation::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        )
        .route("/api/v1/admin/users/invitations",
            get(invitation_handlers::list_invitations::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        )
        .route("/api/v1/admin/users/invitations/{invitation_id}",
            delete(invitation_handlers::revoke_invitation::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        )
        .route("/api/v1/admin/users/invitations/{invitation_id}/resend",
            post(invitation_handlers::resend_invitation::<AuthServiceImpl<PgUserRepository, PgTenantRepository, PgSessionRepository>>)
        )
        .layer(Extension(combined_state.app.clone()))
        // Apply authorization middleware to admin routes
        .layer(shared_auth::CasbinAuthLayer::new(authz_state.clone()))
        .layer(Extension(authz_state.clone()));

    // Personal profile routes (require authentication only, no Casbin - users manage their own profile)
    // Avatar upload route with rate limiting (separated due to multiple layers)
    let avatar_upload_route = Router::new()
        .route("/api/v1/users/profile/avatar",
            post(profile_handlers::upload_avatar::<ProfileServiceImpl>)
        )
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024)) // 5MB
        .layer(RateLimitLayer::with_jwt_secret(
            rate_limit_state.clone(),
            RateLimitEndpoint::FileUpload,
            config.jwt_secret.clone(),
        ))
        .layer(Extension(combined_state.profile.clone()))
        .layer(Extension(authz_state.clone()));

    let personal_profile_routes = Router::new()
        .route(
            "/api/v1/users/profile",
            get(profile_handlers::get_profile::<ProfileServiceImpl>)
                .put(profile_handlers::update_profile::<ProfileServiceImpl>),
        )
        .route(
            "/api/v1/users/profile/visibility",
            put(profile_handlers::update_visibility::<ProfileServiceImpl>),
        )
        .route(
            "/api/v1/users/profile/completeness",
            get(profile_handlers::get_completeness::<ProfileServiceImpl>),
        )
        .route(
            "/api/v1/users/profiles/search",
            post(profile_handlers::search_profiles::<ProfileServiceImpl>),
        )
        .route(
            "/api/v1/users/profiles/{user_id}",
            get(profile_handlers::get_public_profile::<ProfileServiceImpl>),
        )
        .merge(avatar_upload_route)
        .layer(Extension(combined_state.profile.clone()))
        .layer(Extension(authz_state.clone()));

    // Admin profile routes (require authentication + Casbin authorization)
    let admin_profile_routes = Router::new()
        .route(
            "/api/v1/users/profiles/{user_id}/verification",
            put(profile_handlers::update_verification::<ProfileServiceImpl>),
        )
        .layer(Extension(combined_state.profile.clone()))
        .layer(shared_auth::CasbinAuthLayer::new(authz_state.clone()))
        .layer(Extension(authz_state.clone()));

    // Combine all API routes with single unified state
    let api_routes = public_routes
        .merge(protected_routes)
        .merge(admin_routes)
        .merge(personal_profile_routes)
        .merge(admin_profile_routes);

    // Build application with routes and Swagger UI
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .merge(api_routes)
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()))
        .layer(Extension(combined_state))
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
                    axum::http::header::HeaderName::from_static("x-tenant-id"),
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
