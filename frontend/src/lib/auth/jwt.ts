// Simplified JWT handling for client-side use only
// Removed complex signature verification - not needed for client-side

export interface KanidmJWT {
	sub: string; // Kanidm user UUID
	email: string; // User email
	preferred_username?: string;
	name?: string;
	groups: string[]; // Kanidm groups (tenant mappings)
	exp: number; // Expiry timestamp
	iat: number; // Issued at timestamp
	iss?: string; // Issuer
	aud?: string; // Audience
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
 * Extract tenant ID from Kanidm groups
 */
export function extractTenantFromGroups(groups: string[]): string | undefined {
	if (!groups || !Array.isArray(groups)) {
		return undefined;
	}

	// Look for groups that match tenant pattern
	const tenantGroup = groups.find(
		(group) => group.startsWith('tenant_') && group.endsWith('_users')
	);

	if (tenantGroup) {
		// Extract tenant name from group
		return tenantGroup.replace('tenant_', '').replace('_users', '');
	}

	return undefined;
}

/**
 * Validate and parse JWT token (server-side with signature verification)
 * This is a placeholder - in production, you'd verify the JWT signature
 * against the Kanidm public key
 */
export async function validateAndParseToken(
	token: string,
	verifySignature: boolean = false
): Promise<UserInfo | null> {
	try {
		// For now, just decode without verification (client-side behavior)
		// In production, you'd verify the signature here
		const payload = decodeJwtPayload(token);
		if (!payload) return null;

		// Check if token is expired
		if (isTokenExpired(token)) return null;

		// Extract tenant from groups
		const tenantId = extractTenantFromGroups(payload.groups);

		return {
			userId: payload.sub,
			email: payload.email,
			name: payload.name || payload.preferred_username,
			groups: payload.groups,
			tenantId
		};
	} catch (error) {
		console.error('Token validation failed:', error);
		return null;
	}
}
