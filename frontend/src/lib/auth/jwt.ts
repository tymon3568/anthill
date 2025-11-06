import { VITE_KANIDM_ISSUER_URL } from '$env/static/public';

export interface KanidmJWT {
	sub: string;           // Kanidm user UUID
	email: string;         // User email
	preferred_username?: string;
	name?: string;
	groups: string[];      // Kanidm groups (tenant mappings)
	exp: number;           // Expiry timestamp
	iat: number;           // Issued at timestamp
	iss?: string;          // Issuer
	aud?: string;          // Audience
}

export interface UserInfo {
	userId: string;
	email: string;
	name?: string;
	groups: string[];
	tenantId?: string;
}

/**
 * Decode JWT payload without verification (for client-side use)
 * WARNING: This does NOT verify the JWT signature - only use for display purposes
 * Server-side verification should always be performed for security
 */
export function decodeJwtPayload(token: string): KanidmJWT | null {
	try {
		const payload = token.split('.')[1];
		// Handle URL-safe base64 (base64url)
		const base64 = payload.replace(/-/g, '+').replace(/_/g, '/');
		const decoded = JSON.parse(atob(base64));
		return decoded as KanidmJWT;
	} catch (error) {
		console.error('Failed to decode JWT:', error);
		return null;
	}
}

/**
 * Verify JWT signature using Kanidm's public key
 * NOTE: This should ideally be done server-side for security
 * Client-side verification is provided for completeness but has limitations
 */
export async function verifyJwtSignature(token: string): Promise<boolean> {
	try {
		// Split token into parts
		const parts = token.split('.');
		if (parts.length !== 3) return false;

		const [header, payload, signature] = parts;

		// Decode header to get algorithm and key ID
		const headerData = JSON.parse(atob(header.replace(/-/g, '+').replace(/_/g, '/')));
		const { alg, kid } = headerData;

		if (alg !== 'RS256') {
			console.error('Unsupported JWT algorithm:', alg);
			return false;
		}

		// Fetch JWKS from Kanidm
		const jwksUrl = `${VITE_KANIDM_ISSUER_URL}/.well-known/jwks.json`;
		const jwksResponse = await fetch(jwksUrl);

		if (!jwksResponse.ok) {
			console.error('Failed to fetch JWKS');
			return false;
		}

		const jwks = await jwksResponse.json();
		const key = jwks.keys.find((k: any) => k.kid === kid);

		if (!key) {
			console.error('Public key not found for kid:', kid);
			return false;
		}

		// Import the RSA public key
		const publicKey = await crypto.subtle.importKey(
			'jwk',
			key,
			{
				name: 'RSASSA-PKCS1-v1_5',
				hash: 'SHA-256',
			},
			false,
			['verify']
		);

		// Verify signature
		const encoder = new TextEncoder();
		const data = encoder.encode(`${header}.${payload}`);
		const signatureBytes = Uint8Array.from(atob(signature.replace(/-/g, '+').replace(/_/g, '/')), c => c.charCodeAt(0));

		return await crypto.subtle.verify('RSASSA-PKCS1-v1_5', publicKey, signatureBytes, data);

	} catch (error) {
		console.error('JWT signature verification failed:', error);
		return false;
	}
}

/**
 * Validate JWT structure and extract user information
 * @param accessToken JWT token to validate
 * @param verifySignature Whether to verify JWT signature (default: false for client-side)
 */
export async function validateAndParseToken(accessToken: string, verifySignature = false): Promise<UserInfo | null> {
	// Verify signature if requested
	if (verifySignature) {
		const isValid = await verifyJwtSignature(accessToken);
		if (!isValid) {
			console.error('JWT signature verification failed');
			return null;
		}
	}

	const payload = decodeJwtPayload(accessToken);
	if (!payload) return null;

	// Basic validation
	if (!payload.sub || !payload.email) {
		console.error('Invalid JWT: missing required claims');
		return null;
	}

	// Check expiry
	if (payload.exp && payload.exp * 1000 < Date.now()) {
		console.error('JWT token has expired');
		return null;
	}

	// Extract tenant information from groups
	const tenantId = extractTenantFromGroups(payload.groups);

	return {
		userId: payload.sub,
		email: payload.email,
		name: payload.name || payload.preferred_username,
		groups: payload.groups || [],
		tenantId
	};
}

/**
 * Extract tenant ID from Kanidm groups
 */
export function extractTenantFromGroups(groups: string[]): string | undefined {
	if (!groups || !Array.isArray(groups)) {
		return undefined;
	}

	// Look for groups that match tenant pattern
	// Example: ['tenant_acme_users', 'tenant_xyz_admins']
	const tenantGroup = groups.find(group =>
		group.startsWith('tenant_') && group.endsWith('_users')
	);

	if (tenantGroup) {
		// Extract tenant name from group (e.g., 'acme' from 'tenant_acme_users')
		return tenantGroup.replace('tenant_', '').replace('_users', '');
	}

	return undefined;
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
