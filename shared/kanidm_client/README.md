# shared_kanidm_client

Kanidm OAuth2/OIDC client library for Anthill inventory management system.

## Features

- ✅ OAuth2 Authorization Code Flow with PKCE
- ✅ JWT token validation with JWKS
- ✅ Token refresh
- ✅ User info retrieval
- ✅ Multi-tenant group extraction
- ✅ Async/await support (Tokio)

## Usage

### 1. Configuration

Set environment variables:

```bash
export KANIDM_URL=http://localhost:8300
export KANIDM_OAUTH2_CLIENT_ID=anthill
export KANIDM_OAUTH2_CLIENT_SECRET=your_client_secret
export OAUTH2_REDIRECT_URI=http://localhost:3000/oauth/callback
export OAUTH2_SCOPES=openid,profile,email,groups
```

Or create config programmatically:

```rust
use shared_kanidm_client::KanidmConfig;

let config = KanidmConfig {
    kanidm_url: "http://localhost:8300".to_string(),
    client_id: "anthill".to_string(),
    client_secret: "secret".to_string(),
    redirect_uri: "http://localhost:3000/oauth/callback".to_string(),
    scopes: vec!["openid".to_string(), "profile".to_string()],
    skip_jwt_verification: false, // NEVER true in production!
    allowed_issuers: vec!["http://localhost:8300".to_string()],
    expected_audience: Some("anthill".to_string()),
};
```

### 2. OAuth2 Flow

```rust
use shared_kanidm_client::{KanidmClient, KanidmConfig, KanidmOAuth2Client, PkceState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config
    let config = KanidmConfig::from_env()?;
    let client = KanidmClient::new(config)?;
    
    // Step 1: Generate authorization URL
    let pkce = PkceState::generate();
    let auth_url = client.authorization_url(&pkce)?;
    
    // Redirect user to auth_url
    println!("Redirect to: {}", auth_url);
    
    // Store pkce.state and pkce.code_verifier in session/Redis
    // (you'll need them for the callback)
    
    // Step 2: Handle callback (after user authorizes)
    let code = "authorization_code_from_kanidm";
    let tokens = client.exchange_code(code, &pkce).await?;
    
    println!("Access Token: {}", tokens.access_token);
    println!("Refresh Token: {:?}", tokens.refresh_token);
    
    // Step 3: Validate token and extract claims
    let claims = client.validate_token(&tokens.access_token).await?;
    
    println!("User UUID: {}", claims.sub);
    println!("Email: {:?}", claims.email);
    println!("Username: {:?}", claims.preferred_username);
    println!("Groups: {:?}", claims.groups);
    
    // Step 4: Extract tenant information
    let tenant_groups = claims.tenant_groups();
    println!("Tenant groups: {:?}", tenant_groups);
    
    let tenant_slugs = claims.tenant_slugs();
    println!("Tenant slugs: {:?}", tenant_slugs);
    
    Ok(())
}
```

### 3. Token Refresh

```rust
// When access token expires, use refresh token
let new_tokens = client.refresh_token(&old_refresh_token).await?;
let new_claims = client.validate_token(&new_tokens.access_token).await?;
```

### 4. Get User Info

```rust
// Alternative to validating JWT - call UserInfo endpoint
let userinfo = client.get_userinfo(&access_token).await?;
println!("User: {:?}", userinfo);
```

## API Reference

### `KanidmConfig`

Configuration for Kanidm client.

**Methods:**
- `from_env() -> Result<Self>` - Load from environment variables
- `validate() -> Result<()>` - Validate configuration

### `KanidmClient`

OAuth2 client implementation.

**Methods:**
- `new(config: KanidmConfig) -> Result<Self>` - Create new client
- `authorization_url(&self, pkce: &PkceState) -> Result<Url>` - Generate auth URL
- `exchange_code(&self, code: &str, pkce: &PkceState) -> Result<TokenResponse>` - Exchange code
- `refresh_token(&self, refresh_token: &str) -> Result<TokenResponse>` - Refresh token
- `validate_token(&self, token: &str) -> Result<KanidmClaims>` - Validate JWT
- `get_userinfo(&self, access_token: &str) -> Result<UserInfo>` - Get user info

### `PkceState`

PKCE state for OAuth2 flow.

**Methods:**
- `generate() -> Self` - Generate random PKCE state

**Fields:**
- `code_verifier: String` - Random verifier
- `code_challenge: String` - SHA256(verifier)
- `state: String` - Random state for CSRF protection

### `KanidmClaims`

JWT claims from Kanidm token.

**Methods:**
- `user_uuid() -> Result<Uuid>` - Extract user UUID
- `is_expired() -> bool` - Check if token expired
- `has_group(&self, group: &str) -> bool` - Check group membership
- `tenant_groups() -> Vec<String>` - Get groups starting with "tenant_"
- `tenant_slugs() -> Vec<String>` - Extract tenant slugs from groups

**Fields:**
- `sub: String` - User UUID
- `email: Option<String>` - Email address
- `preferred_username: Option<String>` - Username
- `groups: Vec<String>` - Group memberships
- `iat: i64` - Issued at timestamp
- `exp: i64` - Expiration timestamp
- `iss: String` - Issuer

## Error Handling

```rust
use shared_kanidm_client::KanidmError;

match client.validate_token(token).await {
    Ok(claims) => println!("Valid: {:?}", claims),
    Err(KanidmError::TokenExpired) => println!("Token expired, refresh needed"),
    Err(KanidmError::InvalidSignature) => println!("Invalid signature"),
    Err(KanidmError::ApiError { status, message }) => {
        println!("API error {}: {}", status, message)
    }
    Err(e) => println!("Other error: {}", e),
}
```

## Testing

```bash
# Run tests
cargo test --package shared_kanidm_client

# With output
cargo test --package shared_kanidm_client -- --nocapture
```

## Security Considerations

### Production Checklist

- ✅ NEVER set `skip_jwt_verification = true` in production
- ✅ Always use HTTPS for `kanidm_url` in production
- ✅ Store client_secret securely (environment variables, secrets manager)
- ✅ Use strong random values for PKCE
- ✅ Validate `state` parameter to prevent CSRF
- ✅ Store PKCE verifier securely in session (Redis, encrypted cookies)
- ✅ Implement token refresh before expiration
- ✅ Clear tokens on logout

### Development Mode

For local development, you can skip JWT verification:

```bash
export KANIDM_SKIP_JWT_VERIFICATION=true  # ONLY FOR DEV!
```

This will log a warning and should NEVER be used in production.

## Integration with Anthill

### In user_service/api

```rust
use shared_kanidm_client::{KanidmClient, KanidmConfig};

#[derive(Clone)]
pub struct AppState {
    pub kanidm_client: Arc<KanidmClient>,
    // ... other fields
}

// In main.rs
let kanidm_config = KanidmConfig::from_env()?;
let kanidm_client = Arc::new(KanidmClient::new(kanidm_config)?);

let app_state = AppState {
    kanidm_client,
    // ...
};
```

### In OAuth2 Endpoints

```rust
// GET /api/v1/auth/oauth/authorize
async fn oauth_authorize(
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let pkce = PkceState::generate();
    
    // Store pkce in Redis/session
    store_pkce_in_session(&pkce).await?;
    
    let auth_url = state.kanidm_client.authorization_url(&pkce)?;
    
    Ok(Redirect::temporary(auth_url.as_str()).into_response())
}

// POST /api/v1/auth/oauth/callback
async fn oauth_callback(
    State(state): State<AppState>,
    Json(req): Json<CallbackRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    // Retrieve pkce from session
    let pkce = get_pkce_from_session(&req.state).await?;
    
    // Exchange code for tokens
    let tokens = state.kanidm_client
        .exchange_code(&req.code, &pkce)
        .await?;
    
    // Validate and extract claims
    let claims = state.kanidm_client
        .validate_token(&tokens.access_token)
        .await?;
    
    // Map to tenant, create session, etc.
    // ...
    
    Ok(Json(tokens))
}
```

## License

Part of Anthill project.
