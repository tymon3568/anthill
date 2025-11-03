# Task: Create Kanidm Client Library (shared/kanidm_client)

**Task ID:** V1_MVP/03_User_Service/3.1_Kanidm_Integration/task_03.01.02_create_kanidm_client_library.md  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.1_Kanidm_Integration  
**Priority:** High  
**Status:** Todo  
**Assignee:**  
**Created Date:** 2025-11-03  
**Last Updated:** 2025-11-03

## Detailed Description

Create a shared Rust library (`shared/kanidm_client`) to handle OAuth2/OIDC integration with Kanidm, including authorization flow, token exchange, and JWT validation.

## Specific Sub-tasks

- [ ] 1. Create `shared/kanidm_client` crate with Cargo.toml
- [ ] 2. Add dependencies: `oauth2`, `reqwest`, `jsonwebtoken`, `serde`, `thiserror`
- [ ] 3. Implement `KanidmClient` struct with configuration
- [ ] 4. Implement OAuth2 Authorization Code + PKCE flow:
  - [ ] Generate authorization URL with PKCE challenge
  - [ ] Exchange authorization code for tokens
  - [ ] Handle token response (access_token, refresh_token, id_token)
- [ ] 5. Implement token refresh functionality
- [ ] 6. Implement JWT validation:
  - [ ] Fetch Kanidm public key (JWKS)
  - [ ] Validate JWT signature
  - [ ] Validate token expiration
  - [ ] Extract and validate claims (sub, email, groups, etc.)
- [ ] 7. Implement user info retrieval from Kanidm
- [ ] 8. Create error types for Kanidm operations
- [ ] 9. Write unit tests for core functionality
- [ ] 10. Write integration tests with mock Kanidm server

## Acceptance Criteria

- [ ] `shared/kanidm_client` crate created and compiles
- [ ] Can generate OAuth2 authorization URL with PKCE
- [ ] Can exchange authorization code for tokens
- [ ] Can refresh access tokens using refresh token
- [ ] Can validate Kanidm JWT tokens
- [ ] Can extract claims from validated JWT
- [ ] Error handling for all failure scenarios
- [ ] Unit tests achieve >80% coverage
- [ ] Integration tests verify OAuth2 flow

## Dependencies

- Task 03.01.01 (Kanidm server must be running)
- `oauth2` crate for OAuth2 client
- `jsonwebtoken` for JWT validation
- `reqwest` for HTTP requests to Kanidm

## Files to Create

```
shared/kanidm_client/
├── Cargo.toml
└── src/
    ├── lib.rs              # Re-exports
    ├── client.rs           # KanidmClient implementation
    ├── oauth2.rs           # OAuth2 flow (authorize, callback, refresh)
    ├── jwt.rs              # JWT validation and claims extraction
    ├── types.rs            # DTOs (TokenResponse, UserInfo, Claims)
    ├── error.rs            # Error types
    └── config.rs           # Configuration struct
```

## Code Examples

### KanidmClient Configuration
```rust
pub struct KanidmConfig {
    pub issuer_url: String,           // https://idm.example.com
    pub client_id: String,             // anthill
    pub client_secret: String,         // from Kanidm
    pub redirect_uri: String,          // https://app.example.com/oauth/callback
    pub scopes: Vec<String>,           // openid, profile, email, groups
}

pub struct KanidmClient {
    config: KanidmConfig,
    http_client: reqwest::Client,
    oauth2_client: oauth2::basic::BasicClient,
}
```

### OAuth2 Flow
```rust
impl KanidmClient {
    // Generate authorization URL with PKCE
    pub fn authorize_url(&self) -> Result<(String, PkceCodeVerifier), KanidmError> {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, _csrf_token) = self.oauth2_client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge)
            .url();
        Ok((auth_url.to_string(), pkce_verifier))
    }

    // Exchange code for tokens
    pub async fn exchange_code(
        &self,
        code: &str,
        pkce_verifier: PkceCodeVerifier,
    ) -> Result<TokenResponse, KanidmError> {
        let token_result = self.oauth2_client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(pkce_verifier)
            .request_async(oauth2::reqwest::async_http_client)
            .await?;
        
        Ok(TokenResponse::from(token_result))
    }

    // Refresh access token
    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<TokenResponse, KanidmError> {
        // Implementation
    }
}
```

### JWT Validation
```rust
impl KanidmClient {
    // Validate JWT and extract claims
    pub async fn validate_token(&self, token: &str) -> Result<KanidmClaims, KanidmError> {
        let jwks = self.fetch_jwks().await?;
        let header = jsonwebtoken::decode_header(token)?;
        let kid = header.kid.ok_or(KanidmError::MissingKeyId)?;
        let jwk = jwks.find(&kid).ok_or(KanidmError::KeyNotFound)?;
        
        let validation = Validation::new(header.alg);
        let token_data = jsonwebtoken::decode::<KanidmClaims>(
            token,
            &DecodingKey::from_jwk(jwk)?,
            &validation,
        )?;
        
        Ok(token_data.claims)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KanidmClaims {
    pub sub: String,                    // User ID in Kanidm
    pub email: String,
    pub preferred_username: String,
    pub groups: Vec<String>,            // For tenant mapping
    pub exp: i64,
    pub iat: i64,
}
```

## Testing Steps

```bash
# 1. Create and build crate
cd shared/kanidm_client
cargo build

# 2. Run unit tests
cargo test

# 3. Run integration tests (requires Kanidm)
cargo test --test integration_tests -- --ignored

# 4. Check coverage
cargo tarpaulin --out Html
```

## References

- OAuth2 crate: https://docs.rs/oauth2/
- jsonwebtoken crate: https://docs.rs/jsonwebtoken/
- Kanidm OAuth2 API: https://kanidm.github.io/kanidm/master/integrations/oauth2.html
- PKCE RFC 7636: https://datatracker.ietf.org/doc/html/rfc7636

## Notes

- Store PKCE verifier in session (Redis or encrypted cookie) between authorize and callback
- Cache JWKS with TTL (1 hour) to avoid fetching on every request
- Handle token expiration gracefully (auto-refresh if refresh_token available)
- Validate `iss` claim matches Kanidm issuer URL
- Validate `aud` claim contains our client_id
