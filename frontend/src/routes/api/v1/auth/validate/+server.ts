import { json, error } from '@sveltejs/kit';
import { validateAndParseToken } from '$lib/auth/jwt';
import { createAuthError, AuthErrorCode } from '$lib/auth/errors';
import type { RequestHandler } from './$types';

/**
 * GET /api/v1/auth/validate - Validate current session
 *
 * This endpoint checks if the current session (access_token cookie) is valid.
 * Used by client-side code to verify if localStorage user data is still valid
 * before redirecting authenticated users.
 *
 * Returns:
 * - 200: Session is valid
 * - 401: No session or invalid token
 */
export const GET: RequestHandler = async ({ cookies }) => {
	const accessToken = cookies.get('access_token');

	if (!accessToken) {
		throw error(401, createAuthError(AuthErrorCode.NO_SESSION));
	}

	// SECURITY: Enable signature verification for server-side validation
	// This endpoint is used for authorization decisions, so we must verify the token
	const userInfo = await validateAndParseToken(accessToken, true);

	if (!userInfo) {
		throw error(401, createAuthError(AuthErrorCode.INVALID_TOKEN));
	}

	return json({ valid: true, userId: userInfo.userId });
};
