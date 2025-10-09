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

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, config::ConfigError> {
        // Load .env file if exists
        let _ = dotenvy::dotenv();
        
        let config = config::Config::builder()
            .add_source(config::Environment::default().separator("_"))
            .build()?;
        
        config.try_deserialize()
    }
}
