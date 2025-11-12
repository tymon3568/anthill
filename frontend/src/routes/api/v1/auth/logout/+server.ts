import { json, error as svelteError } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

/**
 * Logout endpoint - clears httpOnly authentication cookies
 *
 * This endpoint is called via fetch() from client-side code to clear
 * httpOnly cookies (access_token, refresh_token) server-side.
 *
 * Returns JSON response instead of redirect to support programmatic logout.
 */
export const POST: RequestHandler = async ({ cookies }) => {
	try {
		// Clear all auth-related cookies
		cookies.delete('access_token', { path: '/' });
		cookies.delete('refresh_token', { path: '/' });
		cookies.delete('oauth_code_verifier', { path: '/' });
		cookies.delete('user_data', { path: '/' });

		// Return success response (no sensitive data)
		return json({
			success: true,
			message: 'Logout successful'
		});
	} catch (err) {
		console.error('Logout error:', err);

		throw svelteError(
			500,
			JSON.stringify({
				code: 'LOGOUT_FAILED',
				message: 'Failed to clear authentication cookies'
			})
		);
	}
};
