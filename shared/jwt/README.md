# shared/jwt - JWT Token Utilities

This crate provides JWT (JSON Web Token) utilities for the Anthill platform.

## Purpose

The `shared_jwt` crate handles:

1. **JWT Token Generation** - Creating access and refresh tokens for authenticated users
2. **JWT Token Validation** - Decoding and validating tokens with signature verification
3. **Claims Management** - Defining the structure and types of JWT claims

## Usage

### For Production Code

```rust
use shared_jwt::{encode_jwt, decode_jwt, Claims};

// Create access token claims
let claims = Claims::new_access(user_id, tenant_id, role, expiration_seconds);
let access_token = encode_jwt(&claims, &jwt_secret)?;

// Create refresh token claims
let refresh_claims = Claims::new_refresh(user_id, tenant_id, role, refresh_expiration);
let refresh_token = encode_jwt(&refresh_claims, &jwt_secret)?;

// Validate and decode a token
let claims = decode_jwt(&token, &jwt_secret)?;
```

### For Tests

```rust
use shared_jwt::{encode_jwt, Claims};

// Generate test tokens
let claims = Claims::new_access(test_user_id, test_tenant_id, "user".to_string(), 3600);
let token = encode_jwt(&claims, &test_jwt_secret)?;
```

## Token Types

- **Access Token**: Short-lived token (default: 15 minutes) for API authentication
- **Refresh Token**: Longer-lived token (default: 7 days) for obtaining new access tokens

## Security Notes

- JWT secrets should be at least 256 bits (32 bytes) for HS256
- Tokens should be transmitted over HTTPS only
- Access tokens are stored hashed (SHA-256) in session records
- Refresh tokens enable token rotation for security

## Dependencies

This crate is used by:
- `shared/auth` - JWT validation in request extractors
- `services/user_service/infra` - Token generation during login/registration
- `services/user_service/api` - Authentication handlers

## See Also

- `shared/auth/src/extractors.rs` - JWT validation in API requests
- `services/user_service/infra/src/auth/service.rs` - Token generation
