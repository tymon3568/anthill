use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Kanidm JWT Claims (OIDC-compliant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanidmClaims {
    /// Subject - Kanidm user UUID
    pub sub: String,

    /// Email address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Preferred username
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_username: Option<String>,

    /// User's full name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Groups the user belongs to
    #[serde(default)]
    pub groups: Vec<String>,

    /// Issued at timestamp
    pub iat: i64,

    /// Expiration timestamp
    pub exp: i64,

    /// Issuer
    pub iss: String,

    /// Audience
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,

    /// Nonce (for replay protection)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
}

impl KanidmClaims {
    /// Extract Kanidm user UUID from subject claim
    pub fn user_uuid(&self) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(&self.sub)
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.exp < now
    }

    /// Check if user belongs to a specific group
    pub fn has_group(&self, group_name: &str) -> bool {
        self.groups.iter().any(|g| g == group_name)
    }

    /// Extract tenant groups (groups starting with "tenant_")
    pub fn tenant_groups(&self) -> Vec<String> {
        self.groups
            .iter()
            .filter(|g| g.starts_with("tenant_"))
            .cloned()
            .collect()
    }

    /// Extract tenant IDs from group names
    /// e.g., "tenant_acme_users" -> "acme"
    pub fn tenant_slugs(&self) -> Vec<String> {
        self.tenant_groups()
            .iter()
            .filter_map(|g| {
                g.strip_prefix("tenant_")
                    .and_then(|s| s.split('_').next())
                    .map(String::from)
            })
            .collect()
    }
}

/// OAuth2 Token Response from Kanidm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// User Info from Kanidm UserInfo endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub sub: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_groups_extraction() {
        let claims = KanidmClaims {
            sub: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            email: Some("alice@example.com".to_string()),
            preferred_username: Some("alice".to_string()),
            name: Some("Alice Admin".to_string()),
            groups: vec![
                "tenant_acme_users".to_string(),
                "tenant_acme_admins".to_string(),
                "tenant_globex_users".to_string(),
                "system_admins".to_string(),
            ],
            iat: 1234567890,
            exp: 1234571490,
            iss: "http://localhost:8300".to_string(),
            aud: Some("anthill".to_string()),
            nonce: None,
        };

        let tenant_groups = claims.tenant_groups();
        assert_eq!(tenant_groups.len(), 3);
        assert!(tenant_groups.contains(&"tenant_acme_users".to_string()));

        let tenant_slugs = claims.tenant_slugs();
        assert!(tenant_slugs.contains(&"acme".to_string()));
        assert!(tenant_slugs.contains(&"globex".to_string()));
    }

    #[test]
    fn test_has_group() {
        let claims = KanidmClaims {
            sub: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            email: None,
            preferred_username: None,
            name: None,
            groups: vec!["tenant_acme_admins".to_string()],
            iat: 1234567890,
            exp: 1234571490,
            iss: "http://localhost:8300".to_string(),
            aud: None,
            nonce: None,
        };

        assert!(claims.has_group("tenant_acme_admins"));
        assert!(!claims.has_group("tenant_globex_users"));
    }
}
