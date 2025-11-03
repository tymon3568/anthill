# shared/jwt - DEPRECATED

⚠️ **This crate is deprecated and will be removed in a future version.**

## Reason for Deprecation

With the migration to Kanidm OAuth2/OIDC authentication, JWT token generation is now handled by Kanidm. This crate is only kept for:

1. **Legacy token support** - Validating existing user tokens during migration period
2. **Test utilities** - Integration tests that need to generate tokens
3. **Backward compatibility** - Dual authentication mode (Kanidm + legacy JWT)

## Migration Path

### For Production Code

**Before** (using shared_jwt):
```rust
use shared_jwt::{encode_jwt, Claims};

let claims = Claims::new_access(user_id, tenant_id, role, expiration);
let token = encode_jwt(&claims, secret)?;
```

**After** (using Kanidm):
```rust
// Users authenticate with Kanidm OAuth2
// Kanidm issues JWT tokens
// User service validates Kanidm JWTs with JWKS
```

### For Tests

**Before** (using shared_jwt):
```rust
use shared_jwt::{encode_jwt, Claims};

let token = encode_jwt(&claims, jwt_secret)?;
```

**After** (using test helpers):
```rust
use user_service_api::test_helpers::create_test_jwt;

let token = create_test_jwt(user_id, tenant_id, role);
```

## Removal Timeline

- **Phase 3** (Current): Dual authentication mode - both Kanidm and legacy JWT work
- **Phase 4**: Database migration - migrate all users to Kanidm
- **Phase 5**: Deprecation warnings - log warnings when legacy JWT used
- **Phase 6**: Remove legacy JWT support - only Kanidm authentication
- **Phase 7**: Delete this crate

## Dependencies to Remove Later

When removing this crate, also remove from:
- `Cargo.toml` workspace members
- `shared/auth/Cargo.toml`
- `services/user_service/infra/Cargo.toml`
- `services/user_service/api/Cargo.toml`

And refactor:
- `shared/auth/src/extractors.rs` - remove legacy JWT validation
- `services/user_service/infra/src/auth/service.rs` - remove JWT generation
- All test files using `shared_jwt`

## See Also

- `docs/KANIDM_MIGRATION_PLAN.md` - Full migration plan
- `shared/kanidm_client/` - New Kanidm OAuth2 client
- `shared/auth/src/extractors.rs` - Dual JWT validation implementation
