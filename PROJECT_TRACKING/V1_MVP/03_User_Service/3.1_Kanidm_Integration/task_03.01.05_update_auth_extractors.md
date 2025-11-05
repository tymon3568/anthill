# Task: Update Auth Extractors for Kanidm JWT

**Task ID:** V1_MVP/03_User_Service/3.1_Kanidm_Integration/task_03.01.05_update_auth_extractors.md  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.1_Kanidm_Integration  
**Priority:** High  
**Status:** Done  
**Assignee:** Claude  
**Created Date:** 2025-11-03  
**Last Updated:** 2025-11-03

## Detailed Description

Update authentication extractors in `shared/auth` to validate Kanidm JWT tokens and extract claims, replacing custom JWT validation logic.

## Specific Sub-tasks

- [x] 1. Remove custom JWT generation code from `shared/auth`
- [x] 2. Update `AuthUser` extractor to:
  - [x] Validate Kanidm JWT using KanidmClient
  - [x] Extract Kanidm claims (sub, email, groups)
  - [x] Resolve tenant_id from groups
  - [x] Populate AuthUser struct with Kanidm data
- [x] 3. Update `RequireAdmin` extractor to check Kanidm admin groups
- [x] 4. Update `RequirePermission` for Casbin with Kanidm user ID
- [x] 5. Remove `JwtSecretProvider` trait (no longer needed)
- [x] 6. Update error handling for Kanidm validation failures
- [x] 7. Add caching for validated tokens (Redis)
- [x] 8. Write unit tests for extractors
- [x] 9. Update all services using auth extractors

## Acceptance Criteria

- [x] `AuthUser` extractor validates Kanidm JWT correctly
- [x] `kanidm_user_id` (sub claim) is available in AuthUser
- [x] `tenant_id` resolved from groups claim
- [x] `RequireAdmin` checks for admin groups from Kanidm
- [x] Casbin enforcement uses Kanidm user_id as subject
- [x] Token validation failures return 401 Unauthorized
- [x] Expired tokens handled gracefully
- [x] Tests verify all extractor functionality

## Dependencies

- Task 03.01.02 (KanidmClient library)
- Task 03.01.04 (Group-tenant mapping)

## Files to Modify

```
shared/auth/src/
├── lib.rs                    # Remove JWT gen exports, update re-exports
├── extractors.rs             # Update all extractors
├── kanidm.rs                 # NEW: Kanidm JWT validation utilities
└── casbin/                   # Update Casbin subject format
    └── middleware.rs

shared/auth/Cargo.toml        # Add kanidm_client dependency
```

## Code Examples

### Updated AuthUser Struct
```rust
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,              // Internal user_id in PostgreSQL
    pub kanidm_user_id: Uuid,       // UUID from Kanidm (sub claim)
    pub tenant_id: Uuid,            // Resolved from groups
    pub email: String,              // From email claim
    pub username: String,           // From preferred_username claim
    pub groups: Vec<String>,        // Kanidm groups (for Casbin)
}
```

### Updated AuthUser Extractor
```rust
#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // 1. Extract token from Authorization header
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::MissingAuthHeader)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::InvalidAuthHeader)?;

        // 2. Get KanidmClient from extensions
        let kanidm_client = parts
            .extensions
            .get::<Arc<KanidmClient>>()
            .ok_or(AppError::InternalError("KanidmClient not found".into()))?;

        // 3. Validate JWT and extract claims
        let claims = kanidm_client
            .validate_token(token)
            .await
            .map_err(|e| AppError::InvalidToken(e.to_string()))?;

        // 4. Get tenant mapping repository
        let mapping_repo = parts
            .extensions
            .get::<Arc<dyn TenantGroupMappingRepository>>()
            .ok_or(AppError::InternalError("Mapping repo not found".into()))?;

        // 5. Resolve tenant from groups
        let tenant_id = resolve_tenant_from_groups(&claims.groups, &mapping_repo)
            .await?;

        // 6. Get user repository and fetch user
        let user_repo = parts
            .extensions
            .get::<Arc<dyn UserRepository>>()
            .ok_or(AppError::InternalError("User repo not found".into()))?;

        let kanidm_user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::InvalidToken("Invalid sub claim".into()))?;

        let user = user_repo
            .find_by_kanidm_id(kanidm_user_id)
            .await?
            .ok_or(AppError::UserNotFound)?;

        // 7. Return AuthUser
        Ok(AuthUser {
            user_id: user.user_id,
            kanidm_user_id,
            tenant_id,
            email: claims.email,
            username: claims.preferred_username,
            groups: claims.groups,
        })
    }
}

// Helper function
async fn resolve_tenant_from_groups(
    groups: &[String],
    repo: &Arc<dyn TenantGroupMappingRepository>,
) -> Result<Uuid, AppError> {
    for group_name in groups {
        if let Some(tenant_id) = repo.find_tenant_by_group_name(group_name).await? {
            return Ok(tenant_id);
        }
    }
    Err(AppError::TenantNotFound(
        "No tenant mapping for user groups".into()
    ))
}
```

### Updated RequireAdmin Extractor
```rust
#[async_trait]
impl<S> FromRequestParts<S> for RequireAdmin
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // First, extract AuthUser (validates JWT)
        let user = AuthUser::from_request_parts(parts, state).await?;

        // Check if user belongs to admin group for their tenant
        let admin_group_pattern = format!("tenant_.*_admins");
        let is_admin = user.groups.iter().any(|g| {
            g.ends_with("_admins") || g == "global_admins"
        });

        if !is_admin {
            return Err(AppError::Forbidden(
                "Admin access required".to_string()
            ));
        }

        Ok(RequireAdmin(user))
    }
}
```

### Updated Casbin Subject Format
```rust
// OLD format: user_id@tenant_id
// NEW format: kanidm_user_id@tenant_id

impl CasbinMiddleware {
    fn get_subject(user: &AuthUser) -> String {
        format!("{}@{}", user.kanidm_user_id, user.tenant_id)
    }
}

// Casbin policies will use Kanidm UUID as subject:
// p, <kanidm_user_uuid>@<tenant_id>, <tenant_id>, products, read
// p, <kanidm_user_uuid>@<tenant_id>, <tenant_id>, products, write
```

### Token Caching (Optional but Recommended)
```rust
// Cache validated tokens to avoid re-validating on every request
pub struct CachedKanidmValidator {
    client: Arc<KanidmClient>,
    cache: Arc<RwLock<HashMap<String, (KanidmClaims, Instant)>>>,
    cache_ttl: Duration,
}

impl CachedKanidmValidator {
    pub async fn validate_token(&self, token: &str) -> Result<KanidmClaims, KanidmError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some((claims, cached_at)) = cache.get(token) {
                if cached_at.elapsed() < self.cache_ttl {
                    return Ok(claims.clone());
                }
            }
        }

        // Not in cache or expired, validate with Kanidm
        let claims = self.client.validate_token(token).await?;

        // Store in cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(token.to_string(), (claims.clone(), Instant::now()));
        }

        Ok(claims)
    }
}
```

## Testing Steps

```bash
# 1. Get valid Kanidm token
TOKEN=$(curl -X POST http://localhost:3000/api/v1/auth/oauth/callback \
  -H "Content-Type: application/json" \
  -d '{"code":"...","state":"..."}' | jq -r '.access_token')

# 2. Test AuthUser extractor
curl http://localhost:3000/api/v1/users \
  -H "Authorization: Bearer $TOKEN"

# 3. Test RequireAdmin extractor
curl http://localhost:3000/api/v1/admin/users \
  -H "Authorization: Bearer $TOKEN"

# 4. Test with invalid token
curl http://localhost:3000/api/v1/users \
  -H "Authorization: Bearer invalid_token"
# Should return 401

# 5. Test with expired token
# (Wait for token to expire, or use old token)

# 6. Run unit tests
cargo test --package shared_auth --lib extractors
```

## Migration Notes

### For existing services using extractors:

1. **Add dependencies to AppState**:
```rust
pub struct AppState {
    pub kanidm_client: Arc<KanidmClient>,
    pub mapping_repo: Arc<dyn TenantGroupMappingRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    // ... other fields
}
```

2. **Update middleware injection**:
```rust
let app = Router::new()
    .route("/api/v1/users", get(list_users))
    .layer(Extension(Arc::new(kanidm_client)))
    .layer(Extension(mapping_repo))
    .layer(Extension(user_repo))
    .with_state(state);
```

3. **Update handler signatures** (no change needed if using extractors):
```rust
// Works the same way
pub async fn list_users(
    user: AuthUser,  // Automatically validated via Kanidm
) -> Result<Json<Vec<User>>, AppError> {
    // user.kanidm_user_id now available
    // user.tenant_id resolved from groups
}
```

## References

- Axum Extractors: https://docs.rs/axum/latest/axum/extract/
- JWT Validation Best Practices: https://tools.ietf.org/html/rfc8725

## Notes

- Token validation happens on **every request** - consider caching
- JWKS (public keys) should be cached with TTL (1 hour recommended)
- Handle network errors gracefully (Kanidm unreachable → fail open or closed?)
- Consider rate limiting token validation to prevent DoS on Kanidm
- Log all authentication failures for security monitoring

## AI Agent Log:
---
*   2025-11-05 10:45: Task status updated by Claude
    - Verified completion from Kanidm migration Phase 3
    - Auth extractors updated to use Kanidm JWT validation
    - Status: Done ✓
