use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use shared_error::AppError;
use uuid::Uuid;

/// JWT Claims for access and refresh tokens
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// User ID
    pub sub: Uuid,

    /// Tenant ID
    pub tenant_id: Uuid,

    /// User role
    pub role: String,

    /// Issued at (Unix timestamp)
    pub iat: i64,

    /// Expiration time (Unix timestamp)
    pub exp: i64,

    /// Token type: "access" or "refresh"
    pub token_type: String,

    /// Tenant authorization version at token issuance time
    /// Used for immediate-effect permission invalidation
    #[serde(default = "default_version")]
    pub tenant_v: i64,

    /// User authorization version at token issuance time
    /// Used for immediate-effect permission invalidation
    #[serde(default = "default_version")]
    pub user_v: i64,
}

/// Default version for backward compatibility with existing tokens
fn default_version() -> i64 {
    0
}

impl Claims {
    /// Create new access token claims
    pub fn new_access(user_id: Uuid, tenant_id: Uuid, role: String, expiration: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sub: user_id,
            tenant_id,
            role,
            iat: now,
            exp: now + expiration,
            token_type: "access".to_string(),
            tenant_v: 0,
            user_v: 0,
        }
    }

    /// Create new access token claims with authorization versions
    pub fn new_access_with_versions(
        user_id: Uuid,
        tenant_id: Uuid,
        role: String,
        expiration: i64,
        tenant_v: i64,
        user_v: i64,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sub: user_id,
            tenant_id,
            role,
            iat: now,
            exp: now + expiration,
            token_type: "access".to_string(),
            tenant_v,
            user_v,
        }
    }

    /// Create new refresh token claims
    pub fn new_refresh(user_id: Uuid, tenant_id: Uuid, role: String, expiration: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sub: user_id,
            tenant_id,
            role,
            iat: now,
            exp: now + expiration,
            token_type: "refresh".to_string(),
            tenant_v: 0,
            user_v: 0,
        }
    }

    /// Create new refresh token claims with authorization versions
    pub fn new_refresh_with_versions(
        user_id: Uuid,
        tenant_id: Uuid,
        role: String,
        expiration: i64,
        tenant_v: i64,
        user_v: i64,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sub: user_id,
            tenant_id,
            role,
            iat: now,
            exp: now + expiration,
            token_type: "refresh".to_string(),
            tenant_v,
            user_v,
        }
    }

    /// Check if this token has authorization versions set
    /// Tokens without versions (legacy) should skip version validation
    pub fn has_authz_versions(&self) -> bool {
        self.tenant_v > 0 || self.user_v > 0
    }
}

/// Encode claims into a JWT token
pub fn encode_jwt(claims: &Claims, secret: &str) -> Result<String, AppError> {
    let key = EncodingKey::from_secret(secret.as_bytes());
    encode(&Header::new(Algorithm::HS256), claims, &key)
        .map_err(|e| AppError::InternalError(format!("Failed to encode JWT: {}", e)))
}

/// Decode and validate a JWT token
pub fn decode_jwt(token: &str, secret: &str) -> Result<Claims, AppError> {
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);

    decode::<Claims>(token, &key, &validation)
        .map(|data| data.claims)
        .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_encode_decode() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let secret = "test_secret";

        let claims = Claims::new_access(user_id, tenant_id, "admin".to_string(), 3600);
        let token = encode_jwt(&claims, secret).unwrap();

        let decoded = decode_jwt(&token, secret).unwrap();
        assert_eq!(decoded.sub, user_id);
        assert_eq!(decoded.tenant_id, tenant_id);
        assert_eq!(decoded.role, "admin");
        assert_eq!(decoded.token_type, "access");
    }
}
