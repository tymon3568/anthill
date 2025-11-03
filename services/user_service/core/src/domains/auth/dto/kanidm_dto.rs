use serde::{Deserialize, Serialize};

/// Request to initiate OAuth2 authorization flow
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OAuth2AuthorizeReq {
    /// Optional state parameter for CSRF protection
    pub state: Option<String>,
}

/// Response with authorization URL
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OAuth2AuthorizeResp {
    /// Authorization URL to redirect user to
    pub authorization_url: String,
    /// State parameter for CSRF verification
    pub state: String,
    /// PKCE code verifier (must be stored client-side)
    pub code_verifier: String,
}

/// OAuth2 callback request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OAuth2CallbackReq {
    /// Authorization code from Kanidm
    pub code: String,
    /// State parameter for CSRF verification
    pub state: String,
    /// PKCE code verifier from authorization request
    pub code_verifier: String,
}

/// OAuth2 callback response (successful authentication)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OAuth2CallbackResp {
    /// Access token (Kanidm JWT)
    pub access_token: String,
    /// Refresh token for getting new access tokens
    pub refresh_token: Option<String>,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Expires in seconds
    pub expires_in: Option<i64>,
    /// User information
    pub user: KanidmUserInfo,
    /// Tenant information (if successfully mapped)
    pub tenant: Option<TenantInfo>,
}

/// Kanidm user information from JWT claims
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct KanidmUserInfo {
    /// Kanidm user UUID
    pub kanidm_user_id: String,
    /// User email
    pub email: Option<String>,
    /// Preferred username
    pub preferred_username: Option<String>,
    /// Kanidm groups
    pub groups: Vec<String>,
}

/// Tenant information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TenantInfo {
    /// Tenant ID (UUID)
    pub tenant_id: String,
    /// Tenant name
    pub name: String,
    /// Tenant slug
    pub slug: String,
    /// User role in this tenant
    pub role: String,
}

/// OAuth2 refresh token request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OAuth2RefreshReq {
    /// Refresh token from previous authentication
    pub refresh_token: String,
}

/// OAuth2 refresh token response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OAuth2RefreshResp {
    /// New access token
    pub access_token: String,
    /// New refresh token (optional)
    pub refresh_token: Option<String>,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Expires in seconds
    pub expires_in: Option<i64>,
}
