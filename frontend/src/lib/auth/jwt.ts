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
 * In production, JWT verification should be done on the server
 */
export function decodeJwtPayload(token: string): KanidmJWT | null {
	try {
		const payload = token.split('.')[1];
		const decoded = JSON.parse(atob(payload));
		return decoded as KanidmJWT;
	} catch (error) {
		console.error('Failed to decode JWT:', error);
		return null;
	}
}

/**
 * Validate JWT structure and extract user information
 */
export function validateAndParseToken(accessToken: string): UserInfo | null {
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
