# Task: Implement OAuth2 Endpoints in User Service

**Task ID:** V1_MVP/03_User_Service/3.1_Kanidm_Integration/task_03.01.03_implement_oauth2_endpoints.md  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.1_Kanidm_Integration  
**Priority:** High  
**Status:** Todo  
**Assignee:**  
**Created Date:** 2025-11-03  
**Last Updated:** 2025-11-03

## Detailed Description

Implement OAuth2 endpoints in User Service API to handle the authorization flow with Kanidm, replacing traditional login/register endpoints.

## Specific Sub-tasks

- [ ] 1. Remove deprecated endpoints:
  - [ ] DELETE `POST /api/v1/auth/register`
  - [ ] DELETE `POST /api/v1/auth/login`
  - [ ] KEEP `POST /api/v1/auth/logout` (modify to invalidate Kanidm session)
- [ ] 2. Implement `GET /api/v1/auth/oauth/authorize`
  - [ ] Generate OAuth2 authorization URL with PKCE
  - [ ] Store PKCE verifier in session/Redis
  - [ ] Redirect to Kanidm authorization page
- [ ] 3. Implement `POST /api/v1/auth/oauth/callback`
  - [ ] Receive authorization code from Kanidm
  - [ ] Retrieve PKCE verifier from session
  - [ ] Exchange code for tokens using KanidmClient
  - [ ] Validate JWT token
  - [ ] Map Kanidm user to tenant (via groups)
  - [ ] Create/update user record in PostgreSQL
  - [ ] Return access_token to frontend
- [ ] 4. Implement `POST /api/v1/auth/oauth/refresh`
  - [ ] Receive refresh_token from client
  - [ ] Call Kanidm to refresh access_token
  - [ ] Return new access_token
- [ ] 5. Update DTOs for OAuth2 flow
- [ ] 6. Update OpenAPI documentation
- [ ] 7. Write integration tests

## Acceptance Criteria

- [ ] Old auth endpoints removed from codebase
- [ ] OAuth2 authorize endpoint generates valid authorization URL
- [ ] OAuth2 callback successfully exchanges code for tokens
- [ ] User is created/updated in PostgreSQL after OAuth callback
- [ ] Tenant mapping works correctly (groups → tenant_id)
- [ ] Token refresh endpoint returns new access_token
- [ ] All endpoints documented in OpenAPI spec
- [ ] Integration tests pass

## Dependencies

- Task 03.01.01 (Kanidm server running)
- Task 03.01.02 (KanidmClient library)
- Redis for session storage (PKCE verifier)

## Files to Modify

```
services/user_service/api/src/
├── handlers.rs                # Remove old handlers, add new
├── lib.rs or main.rs         # Update routes
└── oauth/                    # New module
    ├── mod.rs
    ├── handlers.rs           # OAuth2 handlers
    └── dto.rs                # OAuth2 DTOs

services/user_service/core/src/domains/auth/
├── dto/auth_dto.rs           # Add OAuth2 DTOs
└── domain/service.rs         # Update AuthService trait

services/user_service/infra/src/auth/
└── service.rs                # Implement OAuth2 methods
```

## Code Examples

### New DTOs
```rust
// OAuth2 authorization request
#[derive(Serialize, ToSchema)]
pub struct OAuth2AuthorizeResp {
    pub authorization_url: String,
    pub state: String,
}

// OAuth2 callback request
#[derive(Deserialize, Validate, ToSchema)]
pub struct OAuth2CallbackReq {
    pub code: String,
    pub state: String,
}

// OAuth2 callback response (same as old AuthResp)
#[derive(Serialize, ToSchema)]
pub struct OAuth2CallbackResp {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub token_type: String,
    pub user: UserInfo,
}

// Token refresh request
#[derive(Deserialize, Validate, ToSchema)]
pub struct OAuth2RefreshReq {
    pub refresh_token: String,
}
```

### Handler Implementation
```rust
/// Initiate OAuth2 flow
#[utoipa::path(
    get,
    path = "/api/v1/auth/oauth/authorize",
    tag = "auth",
    operation_id = "oauth_authorize",
    responses(
        (status = 200, body = OAuth2AuthorizeResp),
        (status = 500, body = ErrorResp)
    )
)]
pub async fn oauth_authorize<S: AuthService>(
    State(state): State<AppState<S>>,
) -> Result<Json<OAuth2AuthorizeResp>, AppError> {
    let (auth_url, pkce_verifier, state_token) = state
        .auth_service
        .generate_oauth_authorize_url()
        .await?;
    
    // Store PKCE verifier in Redis with state as key
    // TTL: 10 minutes
    state.redis.set_ex(&state_token, pkce_verifier, 600).await?;
    
    Ok(Json(OAuth2AuthorizeResp {
        authorization_url: auth_url,
        state: state_token,
    }))
}

/// Handle OAuth2 callback
#[utoipa::path(
    post,
    path = "/api/v1/auth/oauth/callback",
    tag = "auth",
    operation_id = "oauth_callback",
    request_body = OAuth2CallbackReq,
    responses(
        (status = 200, body = OAuth2CallbackResp),
        (status = 400, body = ErrorResp),
        (status = 401, body = ErrorResp)
    )
)]
pub async fn oauth_callback<S: AuthService>(
    State(state): State<AppState<S>>,
    Json(payload): Json<OAuth2CallbackReq>,
) -> Result<Json<OAuth2CallbackResp>, AppError> {
    // Validate request
    payload.validate()?;
    
    // Retrieve PKCE verifier from Redis
    let pkce_verifier = state.redis.get_del(&payload.state).await?
        .ok_or(AppError::InvalidState)?;
    
    // Exchange code for tokens and create/update user
    let resp = state
        .auth_service
        .handle_oauth_callback(payload.code, pkce_verifier)
        .await?;
    
    Ok(Json(resp))
}

/// Refresh access token
#[utoipa::path(
    post,
    path = "/api/v1/auth/oauth/refresh",
    tag = "auth",
    operation_id = "oauth_refresh",
    request_body = OAuth2RefreshReq,
    responses(
        (status = 200, body = OAuth2CallbackResp),
        (status = 401, body = ErrorResp)
    )
)]
pub async fn oauth_refresh<S: AuthService>(
    State(state): State<AppState<S>>,
    Json(payload): Json<OAuth2RefreshReq>,
) -> Result<Json<OAuth2CallbackResp>, AppError> {
    payload.validate()?;
    
    let resp = state
        .auth_service
        .refresh_oauth_token(payload.refresh_token)
        .await?;
    
    Ok(Json(resp))
}
```

## Testing Steps

```bash
# 1. Start services
docker-compose up -d
cargo run --bin user-service

# 2. Test OAuth2 flow
# Step 1: Get authorization URL
curl http://localhost:3000/api/v1/auth/oauth/authorize

# Step 2: Open URL in browser, login to Kanidm
# Step 3: Get redirected to callback with code

# Step 4: Exchange code for token
curl -X POST http://localhost:3000/api/v1/auth/oauth/callback \
  -H "Content-Type: application/json" \
  -d '{"code":"xyz","state":"abc"}'

# Step 5: Test token refresh
curl -X POST http://localhost:3000/api/v1/auth/oauth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token":"..."}'

# 3. Run integration tests
cargo test --package user_service_api --test oauth_integration
```

## References

- OAuth 2.0 RFC 6749: https://datatracker.ietf.org/doc/html/rfc6749
- PKCE RFC 7636: https://datatracker.ietf.org/doc/html/rfc7636
- OpenID Connect: https://openid.net/specs/openid-connect-core-1_0.html

## Notes

- Frontend must handle redirect to Kanidm and back
- Store state token in frontend to validate callback
- Use httpOnly cookies for tokens if possible (more secure than localStorage)
- Consider implementing "login with Kanidm" button in frontend
- Handle edge cases: user closes browser during OAuth flow, expired state, etc.
