use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct KanidmAdminClient {
    base_url: String,
    client: Client,
    session_token: Option<String>,
}

#[derive(Debug, Serialize)]
struct AuthInitRequest {
    step: AuthStep,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum AuthStep {
    Init { init: String },
    Begin { begin: String },
    Credential { cred: CredentialType },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum CredentialType {
    Password(String),
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    sessionid: String,
    state: AuthState,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AuthState {
    Choose(Vec<String>),
    Continue(Vec<String>),
    Success(String), // Contains JWT token
    Denied(String),
}

#[derive(Debug, Serialize)]
pub struct CreateOAuth2Request {
    pub name: String,
    pub displayname: String,
    pub origin: String,
}

#[derive(Debug, Serialize)]
pub struct AddRedirectUrlRequest {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateScopeMapRequest {
    pub group: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuth2ClientResponse {
    pub name: String,
    pub displayname: String,
    pub origin: String,
    pub oauth2_rs_basic_secret: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateGroupRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreatePersonRequest {
    pub name: String,
    pub displayname: String,
    pub mail: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct AddMembersRequest {
    pub members: Vec<String>,
}

impl KanidmAdminClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        let client = Client::builder()
            .danger_accept_invalid_certs(true) // Development only
            .timeout(DEFAULT_TIMEOUT)
            .build()?;

        Ok(Self {
            base_url: base_url.into(),
            client,
            session_token: None,
        })
    }

    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<()> {
        info!("🔐 Authenticating as {}", username);

        let url = format!("{}/v1/auth", self.base_url);

        // Step 1: Init authentication
        let init_req = AuthInitRequest {
            step: AuthStep::Init {
                init: username.to_string(),
            },
        };

        let response = self
            .client
            .post(&url)
            .json(&init_req)
            .send()
            .await
            .context("Failed to init authentication")?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Auth init failed: {}", text);
        }

        // Extract session token from response header (JWT) or cookie
        debug!("Response headers:");
        for (name, value) in response.headers() {
            debug!("  {}: {:?}", name, value);
        }

        let session_token = if let Some(header_value) = response.headers().get("x-kanidm-auth-session-id") {
            debug!("Found session token in header");
            header_value.to_str().context("Invalid session token header")?.to_string()
        } else if let Some(cookie_header) = response.headers().get("set-cookie") {
            debug!("No header found, trying cookie");
            let cookie_str = cookie_header.to_str().context("Invalid cookie header")?;
            // Parse cookie like: auth-session-id=eyJ...; HttpOnly; SameSite=Strict; Secure
            if let Some(token_part) = cookie_str.split(';').next() {
                if let Some(token) = token_part.strip_prefix("auth-session-id=") {
                    debug!("Found session token in cookie");
                    token.to_string()
                } else {
                    anyhow::bail!("Cookie doesn't contain auth-session-id");
                }
            } else {
                anyhow::bail!("Invalid cookie format");
            }
        } else {
            anyhow::bail!("No session token in header or cookie");
        };

        debug!("Got session token from header: {}...", &session_token[..30]);

                // Step 2: Begin with Password mechanism
        let begin_req = AuthInitRequest {
            step: AuthStep::Begin {
                begin: "password".to_string(),
            },
        };

        let response = self
            .client
            .post(&url)
            .header("X-KANIDM-AUTH-SESSION-ID", &session_token)
            .json(&begin_req)
            .send()
            .await
            .context("Failed to begin password auth")?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Auth begin failed: {}", text);
        }

        // Step 3: Submit password credential
        let cred_req = AuthInitRequest {
            step: AuthStep::Credential {
                cred: CredentialType::Password(password.to_string()),
            },
        };

        let response = self
            .client
            .post(&url)
            .header("X-KANIDM-AUTH-SESSION-ID", &session_token)
            .json(&cred_req)
            .send()
            .await
            .context("Failed to submit credentials")?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Auth credential submit failed: {}", text);
        }

        let auth_response: AuthResponse = response
            .json()
            .await
            .context("Failed to parse auth response")?;

        match auth_response.state {
            AuthState::Success(jwt_token) => {
                self.session_token = Some(jwt_token);
                info!("✅ Authentication successful");
                Ok(())
            }
            AuthState::Denied(reason) => {
                anyhow::bail!("Authentication denied: {}", reason)
            }
            other => {
                anyhow::bail!("Unexpected auth state: {:?}", other)
            }
        }
    }

    fn auth_header(&self) -> Result<String> {
        self.session_token
            .as_ref()
            .map(|token| format!("Bearer {}", token))
            .context("Not authenticated")
    }

    pub async fn create_oauth2_client(&self, req: CreateOAuth2Request) -> Result<bool> {
        info!("🔧 Creating OAuth2 client: {}", req.name);

        // Try the regular /v1/oauth2 endpoint first (from Context7 docs)
        let url = format!("{}/v1/oauth2", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header()?)
            .json(&req)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK | StatusCode::CREATED => {
                info!("✅ OAuth2 client '{}' created", req.name);
                Ok(true)
            }
            StatusCode::CONFLICT => {
                warn!("⚠️  OAuth2 client '{}' already exists", req.name);
                Ok(false)
            }
            status => {
                let text = response.text().await.unwrap_or_default();
                warn!("Regular endpoint failed ({}), trying _basic endpoint: {}", status, text);

                // Fallback to _basic endpoint with ProtoEntry format
                let basic_url = format!("{}/v1/oauth2/_basic", self.base_url);
                let proto_entry = serde_json::json!({
                    "attrs": {
                        "oauth2_rs_name": [&req.name],
                        "displayname": [&req.displayname],
                        "oauth2_rs_origin": [&req.origin]
                    }
                });

                let basic_response = self
                    .client
                    .post(&basic_url)
                    .header("Authorization", self.auth_header()?)
                    .json(&proto_entry)
                    .send()
                    .await?;

                match basic_response.status() {
                    StatusCode::OK | StatusCode::CREATED => {
                        info!("✅ OAuth2 client '{}' created via _basic endpoint", req.name);
                        Ok(true)
                    }
                    StatusCode::CONFLICT => {
                        warn!("⚠️  OAuth2 client '{}' already exists", req.name);
                        Ok(false)
                    }
                    basic_status => {
                        let basic_text = basic_response.text().await.unwrap_or_default();
                        anyhow::bail!("Failed to create OAuth2 client via both endpoints: regular({})={}, basic({})={}",
                            status, text, basic_status, basic_text);
                    }
                }
            }
        }
    }

    pub async fn add_redirect_url(&self, client_name: &str, redirect_url: &str) -> Result<bool> {
        info!("🔗 Adding redirect URL to {}: {}", client_name, redirect_url);

        let url = format!("{}/v1/oauth2/{}/redirect_url", self.base_url, client_name);

        let req = AddRedirectUrlRequest {
            url: redirect_url.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header()?)
            .json(&req)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK | StatusCode::NO_CONTENT | StatusCode::CREATED => {
                info!("✅ Redirect URL added");
                Ok(true)
            }
            StatusCode::CONFLICT => {
                warn!("⚠️  Redirect URL already exists");
                Ok(false)
            }
            status => {
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to add redirect URL: {} - {}", status, text);
            }
        }
    }

    pub async fn enable_pkce(&self, client_name: &str) -> Result<bool> {
        info!("🔒 Enabling PKCE for {}", client_name);

        let url = format!("{}/v1/oauth2/{}/pkce", self.base_url, client_name);

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => {
                info!("✅ PKCE enabled");
                Ok(true)
            }
            status => {
                let text = response.text().await.unwrap_or_default();
                warn!("⚠️  PKCE enable may have failed: {} - {}", status, text);
                Ok(false)
            }
        }
    }

    pub async fn update_scope_map(
        &self,
        client_name: &str,
        group: &str,
        scopes: Vec<String>,
    ) -> Result<bool> {
        info!(
            "📋 Updating scope map for {}: {} -> {:?}",
            client_name, group, scopes
        );

        let url = format!("{}/v1/oauth2/{}/scopemap/{}", self.base_url, client_name, group);

        let req = UpdateScopeMapRequest {
            group: group.to_string(),
            scopes,
        };

        let response = self
            .client
            .put(&url)
            .header("Authorization", self.auth_header()?)
            .json(&req)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => {
                info!("✅ Scope map updated");
                Ok(true)
            }
            status => {
                let text = response.text().await.unwrap_or_default();
                warn!("⚠️  Scope map update may have failed: {} - {}", status, text);
                Ok(false)
            }
        }
    }

    pub async fn get_oauth2_client(&self, client_name: &str) -> Result<OAuth2ClientResponse> {
        info!("🔍 Getting OAuth2 client: {}", client_name);

        let url = format!("{}/v1/oauth2/{}", self.base_url, client_name);

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get OAuth2 client: {}", text);
        }

        let client: OAuth2ClientResponse = response.json().await?;
        Ok(client)
    }

    pub async fn create_group(&self, name: &str) -> Result<bool> {
        info!("👥 Creating group: {}", name);

        let url = format!("{}/v1/group", self.base_url);

        let req = CreateGroupRequest {
            name: name.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header()?)
            .json(&req)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK | StatusCode::CREATED => {
                info!("✅ Group '{}' created", name);
                Ok(true)
            }
            StatusCode::CONFLICT => {
                warn!("⚠️  Group '{}' already exists", name);
                Ok(false)
            }
            status => {
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to create group: {} - {}", status, text);
            }
        }
    }

    pub async fn create_person(
        &self,
        name: &str,
        displayname: &str,
        mail: Option<&str>,
    ) -> Result<bool> {
        info!("👤 Creating person: {}", name);

        let url = format!("{}/v1/person", self.base_url);

        let req = CreatePersonRequest {
            name: name.to_string(),
            displayname: displayname.to_string(),
            mail: mail.map(|m| vec![m.to_string()]),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header()?)
            .json(&req)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK | StatusCode::CREATED => {
                info!("✅ Person '{}' created", name);
                Ok(true)
            }
            StatusCode::CONFLICT => {
                warn!("⚠️  Person '{}' already exists", name);
                Ok(false)
            }
            status => {
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to create person: {} - {}", status, text);
            }
        }
    }

    pub async fn add_group_members(&self, group: &str, members: Vec<String>) -> Result<bool> {
        info!("➕ Adding members to {}: {:?}", group, members);

        let url = format!("{}/v1/group/{}/_attr/member", self.base_url, group);

        let req = AddMembersRequest { members };

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header()?)
            .json(&req)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => {
                info!("✅ Members added to {}", group);
                Ok(true)
            }
            status => {
                let text = response.text().await.unwrap_or_default();
                warn!("⚠️  Add members may have failed: {} - {}", status, text);
                Ok(false)
            }
        }
    }
}
