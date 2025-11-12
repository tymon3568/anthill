import { json, error as svelteError } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { decodeJwtPayload, extractTenantFromGroups, type KanidmJWT } from '$lib/auth/jwt';

/**
 * Verify JWT token and return parsed user information
 *
 * This endpoint validates the JWT signature (in production, this would verify
 * against Kanidm JWKS) and returns the parsed user information.
 *
 * For now, we decode and validate the token structure. In production, add
 * actual signature verification against Kanidm's public keys.
 */
export const POST: RequestHandler = async ({ request }) => {
	try {
		const { token } = await request.json();

		if (!token || typeof token !== 'string') {
			throw svelteError(
				400,
				JSON.stringify({
					code: 'INVALID_REQUEST',
					message: 'Token is required'
				})
			);
		}

		// Decode JWT payload
		const payload = decodeJwtPayload(token);
		if (!payload) {
			throw svelteError(
				401,
				JSON.stringify({
					code: 'INVALID_TOKEN',
					message: 'Failed to decode JWT token'
				})
			);
		}

		// Validate required fields
		if (!payload.sub || !payload.email) {
			throw svelteError(
				401,
				JSON.stringify({
					code: 'INVALID_TOKEN',
					message: 'Token missing required claims (sub, email)'
				})
			);
		}

		// TODO: In production, verify signature against Kanidm JWKS
		// For now, we trust the decoded payload structure
		// Example production implementation:
		// const jwks = await fetchKanidmJWKS();
		// const verified = await verifyJWT(token, jwks);
		// if (!verified) throw error(401, 'Invalid signature');

		// Extract tenant from groups
		const tenantId = extractTenantFromGroups(payload.groups);

		// Return parsed user information
		return json({
			userId: payload.sub,
			email: payload.email,
			name: payload.name || payload.preferred_username,
			groups: payload.groups || [],
			tenantId
		});
	} catch (err) {
		console.error('Token verification error:', err);

		// Re-throw SvelteKit errors
		if (err && typeof err === 'object' && 'status' in err) {
			throw err;
		}

		// Handle JSON parse errors
		throw svelteError(
			400,
			JSON.stringify({
				code: 'INVALID_REQUEST',
				message: 'Invalid request body'
			})
		);
	}
};
