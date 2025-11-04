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

    /// Server port
    #[serde(default = "default_port")]
    pub port: u16,

    /// Kanidm server URL (optional - for OAuth2/OIDC integration)
    pub kanidm_url: Option<String>,

    /// Kanidm OAuth2 client ID (optional)
    pub kanidm_client_id: Option<String>,

    /// Kanidm OAuth2 client secret (optional)
    pub kanidm_client_secret: Option<String>,

    /// Kanidm redirect URL (optional)
    pub kanidm_redirect_url: Option<String>,

    /// Casbin model configuration file path
    #[serde(default = "default_casbin_model_path")]
    pub casbin_model_path: String,
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

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, config::ConfigError> {
        // Load .env file if exists
        let dotenv_result = dotenvy::dotenv();
        println!("DEBUG: dotenv result = {:?}", dotenv_result);

        println!("DEBUG: Loading config...");
        println!("DEBUG: DATABASE_URL = {:?}", std::env::var("DATABASE_URL"));
        println!("DEBUG: JWT_SECRET = {:?}", std::env::var("JWT_SECRET"));
        println!("DEBUG: Current dir = {:?}", std::env::current_dir());

        let mut builder = config::Config::builder()
            .set_default("database_url", "")?
            .set_default("jwt_secret", "")?
            .set_default("jwt_expiration", 900)?
            .set_default("jwt_refresh_expiration", 604800)?
            .set_default("host", "0.0.0.0")?
            .set_default("port", 3000)?
            .set_default("casbin_model_path", "shared/auth/model.conf")?;

        // Add environment variables
        builder = builder.add_source(config::Environment::default());

        let config = builder.build()?;

        println!("DEBUG: Config built ok");
        println!("DEBUG: Config keys: {:?}", config.cache);

        let deserialized = config.try_deserialize::<Config>()?;
        println!("DEBUG: Config deserialized: database_url = '{}'", deserialized.database_url);

        Ok(deserialized)
    }
}
