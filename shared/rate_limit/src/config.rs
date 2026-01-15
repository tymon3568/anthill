//! Rate limiting configuration

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::str::FromStr;

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

    /// Trusted IPs/CIDRs that bypass rate limiting (comma-separated, e.g., "127.0.0.1,10.0.0.0/8")
    #[serde(default)]
    pub trusted_ips: Option<String>,

    /// Trust proxy headers (X-Forwarded-For, X-Real-IP) for IP extraction
    /// Only enable when running behind a trusted reverse proxy
    #[serde(default)]
    pub trust_proxy_headers: bool,

    /// Number of trusted proxy hops (for rightmost-trusted IP extraction)
    /// Default: 1 (trust the immediate proxy)
    #[serde(default = "default_proxy_count")]
    pub proxy_count: u32,
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
fn default_proxy_count() -> u32 {
    1
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
            trust_proxy_headers: false,
            proxy_count: default_proxy_count(),
        }
    }
}

/// Parsed CIDR network for efficient IP matching
#[derive(Debug, Clone)]
struct CidrNetwork {
    network: IpAddr,
    prefix_len: u8,
}

impl CidrNetwork {
    fn parse(cidr: &str) -> Option<Self> {
        let cidr = cidr.trim();
        if let Some((ip_str, prefix_str)) = cidr.split_once('/') {
            let network = IpAddr::from_str(ip_str).ok()?;
            let prefix_len: u8 = prefix_str.parse().ok()?;
            // Validate prefix length
            let max_prefix = match network {
                IpAddr::V4(_) => 32,
                IpAddr::V6(_) => 128,
            };
            if prefix_len > max_prefix {
                return None;
            }
            Some(Self {
                network,
                prefix_len,
            })
        } else {
            // Single IP address
            let network = IpAddr::from_str(cidr).ok()?;
            let prefix_len = match network {
                IpAddr::V4(_) => 32,
                IpAddr::V6(_) => 128,
            };
            Some(Self {
                network,
                prefix_len,
            })
        }
    }

    fn contains(&self, ip: IpAddr) -> bool {
        match (self.network, ip) {
            (IpAddr::V4(net), IpAddr::V4(addr)) => {
                if self.prefix_len == 0 {
                    return true;
                }
                let net_bits = u32::from(net);
                let addr_bits = u32::from(addr);
                let mask = !0u32 << (32 - self.prefix_len);
                (net_bits & mask) == (addr_bits & mask)
            },
            (IpAddr::V6(net), IpAddr::V6(addr)) => {
                if self.prefix_len == 0 {
                    return true;
                }
                let net_bits = u128::from(net);
                let addr_bits = u128::from(addr);
                let mask = !0u128 << (128 - self.prefix_len);
                (net_bits & mask) == (addr_bits & mask)
            },
            _ => false, // IPv4/IPv6 mismatch
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

    /// Check if an IP is trusted (supports CIDR notation like 10.0.0.0/8)
    pub fn is_trusted_ip(&self, ip: &str) -> bool {
        let trusted_str = match self.trusted_ips.as_deref() {
            Some(s) => s,
            None => return false,
        };

        let ip_addr = match IpAddr::from_str(ip) {
            Ok(addr) => addr,
            Err(_) => return false,
        };

        trusted_str.split(',').any(|cidr| {
            CidrNetwork::parse(cidr)
                .map(|network| network.contains(ip_addr))
                .unwrap_or(false)
        })
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
        assert!(!config.trust_proxy_headers);
        assert_eq!(config.proxy_count, 1);
    }

    #[test]
    fn test_trusted_ips_exact() {
        let mut config = RateLimitConfig::default();
        config.trusted_ips = Some("127.0.0.1, 10.0.0.1, 192.168.1.1".to_string());

        assert!(config.is_trusted_ip("127.0.0.1"));
        assert!(config.is_trusted_ip("10.0.0.1"));
        assert!(config.is_trusted_ip("192.168.1.1"));
        assert!(!config.is_trusted_ip("8.8.8.8"));
    }

    #[test]
    fn test_trusted_ips_cidr() {
        let mut config = RateLimitConfig::default();
        config.trusted_ips = Some("10.0.0.0/8, 192.168.0.0/16, 172.16.0.0/12".to_string());

        // 10.0.0.0/8
        assert!(config.is_trusted_ip("10.0.0.1"));
        assert!(config.is_trusted_ip("10.255.255.255"));
        assert!(!config.is_trusted_ip("11.0.0.1"));

        // 192.168.0.0/16
        assert!(config.is_trusted_ip("192.168.1.1"));
        assert!(config.is_trusted_ip("192.168.255.255"));
        assert!(!config.is_trusted_ip("192.169.1.1"));

        // 172.16.0.0/12
        assert!(config.is_trusted_ip("172.16.0.1"));
        assert!(config.is_trusted_ip("172.31.255.255"));
        assert!(!config.is_trusted_ip("172.32.0.1"));
    }

    #[test]
    fn test_trusted_ips_mixed() {
        let mut config = RateLimitConfig::default();
        config.trusted_ips = Some("127.0.0.1, 10.0.0.0/8".to_string());

        assert!(config.is_trusted_ip("127.0.0.1"));
        assert!(config.is_trusted_ip("10.1.2.3"));
        assert!(!config.is_trusted_ip("8.8.8.8"));
    }

    #[test]
    fn test_trusted_ips_invalid() {
        let mut config = RateLimitConfig::default();
        config.trusted_ips = Some("invalid, 10.0.0.0/8".to_string());

        assert!(!config.is_trusted_ip("invalid"));
        assert!(config.is_trusted_ip("10.1.2.3"));
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
