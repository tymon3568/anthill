
//! Rate limiting configuration

use serde::{Deserialize, Serialize};

/// Rate limit configuration for different endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Redis URL for distributed rate limiting (optional, falls back to in-memory)
    pub redis_url: Option<String>,

    /// Login endpoint limits
    #[serde(default = "default_login_max_attempts")]
    pub login_max_attempts: u32,
    #[serde(default = "default_login_window_seconds")]
    pub login_window_seconds: u64,

    /// Register endpoint limits
    #[serde(default = "default_register_max_attempts")]
    pub register_max_attempts: u32,
    #[serde(default = "default_register_window_seconds")]
    pub register_window_seconds: u64,

    /// Forgot password limits
    #[serde(default = "default_forgot_password_max")]
    pub forgot_password_max: u32,
    #[serde(default = "default_forgot_password_window")]
    pub forgot_password_window: u64,

    /// Resend verification limits
    #[serde(default = "default_resend_verification_max")]
    pub resend_verification_max: u32,
    #[serde(default = "default_resend_verification_window")]
    pub resend_verification_window: u64,

    /// Refresh token limits
    #[serde(default = "default_refresh_max")]
    pub refresh_max: u32,
    #[serde(default = "default_refresh_window")]
    pub refresh_window: u64,

    /// Accept invitation limits (per IP)
    #[serde(default = "default_accept_invite_max")]
    pub accept_invite_max: u32,
    #[serde(default = "default_accept_invite_window")]
    pub accept_invite_window: u64,

    /// Account lockout settings
    #[serde(default = "default_lockout_threshold")]
    pub lockout_threshold: u32,
    #[serde(default = "default_lockout_duration_seconds")]
    pub lockout_duration_seconds: u64,

    /// Global API rate limit (requests per second)
    #[serde(default = "default_global_requests_per_second")]
    pub global_requests_per_second: u32,

    /// Enable rate limiting (can be disabled for testing)
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Trusted IPs that bypass rate limiting (comma-separated)
    #[serde(default)]
    pub trusted_ips: Option<String>,
}

fn default_login_max_attempts() -> u32 {
    5
}
fn default_login_window_seconds() -> u64 {
    900 // 15 minutes
}
fn default_register_max_attempts() -> u32 {
    3
}
fn default_register_window_seconds() -> u64 {
    3600 // 1 hour
}
fn default_forgot_password_max() -> u32 {
    3
}
fn default_forgot_password_window() -> u64 {
    3600 // 1 hour
}
fn default_resend_verification_max() -> u32 {
    3
}
fn default_resend_verification_window() -> u64 {
    3600 // 1 hour
}
fn default_refresh_max() -> u32 {
    30
}
fn default_refresh_window() -> u64 {
    3600 // 1 hour
}
fn default_accept_invite_max() -> u32 {
    10
}
fn default_accept_invite_window() -> u64 {
    3600 // 1 hour
}
fn default_lockout_threshold() -> u32 {
    10
}
fn default_lockout_duration_seconds() -> u64 {
    3600 // 1 hour
}
fn default_global_requests_per_second() -> u32 {
    100
}
fn default_enabled() -> bool {
    true
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            redis_url: None,
            login_max_attempts: default_login_max_attempts(),
            login_window_seconds: default_login_window_seconds(),
            register_max_attempts: default_register_max_attempts(),
            register_window_seconds: default_register_window_seconds(),
            forgot_password_max: default_forgot_password_max(),
            forgot_password_window: default_forgot_password_window(),
            resend_verification_max: default_resend_verification_max(),
            resend_verification_window: default_resend_verification_window(),
            refresh_max: default_refresh_max(),
            refresh_window: default_refresh_window(),
            accept_invite_max: default_accept_invite_max(),
            accept_invite_window: default_accept_invite_window(),
            lockout_threshold: default_lockout_threshold(),
            lockout_duration_seconds: default_lockout_duration_seconds(),
            global_requests_per_second: default_global_requests_per_second(),
            enabled: default_enabled(),
            trusted_ips: None,
        }
    }
}

impl RateLimitConfig {
    /// Get trusted IPs as a vector
    pub fn get_trusted_ips(&self) -> Vec<String> {
        self.trusted_ips
            .as_ref()
            .map(|s| {
                s.split(',')
                    .map(|ip| ip.trim().to_string())
                    .filter(|ip| !ip.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if an IP is trusted
    pub fn is_trusted_ip(&self, ip: &str) -> bool {
        self.get_trusted_ips().contains(&ip.to_string())
    }
}

/// Endpoint-specific rate limit rule
#[derive(Debug, Clone)]
pub struct RateLimitRule {
    /// Maximum number of requests allowed
    pub max_requests: u32,
    /// Time window in seconds
    pub window_seconds: u64,
    /// Key prefix for this rule
    pub key_prefix: String,
}

impl RateLimitRule {
    pub fn new(max_requests: u32, window_seconds: u64, key_prefix: impl Into<String>) -> Self {
        Self {
            max_requests,
            window_seconds,
            key_prefix: key_prefix.into(),
        }
    }
}

/// Predefined rate limit rules for common endpoints
#[derive(Debug, Clone)]
pub struct EndpointRules {
    pub login: RateLimitRule,
    pub register: RateLimitRule,
    pub forgot_password: RateLimitRule,
    pub resend_verification: RateLimitRule,
    pub refresh: RateLimitRule,
    pub accept_invite: RateLimitRule,
}

impl From<&RateLimitConfig> for EndpointRules {
    fn from(config: &RateLimitConfig) -> Self {
        Self {
            login: RateLimitRule::new(
                config.login_max_attempts,
                config.login_window_seconds,
                "rate_limit:login:ip",
            ),
            register: RateLimitRule::new(
                config.register_max_attempts,
                config.register_window_seconds,
                "rate_limit:register:ip",
            ),
            forgot_password: RateLimitRule::new(
                config.forgot_password_max,
                config.forgot_password_window,
                "rate_limit:forgot:email",
            ),
            resend_verification: RateLimitRule::new(
                config.resend_verification_max,
                config.resend_verification_window,
                "rate_limit:resend:user",
            ),
            refresh: RateLimitRule::new(
                config.refresh_max,
                config.refresh_window,
                "rate_limit:refresh:user",
            ),
            accept_invite: RateLimitRule::new(
                config.accept_invite_max,
                config.accept_invite_window,
                "rate_limit:accept_invite:ip",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RateLimitConfig::default();
        assert_eq!(config.login_max_attempts, 5);
        assert_eq!(config.login_window_seconds, 900);
        assert_eq!(config.register_max_attempts, 3);
        assert!(config.enabled);
    }

    #[test]
    fn test_trusted_ips() {
        let mut config = RateLimitConfig::default();
        config.trusted_ips = Some("127.0.0.1, 10.0.0.1, 192.168.1.1".to_string());

        let ips = config.get_trusted_ips();
        assert_eq!(ips.len(), 3);
        assert!(config.is_trusted_ip("127.0.0.1"));
        assert!(config.is_trusted_ip("10.0.0.1"));
        assert!(!config.is_trusted_ip("8.8.8.8"));
    }

    #[test]
    fn test_endpoint_rules_from_config() {
        let config = RateLimitConfig::default();
        let rules = EndpointRules::from(&config);

        assert_eq!(rules.login.max_requests, 5);
        assert_eq!(rules.login.window_seconds, 900);
        assert_eq!(rules.login.key_prefix, "rate_limit:login:ip");
    }
}
