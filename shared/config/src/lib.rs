use serde::Deserialize;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Database connection URL
    pub database_url: String,

    /// JWT secret key for signing tokens
    pub jwt_secret: String,

    /// JWT access token expiration in seconds (default: 900 = 15 minutes)
    #[serde(default = "default_jwt_expiration")]
    pub jwt_expiration: i64,

    /// JWT refresh token expiration in seconds (default: 604800 = 7 days)
    #[serde(default = "default_jwt_refresh_expiration")]
    pub jwt_refresh_expiration: i64,

    /// Server host address
    #[serde(default = "default_host")]
    pub host: String,

    /// Server port (override per service via PORT environment variable)
    /// Standard ports:
    /// - User Service: 8000
    /// - Inventory Service: 8001
    /// - Order Service: 8002
    /// - Integration Service: 8003
    #[serde(default = "default_port")]
    pub port: u16,

    /// CORS allowed origins (comma-separated list, optional)
    pub cors_origins: Option<String>,

    /// NATS server URL (optional - for event-driven messaging)
    pub nats_url: Option<String>,

    /// Redis server URL (optional - for caching and distributed locking)
    pub redis_url: Option<String>,

    /// Casbin model configuration file path
    #[serde(default = "default_casbin_model_path")]
    pub casbin_model_path: String,

    /// Maximum database connections (optional, default: 10)
    pub max_connections: Option<u32>,

    /// Invitation base URL for generating invite links
    #[serde(default = "default_invitation_base_url")]
    pub invitation_base_url: String,

    /// Invitation expiry in hours
    #[serde(default = "default_invitation_expiry_hours")]
    pub invitation_expiry_hours: i64,

    /// Maximum attempts for invitation acceptance
    #[serde(default = "default_invitation_max_attempts")]
    pub invitation_max_attempts: i32,

    /// Maximum invitations per admin per day
    #[serde(default = "default_invitation_max_per_admin_per_day")]
    pub invitation_max_per_admin_per_day: i32,

    // ===== Email Verification Configuration =====
    /// Base URL for email verification links
    #[serde(default = "default_verification_base_url")]
    pub verification_base_url: String,

    /// Email verification token expiry in hours (default: 24)
    #[serde(default = "default_verification_expiry_hours")]
    pub verification_expiry_hours: i64,

    // ===== SMTP Configuration =====
    /// SMTP server host (optional - email sending disabled if not configured)
    pub smtp_host: Option<String>,

    /// SMTP server port (default: 587 for TLS)
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,

    /// SMTP username for authentication (optional)
    pub smtp_username: Option<String>,

    /// SMTP password for authentication (optional)
    pub smtp_password: Option<String>,

    /// SMTP from email address
    #[serde(default = "default_smtp_from_email")]
    pub smtp_from_email: String,

    /// SMTP from name
    #[serde(default = "default_smtp_from_name")]
    pub smtp_from_name: String,

    /// Enable SMTP TLS (default: true)
    #[serde(default = "default_smtp_tls")]
    pub smtp_tls: bool,

    // ===== Rate Limiting Configuration =====
    /// Enable rate limiting (default: true)
    #[serde(default = "default_rate_limit_enabled")]
    pub rate_limit_enabled: bool,

    /// Login endpoint: max attempts per IP (default: 5)
    #[serde(default = "default_rate_limit_login_max")]
    pub rate_limit_login_max: u32,

    /// Login endpoint: window in seconds (default: 900 = 15 min)
    #[serde(default = "default_rate_limit_login_window")]
    pub rate_limit_login_window: u64,

    /// Register endpoint: max attempts per IP (default: 3)
    #[serde(default = "default_rate_limit_register_max")]
    pub rate_limit_register_max: u32,

    /// Register endpoint: window in seconds (default: 3600 = 1 hour)
    #[serde(default = "default_rate_limit_register_window")]
    pub rate_limit_register_window: u64,

    /// Forgot password: max attempts per email (default: 3)
    #[serde(default = "default_rate_limit_forgot_max")]
    pub rate_limit_forgot_max: u32,

    /// Forgot password: window in seconds (default: 3600 = 1 hour)
    #[serde(default = "default_rate_limit_forgot_window")]
    pub rate_limit_forgot_window: u64,

    /// Accept invite: max attempts per IP (default: 10)
    #[serde(default = "default_rate_limit_accept_invite_max")]
    pub rate_limit_accept_invite_max: u32,

    /// Accept invite: window in seconds (default: 3600 = 1 hour)
    #[serde(default = "default_rate_limit_accept_invite_window")]
    pub rate_limit_accept_invite_window: u64,

    /// Account lockout: threshold for consecutive failures (default: 10)
    #[serde(default = "default_rate_limit_lockout_threshold")]
    pub rate_limit_lockout_threshold: u32,

    /// Account lockout: duration in seconds (default: 3600 = 1 hour)
    #[serde(default = "default_rate_limit_lockout_duration")]
    pub rate_limit_lockout_duration: u64,

    /// Refresh token endpoint: max attempts per user (default: 30)
    #[serde(default = "default_rate_limit_refresh_max")]
    pub rate_limit_refresh_max: u32,

    /// Refresh token endpoint: window in seconds (default: 3600 = 1 hour)
    #[serde(default = "default_rate_limit_refresh_window")]
    pub rate_limit_refresh_window: u64,

    /// Resend verification: max attempts per user (default: 3)
    #[serde(default = "default_rate_limit_resend_max")]
    pub rate_limit_resend_max: u32,

    /// Resend verification: window in seconds (default: 3600 = 1 hour)
    #[serde(default = "default_rate_limit_resend_window")]
    pub rate_limit_resend_window: u64,

    /// Trust proxy headers for IP extraction (default: false)
    /// Only enable if behind a trusted reverse proxy
    #[serde(default)]
    pub rate_limit_trust_proxy_headers: bool,

    /// Number of trusted proxies in front of the service (default: 0)
    /// Used with trust_proxy_headers for rightmost-trusted IP extraction
    #[serde(default)]
    pub rate_limit_proxy_count: u32,

    /// Trusted IPs that bypass rate limiting (comma-separated, supports CIDR notation, optional)
    pub rate_limit_trusted_ips: Option<String>,

    // ===== Decision Cache Configuration =====
    /// Enable authorization decision caching (default: true)
    #[serde(default = "default_decision_cache_enabled")]
    pub decision_cache_enabled: bool,

    /// Decision cache TTL in seconds (default: 15)
    #[serde(default = "default_decision_cache_ttl_seconds")]
    pub decision_cache_ttl_seconds: u64,

    /// Decision cache max entries for in-memory backend (default: 10000)
    #[serde(default = "default_decision_cache_max_entries")]
    pub decision_cache_max_entries: u64,

    // ===== Audit Logging Configuration =====
    /// Enable authorization audit logging (default: true)
    #[serde(default = "default_audit_log_enabled")]
    pub audit_log_enabled: bool,

    /// Audit log retention in days (default: 90)
    #[serde(default = "default_audit_log_retention_days")]
    pub audit_log_retention_days: u32,

    /// Audit log batch size for background writer (default: 100)
    #[serde(default = "default_audit_log_batch_size")]
    pub audit_log_batch_size: u32,

    /// Audit log flush interval in milliseconds (default: 1000)
    #[serde(default = "default_audit_log_flush_interval_ms")]
    pub audit_log_flush_interval_ms: u64,

    // ===== Cookie Configuration =====
    /// Cookie domain (optional - if not set, cookies are set for the request host)
    pub cookie_domain: Option<String>,

    /// Enable secure flag on cookies (default: true for production, should be false for local dev)
    #[serde(default = "default_cookie_secure")]
    pub cookie_secure: bool,

    /// Cookie SameSite attribute (default: "Strict")
    #[serde(default = "default_cookie_same_site")]
    pub cookie_same_site: String,

    /// Cookie path (default: "/")
    #[serde(default = "default_cookie_path")]
    pub cookie_path: String,
}

fn default_jwt_expiration() -> i64 {
    900 // 15 minutes
}

fn default_jwt_refresh_expiration() -> i64 {
    604_800 // 7 days
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_casbin_model_path() -> String {
    "shared/auth/model.conf".to_string()
}

fn default_invitation_base_url() -> String {
    "https://app.example.com".to_string()
}

fn default_invitation_expiry_hours() -> i64 {
    48
}

fn default_invitation_max_attempts() -> i32 {
    5
}

fn default_invitation_max_per_admin_per_day() -> i32 {
    10
}

fn default_verification_base_url() -> String {
    "https://app.example.com".to_string()
}

fn default_verification_expiry_hours() -> i64 {
    24
}

fn default_smtp_port() -> u16 {
    587
}

fn default_smtp_from_email() -> String {
    "noreply@example.com".to_string()
}

fn default_smtp_from_name() -> String {
    "Anthill".to_string()
}

fn default_smtp_tls() -> bool {
    true
}

// Rate limiting defaults
fn default_rate_limit_enabled() -> bool {
    true
}

fn default_rate_limit_login_max() -> u32 {
    5
}

fn default_rate_limit_login_window() -> u64 {
    900 // 15 minutes
}

fn default_rate_limit_register_max() -> u32 {
    3
}

fn default_rate_limit_register_window() -> u64 {
    3600 // 1 hour
}

fn default_rate_limit_forgot_max() -> u32 {
    3
}

fn default_rate_limit_forgot_window() -> u64 {
    3600 // 1 hour
}

fn default_rate_limit_accept_invite_max() -> u32 {
    10
}

fn default_rate_limit_accept_invite_window() -> u64 {
    3600 // 1 hour
}

fn default_rate_limit_lockout_threshold() -> u32 {
    10
}

fn default_rate_limit_lockout_duration() -> u64 {
    3600 // 1 hour
}

fn default_rate_limit_refresh_max() -> u32 {
    30
}

fn default_rate_limit_refresh_window() -> u64 {
    3600 // 1 hour
}

fn default_rate_limit_resend_max() -> u32 {
    3
}

fn default_rate_limit_resend_window() -> u64 {
    3600 // 1 hour
}

// Decision cache defaults
fn default_decision_cache_enabled() -> bool {
    true
}

fn default_decision_cache_ttl_seconds() -> u64 {
    15
}

fn default_decision_cache_max_entries() -> u64 {
    10_000
}

// Audit logging defaults
fn default_audit_log_enabled() -> bool {
    true
}

fn default_audit_log_retention_days() -> u32 {
    90
}

fn default_audit_log_batch_size() -> u32 {
    100
}

fn default_audit_log_flush_interval_ms() -> u64 {
    1000
}

// Cookie configuration defaults
fn default_cookie_secure() -> bool {
    // Default to true for security, should be overridden to false for local dev
    true
}

fn default_cookie_same_site() -> String {
    "Strict".to_string()
}

fn default_cookie_path() -> String {
    "/".to_string()
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, config::ConfigError> {
        // Load .env file if exists
        let _ = dotenvy::dotenv();

        let mut builder = config::Config::builder()
            .set_default("database_url", "")?
            .set_default("jwt_secret", "")?
            .set_default("jwt_expiration", 900)?
            .set_default("jwt_refresh_expiration", 604800)?
            .set_default("host", "0.0.0.0")?
            .set_default("port", 3000)?
            .set_default("casbin_model_path", "shared/auth/model.conf")?
            .set_default("max_connections", 10)?
            .set_default("invitation_base_url", "https://app.example.com")?
            .set_default("invitation_expiry_hours", 48)?
            .set_default("invitation_max_attempts", 5)?
            .set_default("invitation_max_per_admin_per_day", 10)?
            // Email verification defaults
            .set_default("verification_base_url", "https://app.example.com")?
            .set_default("verification_expiry_hours", 24)?
            // SMTP defaults
            .set_default("smtp_port", 587)?
            .set_default("smtp_from_email", "noreply@example.com")?
            .set_default("smtp_from_name", "Anthill")?
            .set_default("smtp_tls", true)?
            // Rate limiting defaults
            .set_default("rate_limit_enabled", true)?
            .set_default("rate_limit_login_max", 5)?
            .set_default("rate_limit_login_window", 900)?
            .set_default("rate_limit_register_max", 3)?
            .set_default("rate_limit_register_window", 3600)?
            .set_default("rate_limit_forgot_max", 3)?
            .set_default("rate_limit_forgot_window", 3600)?
            .set_default("rate_limit_accept_invite_max", 10)?
            .set_default("rate_limit_accept_invite_window", 3600)?
            .set_default("rate_limit_lockout_threshold", 10)?
            .set_default("rate_limit_lockout_duration", 3600)?
            .set_default("rate_limit_refresh_max", 30)?
            .set_default("rate_limit_refresh_window", 3600)?
            .set_default("rate_limit_resend_max", 3)?
            .set_default("rate_limit_resend_window", 3600)?
            .set_default("rate_limit_trust_proxy_headers", false)?
            .set_default("rate_limit_proxy_count", 0)?
            // Decision cache defaults
            .set_default("decision_cache_enabled", true)?
            .set_default("decision_cache_ttl_seconds", 15)?
            .set_default("decision_cache_max_entries", 10_000)?
            // Audit logging defaults
            .set_default("audit_log_enabled", true)?
            .set_default("audit_log_retention_days", 90)?
            .set_default("audit_log_batch_size", 100)?
            .set_default("audit_log_flush_interval_ms", 1000)?
            // Cookie configuration defaults
            .set_default("cookie_secure", true)?
            .set_default("cookie_same_site", "Strict")?
            .set_default("cookie_path", "/")?;

        // Add environment variables
        builder = builder.add_source(config::Environment::default());

        let config = builder.build()?;

        let deserialized = config.try_deserialize::<Config>()?;

        Ok(deserialized)
    }

    /// Get CORS allowed origins as a vector
    /// If cors_origins is None or empty, returns empty vec (accept all origins)
    pub fn get_cors_origins(&self) -> Vec<String> {
        self.cors_origins
            .as_ref()
            .map(|s| {
                s.split(',')
                    .map(|origin| origin.trim().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: String::new(),
            jwt_secret: String::new(),
            jwt_expiration: default_jwt_expiration(),
            jwt_refresh_expiration: default_jwt_refresh_expiration(),
            host: default_host(),
            port: default_port(),
            cors_origins: None,
            nats_url: None,
            redis_url: None,
            casbin_model_path: default_casbin_model_path(),
            max_connections: Some(10),
            invitation_base_url: default_invitation_base_url(),
            invitation_expiry_hours: default_invitation_expiry_hours(),
            invitation_max_attempts: default_invitation_max_attempts(),
            invitation_max_per_admin_per_day: default_invitation_max_per_admin_per_day(),
            verification_base_url: default_verification_base_url(),
            verification_expiry_hours: default_verification_expiry_hours(),
            smtp_host: None,
            smtp_port: default_smtp_port(),
            smtp_username: None,
            smtp_password: None,
            smtp_from_email: default_smtp_from_email(),
            smtp_from_name: default_smtp_from_name(),
            smtp_tls: default_smtp_tls(),
            rate_limit_enabled: default_rate_limit_enabled(),
            rate_limit_login_max: default_rate_limit_login_max(),
            rate_limit_login_window: default_rate_limit_login_window(),
            rate_limit_register_max: default_rate_limit_register_max(),
            rate_limit_register_window: default_rate_limit_register_window(),
            rate_limit_forgot_max: default_rate_limit_forgot_max(),
            rate_limit_forgot_window: default_rate_limit_forgot_window(),
            rate_limit_accept_invite_max: default_rate_limit_accept_invite_max(),
            rate_limit_accept_invite_window: default_rate_limit_accept_invite_window(),
            rate_limit_lockout_threshold: default_rate_limit_lockout_threshold(),
            rate_limit_lockout_duration: default_rate_limit_lockout_duration(),
            rate_limit_refresh_max: default_rate_limit_refresh_max(),
            rate_limit_refresh_window: default_rate_limit_refresh_window(),
            rate_limit_resend_max: default_rate_limit_resend_max(),
            rate_limit_resend_window: default_rate_limit_resend_window(),
            rate_limit_trust_proxy_headers: false,
            rate_limit_proxy_count: 0,
            rate_limit_trusted_ips: None,
            decision_cache_enabled: default_decision_cache_enabled(),
            decision_cache_ttl_seconds: default_decision_cache_ttl_seconds(),
            decision_cache_max_entries: default_decision_cache_max_entries(),
            audit_log_enabled: default_audit_log_enabled(),
            audit_log_retention_days: default_audit_log_retention_days(),
            audit_log_batch_size: default_audit_log_batch_size(),
            audit_log_flush_interval_ms: default_audit_log_flush_interval_ms(),
            cookie_domain: None,
            cookie_secure: default_cookie_secure(),
            cookie_same_site: default_cookie_same_site(),
            cookie_path: default_cookie_path(),
        }
    }
}
