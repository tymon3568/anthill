use axum::{
    extract::{ConnectInfo, FromRequestParts},
    http::{request::Parts, HeaderMap},
};
use std::net::SocketAddr;

/// Extract client IP address from request
/// 
/// Tries in order:
/// 1. X-Forwarded-For header (if behind proxy)
/// 2. X-Real-IP header
/// 3. ConnectInfo socket address
pub fn extract_client_ip(
    headers: &HeaderMap,
    connect_info: Option<ConnectInfo<SocketAddr>>,
) -> Option<String> {
    // Try X-Forwarded-For first (common when behind proxy/load balancer)
    if let Some(forwarded_for) = headers.get("x-forwarded-for") {
        if let Ok(value) = forwarded_for.to_str() {
            // X-Forwarded-For can be comma-separated: "client, proxy1, proxy2"
            // Take the first (leftmost) IP which is the original client
            if let Some(client_ip) = value.split(',').next() {
                return Some(client_ip.trim().to_string());
            }
        }
    }
    
    // Try X-Real-IP (set by some proxies)
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(value) = real_ip.to_str() {
            return Some(value.to_string());
        }
    }
    
    // Fallback to direct connection info
    connect_info.map(|info| info.0.ip().to_string())
}

/// Extract User-Agent from request headers
pub fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

/// Custom extractor for client metadata (IP and User-Agent)
/// 
/// This can be used directly in handler parameters.
pub struct ClientInfo {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl<S> FromRequestParts<S> for ClientInfo
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;
    
    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Extract ConnectInfo if available
        let connect_info = ConnectInfo::<SocketAddr>::from_request_parts(parts, state)
            .await
            .ok();
        
        let ip_address = extract_client_ip(&parts.headers, connect_info);
        let user_agent = extract_user_agent(&parts.headers);
        
        Ok(ClientInfo {
            ip_address,
            user_agent,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use std::net::{IpAddr, Ipv4Addr};
    
    #[test]
    fn test_extract_ip_from_x_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.1, 198.51.100.1"),
        );
        
        let ip = extract_client_ip(&headers, None);
        assert_eq!(ip, Some("203.0.113.1".to_string()));
    }
    
    #[test]
    fn test_extract_ip_from_x_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", HeaderValue::from_static("203.0.113.1"));
        
        let ip = extract_client_ip(&headers, None);
        assert_eq!(ip, Some("203.0.113.1".to_string()));
    }
    
    #[test]
    fn test_extract_ip_from_connect_info() {
        let headers = HeaderMap::new();
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let connect_info = Some(ConnectInfo(addr));
        
        let ip = extract_client_ip(&headers, connect_info);
        assert_eq!(ip, Some("127.0.0.1".to_string()));
    }
    
    #[test]
    fn test_x_forwarded_for_priority() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.1"));
        headers.insert("x-real-ip", HeaderValue::from_static("198.51.100.1"));
        
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let connect_info = Some(ConnectInfo(addr));
        
        let ip = extract_client_ip(&headers, connect_info);
        // X-Forwarded-For should have priority
        assert_eq!(ip, Some("203.0.113.1".to_string()));
    }
    
    #[test]
    fn test_extract_user_agent() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "user-agent",
            HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64)"),
        );
        
        let ua = extract_user_agent(&headers);
        assert_eq!(
            ua,
            Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64)".to_string())
        );
    }
    
    #[test]
    fn test_extract_user_agent_missing() {
        let headers = HeaderMap::new();
        let ua = extract_user_agent(&headers);
        assert_eq!(ua, None);
    }
}
