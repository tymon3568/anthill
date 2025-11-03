use crate::error::{KanidmError, Result};
use serde::{Deserialize, Serialize};
use url::Url;

/// Kanidm Client Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanidmConfig {
    /// Kanidm server URL (e.g., http://localhost:8300)
    pub kanidm_url: String,

    /// OAuth2 client ID
    pub client_id: String,

    /// OAuth2 client secret
    pub client_secret: String,

    /// OAuth2 redirect URI for callback
    pub redirect_uri: String,

    /// OAuth2 scopes to request
    pub scopes: Vec<String>,

    /// JWT validation: skip signature verification (dev only!)
    #[serde(default)]
    pub skip_jwt_verification: bool,

    /// JWT validation: allowed issuers
    #[serde(default)]
    pub allowed_issuers: Vec<String>,

    /// JWT validation: expected audience
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_audience: Option<String>,
}

impl KanidmConfig {
    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let kanidm_url = std::env::var("KANIDM_URL")
            .map_err(|_| KanidmError::ConfigError("KANIDM_URL not set".to_string()))?;

        let client_id = std::env::var("KANIDM_OAUTH2_CLIENT_ID")
            .map_err(|_| KanidmError::ConfigError("KANIDM_OAUTH2_CLIENT_ID not set".to_string()))?;

        let client_secret = std::env::var("KANIDM_OAUTH2_CLIENT_SECRET").map_err(|_| {
            KanidmError::ConfigError("KANIDM_OAUTH2_CLIENT_SECRET not set".to_string())
        })?;

        let redirect_uri = std::env::var("OAUTH2_REDIRECT_URI")
            .map_err(|_| KanidmError::ConfigError("OAUTH2_REDIRECT_URI not set".to_string()))?;

        let scopes = std::env::var("OAUTH2_SCOPES")
            .unwrap_or_else(|_| "openid,profile,email,groups".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let skip_jwt_verification = std::env::var("KANIDM_SKIP_JWT_VERIFICATION")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let allowed_issuers = vec![kanidm_url.clone()];

        let expected_audience = Some(client_id.clone());

        Ok(Self {
            kanidm_url,
            client_id,
            client_secret,
            redirect_uri,
            scopes,
            skip_jwt_verification,
            allowed_issuers,
            expected_audience,
        })
    }

    /// Get authorization endpoint URL
    pub fn authorization_endpoint(&self) -> Result<Url> {
        let base = Url::parse(&self.kanidm_url)?;
        // Kanidm uses /oauth2/authorise (not /ui/oauth2)
        base.join("/oauth2/authorise").map_err(Into::into)
    }

    /// Get token endpoint URL
    pub fn token_endpoint(&self) -> Result<Url> {
        let base = Url::parse(&self.kanidm_url)?;
        base.join("/oauth2/token").map_err(Into::into)
    }

    /// Get userinfo endpoint URL
    pub fn userinfo_endpoint(&self) -> Result<Url> {
        let base = Url::parse(&self.kanidm_url)?;
        // Kanidm format: /oauth2/openid/{client_id}/userinfo
        let path = format!("/oauth2/openid/{}/userinfo", self.client_id);
        base.join(&path).map_err(Into::into)
    }

    /// Get JWKS endpoint URL
    pub fn jwks_endpoint(&self) -> Result<Url> {
        let base = Url::parse(&self.kanidm_url)?;
        // Kanidm format: /oauth2/openid/{client_id}/public_key.jwk
        let path = format!("/oauth2/openid/{}/public_key.jwk", self.client_id);
        base.join(&path).map_err(Into::into)
    }

    /// Get OpenID Connect discovery URL
    pub fn discovery_endpoint(&self) -> Result<Url> {
        let base = Url::parse(&self.kanidm_url)?;
        let path = format!(
            "/oauth2/openid/{}/.well-known/openid-configuration",
            self.client_id
        );
        base.join(&path).map_err(Into::into)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate URLs
        Url::parse(&self.kanidm_url)
            .map_err(|_| KanidmError::ConfigError("Invalid KANIDM_URL".to_string()))?;

        Url::parse(&self.redirect_uri)
            .map_err(|_| KanidmError::ConfigError("Invalid OAUTH2_REDIRECT_URI".to_string()))?;

        // Validate client credentials
        if self.client_id.is_empty() {
            return Err(KanidmError::ConfigError(
                "client_id cannot be empty".to_string(),
            ));
        }

        if self.client_secret.is_empty() {
            return Err(KanidmError::ConfigError(
                "client_secret cannot be empty".to_string(),
            ));
        }

        // Validate scopes
        if self.scopes.is_empty() {
            return Err(KanidmError::ConfigError(
                "At least one scope is required".to_string(),
            ));
        }

        // Warn if JWT verification is skipped
        if self.skip_jwt_verification {
            tracing::warn!("⚠️ JWT signature verification is DISABLED - only use in development!");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config = KanidmConfig {
            kanidm_url: "http://localhost:8300".to_string(),
            client_id: "anthill".to_string(),
            client_secret: "secret".to_string(),
            redirect_uri: "http://localhost:5173/oauth/callback".to_string(),
            scopes: vec!["openid".to_string(), "profile".to_string()],
            skip_jwt_verification: false,
            allowed_issuers: vec!["http://localhost:8300".to_string()],
            expected_audience: Some("anthill".to_string()),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_endpoint_urls() {
        let config = KanidmConfig {
            kanidm_url: "http://localhost:8300".to_string(),
            client_id: "anthill".to_string(),
            client_secret: "secret".to_string(),
            redirect_uri: "http://localhost:5173/oauth/callback".to_string(),
            scopes: vec!["openid".to_string()],
            skip_jwt_verification: false,
            allowed_issuers: vec![],
            expected_audience: None,
        };

        assert_eq!(
            config.authorization_endpoint().unwrap().as_str(),
            "http://localhost:8300/oauth2/authorise"
        );

        assert_eq!(
            config.token_endpoint().unwrap().as_str(),
            "http://localhost:8300/oauth2/token"
        );

        assert_eq!(
            config.jwks_endpoint().unwrap().as_str(),
            "http://localhost:8300/oauth2/openid/anthill/public_key.jwk"
        );
    }
}
