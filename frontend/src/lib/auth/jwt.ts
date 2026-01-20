// Simplified JWT handling for client-side use only
// Removed complex signature verification - not needed for client-side

export interface JWTPayload {
	sub: string; // User UUID
	email: string; // User email
	preferred_username?: string;
	name?: string;
	exp: number; // Expiry timestamp
	iat: number; // Issued at timestamp
	iss?: string; // Issuer
	aud?: string; // Audience
}

// Backend JWT format (from user_service)
export interface BackendJWT {
	sub: string; // User UUID
	tenant_id: string; // Tenant UUID
	role: string; // User role (owner, admin, manager, user)
	exp: number; // Expiry timestamp
	iat: number; // Issued at timestamp
	token_type: string; // "access" or "refresh"
	tenant_v?: number; // Tenant authz version
	user_v?: number; // User authz version
}

export interface UserInfo {
	userId: string;
	email: string;
	name?: string;
	tenantId?: string;
	role?: string;
}

/**
 * Decode JWT payload without verification (for client-side use)
 */
export function decodeJwtPayload(token: string): JWTPayload | null {
	try {
		const payload = token.split('.')[1];
		// Handle URL-safe base64 (base64url)
		let base64 = payload.replace(/-/g, '+').replace(/_/g, '/');
		// Add padding if needed (base64 strings must have length divisible by 4)
		const paddingNeeded = (4 - (base64.length % 4)) % 4;
		base64 += '='.repeat(paddingNeeded);
		const decoded = JSON.parse(atob(base64));
		return decoded as JWTPayload;
	} catch (error) {
		console.error('Failed to decode JWT:', error);
		return null;
	}
}

/**
 * Check if JWT token is expired
 */
export function isTokenExpired(token: string): boolean {
	const payload = decodeJwtPayload(token);
	if (!payload || !payload.exp) return true;

	return payload.exp * 1000 < Date.now();
}

/**
 * Get token expiry time in milliseconds
 */
export function getTokenExpiry(token: string): number | null {
	const payload = decodeJwtPayload(token);
	return payload?.exp ? payload.exp * 1000 : null;
}

/**
 * Check if token needs refresh (expires within 5 minutes)
 */
export function shouldRefreshToken(token: string): boolean {
	const expiry = getTokenExpiry(token);
	if (!expiry) return true;

	const fiveMinutes = 5 * 60 * 1000;
	return expiry - Date.now() < fiveMinutes;
}

/**
 * Validate and parse JWT token with optional signature verification
 *
 * @param token - JWT token to validate
 * @param verifySignature - If true, performs full cryptographic verification via server
 * @returns UserInfo if valid, null otherwise
 *
 * CRITICAL SECURITY MODES:
 *
 * verifySignature=true (SECURE):
 *   - Calls server endpoint that performs full JWT signature verification
 *   - Validates signature against JWKS (cryptographic verification)
 *   - Checks exp, nbf, iss claims
 *   - Rejects expired or manipulated tokens
 *   - USE THIS for any security-critical operations (authorization, access control)
 *
 * verifySignature=false (INSECURE):
 *   - Client-side base64 decode only (NO signature verification)
 *   - Anyone can forge tokens with arbitrary claims
 *   - Only checks basic expiry via exp claim (which can be forged)
 *   - USE THIS ONLY for non-security-critical UI operations (display name, etc.)
 *   - NEVER use for authorization decisions
 */
export async function validateAndParseToken(
	token: string,
	verifySignature: boolean = false
): Promise<UserInfo | null> {
	try {
		if (verifySignature) {
			// SECURE PATH: Server-side cryptographic verification
			// The backend performs full JWT signature verification
			try {
				const response = await fetch('/api/v1/auth/verify-token', {
					method: 'POST',
					headers: {
						'Content-Type': 'application/json'
					},
					body: JSON.stringify({ token }),
					credentials: 'include'
				});

				if (!response.ok) {
					console.error('Server-side token verification failed:', response.status);
					return null; // Reject unverified/expired tokens
				}

				// Trust server response - it has cryptographically verified the token
				const userInfo: UserInfo = await response.json();
				return userInfo;
			} catch (networkError) {
				console.error('Token verification network error:', networkError);
				return null; // Do not accept unverified tokens - fail closed
			}
		}

		// INSECURE PATH: Client-side decode without signature verification
		// ⚠️ WARNING: Use ONLY for non-security-critical operations
		// This path is vulnerable to token forgery and manipulation
		const payload = decodeJwtPayload(token);
		if (!payload) return null;

		// Check if token is expired (note: exp claim itself can be forged!)
		if (isTokenExpired(token)) return null;

		// Detect token type: Backend JWT has tenant_id
		const isBackendToken = 'tenant_id' in payload && 'role' in payload;

		if (isBackendToken) {
			// Backend JWT format (from user_service)
			const backendPayload = payload as unknown as BackendJWT;
			return {
				userId: backendPayload.sub,
				email: '', // Backend JWT doesn't include email, will be fetched from user info
				name: undefined,
				tenantId: backendPayload.tenant_id,
				role: backendPayload.role
			};
		}

		// Standard JWT format
		return {
			userId: payload.sub,
			email: payload.email,
			name: payload.name || payload.preferred_username
		};
	} catch (error) {
		console.error('Token validation failed:', error);
		return null;
	}
}
