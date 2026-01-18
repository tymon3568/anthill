//! Cookie helper for setting httpOnly authentication cookies
//!
//! This module provides utilities for setting secure httpOnly cookies
//! for authentication tokens, preventing JavaScript access and XSS attacks.

use axum::http::{header::SET_COOKIE, HeaderMap, HeaderValue};
use shared_config::Config;

/// Cookie configuration for authentication tokens
pub struct CookieConfig<'a> {
    pub config: &'a Config,
    pub access_token_max_age: i64,
    pub refresh_token_max_age: i64,
}

impl<'a> CookieConfig<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self {
            config,
            access_token_max_age: config.jwt_expiration,
            refresh_token_max_age: config.jwt_refresh_expiration,
        }
    }
}

/// Build a Set-Cookie header value for an authentication cookie
fn build_cookie_header(
    name: &str,
    value: &str,
    max_age: i64,
    config: &Config,
) -> Result<HeaderValue, axum::http::header::InvalidHeaderValue> {
    let mut cookie = format!(
        "{}={}; Path={}; Max-Age={}; HttpOnly; SameSite={}",
        name, value, config.cookie_path, max_age, config.cookie_same_site
    );

    // Add Secure flag if configured
    if config.cookie_secure {
        cookie.push_str("; Secure");
    }

    // Add Domain if configured
    if let Some(ref domain) = config.cookie_domain {
        cookie.push_str(&format!("; Domain={}", domain));
    }

    HeaderValue::from_str(&cookie)
}

/// Build a Set-Cookie header to clear a cookie (set max-age to 0)
fn build_clear_cookie_header(
    name: &str,
    config: &Config,
) -> Result<HeaderValue, axum::http::header::InvalidHeaderValue> {
    let mut cookie = format!(
        "{}=; Path={}; Max-Age=0; HttpOnly; SameSite={}",
        name, config.cookie_path, config.cookie_same_site
    );

    // Add Secure flag if configured
    if config.cookie_secure {
        cookie.push_str("; Secure");
    }

    // Add Domain if configured
    if let Some(ref domain) = config.cookie_domain {
        cookie.push_str(&format!("; Domain={}", domain));
    }

    HeaderValue::from_str(&cookie)
}

/// Set authentication cookies in the response headers
///
/// Sets both access_token and refresh_token as httpOnly cookies.
/// These cookies cannot be accessed by JavaScript, protecting against XSS attacks.
pub fn set_auth_cookies(
    headers: &mut HeaderMap,
    access_token: &str,
    refresh_token: &str,
    cookie_config: &CookieConfig,
) -> Result<(), String> {
    // Set access_token cookie
    let access_cookie = build_cookie_header(
        "access_token",
        access_token,
        cookie_config.access_token_max_age,
        cookie_config.config,
    )
    .map_err(|e| format!("Failed to build access_token cookie: {}", e))?;

    headers.append(SET_COOKIE, access_cookie);

    // Set refresh_token cookie
    let refresh_cookie = build_cookie_header(
        "refresh_token",
        refresh_token,
        cookie_config.refresh_token_max_age,
        cookie_config.config,
    )
    .map_err(|e| format!("Failed to build refresh_token cookie: {}", e))?;

    headers.append(SET_COOKIE, refresh_cookie);

    Ok(())
}

/// Clear authentication cookies from the response headers
///
/// Sets both access_token and refresh_token cookies with max-age=0 to clear them.
pub fn clear_auth_cookies(headers: &mut HeaderMap, config: &Config) -> Result<(), String> {
    // Clear access_token cookie
    let access_cookie = build_clear_cookie_header("access_token", config)
        .map_err(|e| format!("Failed to build clear access_token cookie: {}", e))?;

    headers.append(SET_COOKIE, access_cookie);

    // Clear refresh_token cookie
    let refresh_cookie = build_clear_cookie_header("refresh_token", config)
        .map_err(|e| format!("Failed to build clear refresh_token cookie: {}", e))?;

    headers.append(SET_COOKIE, refresh_cookie);

    Ok(())
}

/// Extract a cookie value from the Cookie header
pub fn get_cookie_value(headers: &axum::http::HeaderMap, cookie_name: &str) -> Option<String> {
    headers
        .get(axum::http::header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let mut parts = cookie.trim().splitn(2, '=');
                let name = parts.next()?;
                let value = parts.next()?;
                if name == cookie_name {
                    Some(value.to_string())
                } else {
                    None
                }
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config {
            cookie_secure: false,
            cookie_same_site: "Strict".to_string(),
            cookie_path: "/".to_string(),
            cookie_domain: None,
            jwt_expiration: 900,
            jwt_refresh_expiration: 604800,
            ..Default::default()
        }
    }

    #[test]
    fn test_set_auth_cookies() {
        let config = test_config();
        let cookie_config = CookieConfig::new(&config);
        let mut headers = HeaderMap::new();

        set_auth_cookies(&mut headers, "test_access", "test_refresh", &cookie_config).unwrap();

        let cookies: Vec<_> = headers.get_all(SET_COOKIE).iter().collect();
        assert_eq!(cookies.len(), 2);

        let access_cookie = cookies[0].to_str().unwrap();
        assert!(access_cookie.contains("access_token=test_access"));
        assert!(access_cookie.contains("HttpOnly"));
        assert!(access_cookie.contains("SameSite=Strict"));
        assert!(access_cookie.contains("Max-Age=900"));

        let refresh_cookie = cookies[1].to_str().unwrap();
        assert!(refresh_cookie.contains("refresh_token=test_refresh"));
        assert!(refresh_cookie.contains("HttpOnly"));
        assert!(refresh_cookie.contains("Max-Age=604800"));
    }

    #[test]
    fn test_clear_auth_cookies() {
        let config = test_config();
        let mut headers = HeaderMap::new();

        clear_auth_cookies(&mut headers, &config).unwrap();

        let cookies: Vec<_> = headers.get_all(SET_COOKIE).iter().collect();
        assert_eq!(cookies.len(), 2);

        for cookie in cookies {
            let cookie_str = cookie.to_str().unwrap();
            assert!(cookie_str.contains("Max-Age=0"));
            assert!(cookie_str.contains("HttpOnly"));
        }
    }

    #[test]
    fn test_secure_flag_when_enabled() {
        let mut config = test_config();
        config.cookie_secure = true;
        let cookie_config = CookieConfig::new(&config);
        let mut headers = HeaderMap::new();

        set_auth_cookies(&mut headers, "test", "test", &cookie_config).unwrap();

        let cookie = headers.get(SET_COOKIE).unwrap().to_str().unwrap();
        assert!(cookie.contains("; Secure"));
    }

    #[test]
    fn test_domain_when_configured() {
        let mut config = test_config();
        config.cookie_domain = Some("example.com".to_string());
        let cookie_config = CookieConfig::new(&config);
        let mut headers = HeaderMap::new();

        set_auth_cookies(&mut headers, "test", "test", &cookie_config).unwrap();

        let cookie = headers.get(SET_COOKIE).unwrap().to_str().unwrap();
        assert!(cookie.contains("; Domain=example.com"));
    }

    #[test]
    fn test_get_cookie_value() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            HeaderValue::from_static("access_token=abc123; refresh_token=xyz789"),
        );

        assert_eq!(get_cookie_value(&headers, "access_token"), Some("abc123".to_string()));
        assert_eq!(get_cookie_value(&headers, "refresh_token"), Some("xyz789".to_string()));
        assert_eq!(get_cookie_value(&headers, "nonexistent"), None);
    }
}
