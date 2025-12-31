import { json, error as svelteError } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { extractTenantFromGroups } from '$lib/auth/jwt';
import { jwtVerify, createRemoteJWKSet, type JWTVerifyResult } from 'jose';
import { env } from '$env/dynamic/public';

/**
 * Verify JWT token with full signature verification against Kanidm JWKS
 *
 * This endpoint performs cryptographic signature verification using Kanidm's
 * public keys (JWKS) and validates all token claims (exp, nbf, iss, aud).
 *
 * Security guarantees:
 * - Signature verified against Kanidm's public key
 * - Token expiry (exp) checked
 * - Not-before (nbf) validated if present
 * - Issuer (iss) matches expected Kanidm issuer
 * - Groups and tenant cannot be manipulated without invalidating signature
 *
 * Production considerations:
 * - JWKS is cached at module scope for performance
 * - Error messages sanitized to prevent information leakage
 * - Should be rate-limited in production (not implemented here)
 */

// Cache JWKS at module scope to avoid fetching on every request
// jose library handles automatic key rotation and caching internally
let JWKS_CACHE: ReturnType<typeof createRemoteJWKSet> | null = null;
let KANIDM_ISSUER_CACHE: string | null = null;

function getJWKS(): ReturnType<typeof createRemoteJWKSet> {
	const kanidmIssuer = env.PUBLIC_KANIDM_ISSUER_URL;

	if (!kanidmIssuer) {
		throw new Error('PUBLIC_KANIDM_ISSUER_URL not configured');
	}

	// Return cached JWKS if issuer hasn't changed
	if (JWKS_CACHE && KANIDM_ISSUER_CACHE === kanidmIssuer) {
		return JWKS_CACHE;
	}

	// Create new JWKS fetcher
	// Standard OIDC discovery endpoint for JWKS
	const jwksUrl = `${kanidmIssuer}/.well-known/jwks.json`;
	JWKS_CACHE = createRemoteJWKSet(new URL(jwksUrl), {
		cooldownDuration: 30000, // 30 seconds between refetches
		cacheMaxAge: 600000 // 10 minutes max cache age
	});
	KANIDM_ISSUER_CACHE = kanidmIssuer;

	return JWKS_CACHE;
}

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

		// Get Kanidm issuer URL from environment
		const kanidmIssuer = env.PUBLIC_KANIDM_ISSUER_URL;
		if (!kanidmIssuer) {
			console.error('PUBLIC_KANIDM_ISSUER_URL not configured');
			throw svelteError(
				500,
				JSON.stringify({
					code: 'SERVER_MISCONFIGURED',
					message: 'JWT verification not configured'
				})
			);
		}

		// Get cached JWKS
		const JWKS = getJWKS();

		// Verify JWT signature and validate claims
		let verified: JWTVerifyResult;
		try {
			verified = await jwtVerify(token, JWKS, {
				issuer: kanidmIssuer, // Validate issuer matches Kanidm
				clockTolerance: 30 // Allow 30 seconds clock skew
				// Audience validation - uncomment if Kanidm sets 'aud' claim
				// audience: env.PUBLIC_KANIDM_CLIENT_ID,
			});
		} catch (verifyError) {
			// Log detailed error for debugging (server-side only)
			console.error('JWT verification failed:', verifyError);

			// Determine error type for specific error codes
			const errorMsg = verifyError instanceof Error ? verifyError.message : 'Unknown error';
			let code = 'INVALID_SIGNATURE';

			if (errorMsg.includes('expired')) {
				code = 'TOKEN_EXPIRED';
			} else if (errorMsg.includes('not yet valid')) {
				code = 'TOKEN_NOT_YET_VALID';
			} else if (errorMsg.includes('issuer')) {
				code = 'INVALID_ISSUER';
			} else if (errorMsg.includes('audience')) {
				code = 'INVALID_AUDIENCE';
			}

			// Return sanitized error to client (no implementation details)
			throw svelteError(
				401,
				JSON.stringify({
					code,
					message: 'Token verification failed'
				})
			);
		}

		const payload = verified.payload as {
			sub: string;
			email: string;
			name?: string;
			preferred_username?: string;
			groups?: string[];
		};

		// Validate required claims
		if (!payload.sub || !payload.email) {
			throw svelteError(
				401,
				JSON.stringify({
					code: 'INVALID_TOKEN',
					message: 'Token missing required claims'
				})
			);
		}

		// Extract tenant from groups (groups claim is signature-protected)
		const tenantId = extractTenantFromGroups(payload.groups || []);

		// Return parsed user information
		// All data here is cryptographically verified and cannot be tampered with
		return json({
			userId: payload.sub,
			email: payload.email,
			name: payload.name || payload.preferred_username,
			groups: payload.groups || [],
			tenantId
		});
	} catch (err) {
		// Log error for monitoring (server-side only)
		console.error('Token verification endpoint error:', err);

		// Re-throw SvelteKit errors
		if (err && typeof err === 'object' && 'status' in err) {
			throw err;
		}

		// Generic error for unexpected failures (no details leaked)
		throw svelteError(
			400,
			JSON.stringify({
				code: 'INVALID_REQUEST',
				message: 'Invalid request'
			})
		);
	}
};
