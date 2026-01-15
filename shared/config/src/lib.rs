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

    /// Kanidm server URL (optional - for OAuth2/OIDC integration)
    pub kanidm_url: Option<String>,

    /// Kanidm OAuth2 client ID (optional)
    pub kanidm_client_id: Option<String>,

    /// Kanidm OAuth2 client secret (optional)
    pub kanidm_client_secret: Option<String>,

    /// Kanidm redirect URL (optional)
    pub kanidm_redirect_url: Option<String>,

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

    /// Trusted IPs that bypass rate limiting (comma-separated, optional)
    pub rate_limit_trusted_ips: Option<String>,
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
            .set_default("rate_limit_lockout_duration", 3600)?;

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
