# JWT Verification Endpoint

## Overview

This endpoint performs **cryptographic JWT signature verification** against Kanidm's JWKS (JSON Web Key Set).

**Endpoint**: `POST /api/v1/auth/verify-token`

## Security Model

### What This Endpoint Does ✅

1. **Signature Verification**: Cryptographically verifies JWT signature using Kanidm's public keys
2. **Claims Validation**: Validates `exp` (expiry), `nbf` (not-before), `iss` (issuer)
3. **Tamper Protection**: Groups and tenantId are signature-protected (cannot be forged)
4. **Fail Closed**: Returns 401 on ANY verification failure

### What This Endpoint Does NOT Do ❌

1. **Rate Limiting**: No built-in rate limiting (add in production)
2. **Metrics/Monitoring**: No tracking of verification failures
3. **Token Revocation**: Does not check if token is revoked/blacklisted
4. **Audience Validation**: Currently commented out (enable if Kanidm sets `aud`)

## Production Deployment Checklist

### Required ✅

- [x] **JWKS Caching**: Module-level cache implemented
- [x] **Error Sanitization**: No internal error details leaked to client
- [x] **Type Safety**: Full TypeScript types
- [ ] **Rate Limiting**: Add rate limiting middleware
- [ ] **Monitoring**: Add metrics for verification failures
- [ ] **Alerting**: Alert on high failure rates

### Recommended ⚠️

- [ ] **Audience Validation**: Enable if Kanidm sets `aud` claim
- [ ] **Token Revocation**: Implement revocation check if needed
- [ ] **OIDC Discovery**: Use `.well-known/openid-configuration` instead of hardcoded JWKS URL
- [ ] **CORS Headers**: Configure if called from different origins
- [ ] **Request Validation**: Add request body size limits

### Environment Variables

```bash
PUBLIC_KANIDM_ISSUER_URL=https://idm.example.com  # Required
PUBLIC_KANIDM_CLIENT_ID=anthill-frontend         # Optional (for aud validation)
```

## Performance Characteristics

### JWKS Caching

- **First Request**: ~100-200ms (fetch JWKS from Kanidm)
- **Subsequent Requests**: ~5-10ms (cached JWKS)
- **Cache Duration**: 10 minutes max age
- **Cooldown**: 30 seconds between refetches

### Expected Latency

- **Success**: 5-15ms (with cached JWKS)
- **Failure**: 5-10ms (fast fail on invalid signature)
- **Network Issue**: 1-2s timeout (if JWKS unreachable)

## Error Codes

| Code                   | HTTP Status | Meaning             | Client Action          |
| ---------------------- | ----------- | ------------------- | ---------------------- |
| `INVALID_REQUEST`      | 400         | Malformed request   | Fix request format     |
| `TOKEN_EXPIRED`        | 401         | Token past expiry   | Refresh token          |
| `TOKEN_NOT_YET_VALID`  | 401         | Token nbf in future | Clock sync issue       |
| `INVALID_ISSUER`       | 401         | Wrong issuer        | Configuration error    |
| `INVALID_SIGNATURE`    | 401         | Signature mismatch  | Token forged/corrupted |
| `INVALID_TOKEN`        | 401         | Missing claims      | Malformed token        |
| `SERVER_MISCONFIGURED` | 500         | Missing env vars    | Server configuration   |

## Usage Example

```typescript
const response = await fetch('/api/v1/auth/verify-token', {
	method: 'POST',
	headers: { 'Content-Type': 'application/json' },
	body: JSON.stringify({ token: 'eyJhbGc...' }),
	credentials: 'include'
});

if (response.ok) {
	const userInfo = await response.json();
	// { userId, email, name, groups, tenantId }
} else {
	const error = await response.json();
	// { code, message }
}
```

## Production Rate Limiting Example

Add this middleware in `hooks.server.ts`:

```typescript
import { rateLimit } from '@sveltekit-rate-limiter/server';

const limiter = rateLimit({
	IP: [10, 'm'], // 10 requests per minute per IP
	IPUA: [5, 'm'] // 5 requests per minute per IP+UserAgent
});

export async function handle({ event, resolve }) {
	if (event.url.pathname === '/api/v1/auth/verify-token') {
		await limiter.check(event);
	}
	return resolve(event);
}
```

## Monitoring Example

Track verification failures:

```typescript
// Add to verify-token/+server.ts
import { metrics } from '$lib/monitoring';

// On verification failure:
metrics.increment('jwt.verification.failed', {
	error_code: code,
	issuer: kanidmIssuer
});

// On success:
metrics.increment('jwt.verification.success', {
	tenant_id: tenantId
});
```

## Security Considerations

### Why JWKS Caching is Safe

- JWKS contains **public keys only** (no secrets)
- Keys are rotated infrequently (days/weeks)
- Jose library handles automatic key rotation
- Cooldown prevents cache poisoning

### Attack Vectors Mitigated

1. **Token Forgery**: ❌ Signature verification prevents forged tokens
2. **Expired Tokens**: ❌ Exp claim validation rejects old tokens
3. **Future Tokens**: ❌ Nbf claim validation rejects premature tokens
4. **Wrong Issuer**: ❌ Issuer validation prevents cross-tenant attacks
5. **Manipulated Groups**: ❌ Signature protection prevents claim tampering

### Attack Vectors NOT Mitigated

1. **Token Replay**: ⚠️ Valid tokens can be reused until expiry
2. **Token Theft**: ⚠️ Stolen valid tokens will verify successfully
3. **Rate Limiting**: ⚠️ No protection against brute force attempts
4. **Token Revocation**: ⚠️ Revoked tokens still verify if not expired

## Integration with Kanidm

### JWKS Endpoint

Kanidm exposes JWKS at:

```
{issuer}/.well-known/jwks.json
```

Example:

```
https://idm.example.com/.well-known/jwks.json
```

### Expected Claims

```json
{
	"sub": "00000000-0000-0000-0000-000000000000",
	"email": "user@example.com",
	"name": "Full Name",
	"preferred_username": "username",
	"groups": ["tenant_acme_users", "tenant_acme_admins"],
	"exp": 1700000000,
	"iat": 1699999000,
	"iss": "https://idm.example.com",
	"aud": "anthill-frontend"
}
```

## Troubleshooting

### "SERVER_MISCONFIGURED" Error

**Cause**: `PUBLIC_KANIDM_ISSUER_URL` not set

**Fix**:

```bash
# .env
PUBLIC_KANIDM_ISSUER_URL=https://idm.example.com
```

### "INVALID_ISSUER" Error

**Cause**: Token issuer doesn't match configured issuer

**Fix**: Ensure `PUBLIC_KANIDM_ISSUER_URL` matches Kanidm's issuer claim

### High Latency on First Request

**Cause**: JWKS not cached, fetching from Kanidm

**Fix**: This is expected. Subsequent requests will be fast.

### "INVALID_SIGNATURE" for Valid Tokens

**Possible Causes**:

1. Wrong JWKS endpoint URL
2. Kanidm key rotation in progress
3. Clock skew > 30 seconds
4. Token from different Kanidm instance

## Testing

### Unit Test Example

```typescript
import { POST } from './+server';

describe('verify-token endpoint', () => {
	it('should verify valid token', async () => {
		const request = new Request('http://localhost/api/v1/auth/verify-token', {
			method: 'POST',
			body: JSON.stringify({ token: validToken })
		});

		const response = await POST({ request });
		expect(response.status).toBe(200);
	});

	it('should reject expired token', async () => {
		// Test with TOKEN_EXPIRED error
	});
});
```

## Changelog

- **v1.0.0**: Initial implementation with jose library
- **v1.1.0**: Added JWKS caching for production performance
- **v1.2.0**: Sanitized error messages to prevent information leakage
