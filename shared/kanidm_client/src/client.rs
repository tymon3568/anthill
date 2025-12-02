use crate::config::KanidmConfig;
use crate::error::{KanidmError, Result};
use crate::types::{KanidmClaims, TokenResponse, UserInfo};
use async_trait::async_trait;
use base64::prelude::*;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

/// PKCE (Proof Key for Code Exchange) state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkceState {
    pub code_verifier: String,
    pub code_challenge: String,
    pub state: String,
}

impl PkceState {
    /// Generate new PKCE state with random values
    pub fn generate() -> Self {
        use sha2::{Digest, Sha256};

        // Generate random code verifier (43-128 chars)
        let code_verifier: String = (0..64)
            .map(|_| {
                let idx = rand::random::<usize>() % 62;
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"[idx] as char
            })
            .collect();

        // Generate code challenge (SHA256 hash of verifier)
        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let hash = hasher.finalize();
        let code_challenge = BASE64_URL_SAFE_NO_PAD.encode(hash);

        // Generate random state
        let state: String = (0..32)
            .map(|_| {
                let idx = rand::random::<usize>() % 62;
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"[idx] as char
            })
            .collect();

        Self {
            code_verifier,
            code_challenge,
            state,
        }
    }
}

/// Kanidm OAuth2 Client
#[async_trait]
pub trait KanidmOAuth2Client: Send + Sync {
    /// Generate authorization URL with PKCE
    fn authorization_url(&self, pkce: &PkceState) -> Result<Url>;

    /// Exchange authorization code for tokens
    async fn exchange_code(&self, code: &str, pkce: &PkceState) -> Result<TokenResponse>;

    /// Refresh access token
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse>;

    /// Validate JWT token and extract claims
    async fn validate_token(&self, token: &str) -> Result<KanidmClaims>;

    /// Get user info from Kanidm
    async fn get_userinfo(&self, access_token: &str) -> Result<UserInfo>;
}

/// Default implementation of Kanidm OAuth2 Client
#[derive(Clone)]
pub struct KanidmClient {
    config: Arc<KanidmConfig>,
    http_client: Client,
}

impl KanidmClient {
    /// Create new Kanidm client with configuration
    pub fn new(config: KanidmConfig) -> Result<Self> {
        config.validate()?;

        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| KanidmError::ConfigError(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self {
            config: Arc::new(config),
            http_client,
        })
    }

    /// Get JWKS (JSON Web Key Set) from Kanidm
    async fn fetch_jwks(&self) -> Result<jsonwebtoken::jwk::JwkSet> {
        let jwks_url = self.config.jwks_endpoint()?;

        let response = self
            .http_client
            .get(jwks_url)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| KanidmError::ApiError {
                status: e.status().map(|s| s.as_u16()).unwrap_or(500),
                message: format!("Failed to fetch JWKS: {}", e),
            })?;

        let jwks: jsonwebtoken::jwk::JwkSet = response.json().await?;
        Ok(jwks)
    }
}

#[async_trait]
impl KanidmOAuth2Client for KanidmClient {
    fn authorization_url(&self, pkce: &PkceState) -> Result<Url> {
        let mut url = self.config.authorization_endpoint()?;

        url.query_pairs_mut()
            .append_pair("client_id", &self.config.client_id)
            .append_pair("redirect_uri", &self.config.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", &self.config.scopes.join(" "))
            .append_pair("state", &pkce.state)
            .append_pair("code_challenge", &pkce.code_challenge)
            .append_pair("code_challenge_method", "S256");

        Ok(url)
    }

    async fn exchange_code(&self, code: &str, pkce: &PkceState) -> Result<TokenResponse> {
        let token_url = self.config.token_endpoint()?;

        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.config.redirect_uri),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("code_verifier", &pkce.code_verifier),
        ];

        let response = self
            .http_client
            .post(token_url)
            .form(&params)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| KanidmError::ApiError {
                status: e.status().map(|s| s.as_u16()).unwrap_or(500),
                message: format!("Token exchange failed: {}", e),
            })?;

        let token_response: TokenResponse = response.json().await?;
        Ok(token_response)
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let token_url = self.config.token_endpoint()?;

        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];

        let response = self
            .http_client
            .post(token_url)
            .form(&params)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| KanidmError::ApiError {
                status: e.status().map(|s| s.as_u16()).unwrap_or(500),
                message: format!("Token refresh failed: {}", e),
            })?;

        let token_response: TokenResponse = response.json().await?;
        Ok(token_response)
    }

    async fn validate_token(&self, token: &str) -> Result<KanidmClaims> {
        // Skip verification in dev mode (NOT RECOMMENDED FOR PRODUCTION!)
        if self.config.skip_jwt_verification {
            tracing::warn!("⚠️ Skipping JWT signature verification - DEV MODE ONLY!");

            let mut validation = Validation::new(Algorithm::RS256);
            validation.insecure_disable_signature_validation();
            validation.validate_exp = false;

            let token_data = decode::<KanidmClaims>(
                token,
                &DecodingKey::from_secret(&[]), // Dummy key
                &validation,
            )?;

            return Ok(token_data.claims);
        }

        // Production: Verify JWT signature with JWKS
        let header = decode_header(token)?;

        let kid = header
            .kid
            .ok_or_else(|| KanidmError::InvalidToken("Missing 'kid' in JWT header".to_string()))?;

        // Fetch JWKS from Kanidm
        let jwks = self.fetch_jwks().await?;

        // Find matching key
        let jwk = jwks
            .find(&kid)
            .ok_or_else(|| KanidmError::InvalidToken(format!("Key '{}' not found in JWKS", kid)))?;

        // Create decoding key from JWK
        let decoding_key = DecodingKey::from_jwk(jwk)?;

        // Setup validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&self.config.allowed_issuers);

        if let Some(ref aud) = self.config.expected_audience {
            validation.set_audience(&[aud]);
        }

        // Decode and validate token
        let token_data = decode::<KanidmClaims>(token, &decoding_key, &validation)?;

        // Additional checks
        if token_data.claims.is_expired() {
            return Err(KanidmError::TokenExpired);
        }

        Ok(token_data.claims)
    }

    async fn get_userinfo(&self, access_token: &str) -> Result<UserInfo> {
        let userinfo_url = self.config.userinfo_endpoint()?;

        let response = self
            .http_client
            .get(userinfo_url)
            .bearer_auth(access_token)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| KanidmError::ApiError {
                status: e.status().map(|s| s.as_u16()).unwrap_or(500),
                message: format!("UserInfo request failed: {}", e),
            })?;

        let userinfo: UserInfo = response.json().await?;
        Ok(userinfo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_generation() {
        let pkce = PkceState::generate();

        assert_eq!(pkce.code_verifier.len(), 64);
        assert!(!pkce.code_challenge.is_empty());
        assert_eq!(pkce.state.len(), 32);
    }

    #[tokio::test]
    async fn test_authorization_url_generation() {
        let config = KanidmConfig {
            kanidm_url: "http://localhost:8300".to_string(),
            client_id: "anthill".to_string(),
            client_secret: "secret".to_string(),
            redirect_uri: "http://localhost:5173/oauth/callback".to_string(),
            scopes: vec!["openid".to_string(), "profile".to_string()],
            skip_jwt_verification: false,
            allowed_issuers: vec![],
            expected_audience: None,
        };

        let client = KanidmClient::new(config).unwrap();
        let pkce = PkceState::generate();

        let auth_url = client.authorization_url(&pkce).unwrap();

        assert!(auth_url.as_str().contains("client_id=anthill"));
        assert!(auth_url.as_str().contains("code_challenge="));
        assert!(auth_url.as_str().contains("code_challenge_method=S256"));
    }
}
