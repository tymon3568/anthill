import { redirect } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ cookies, url }) => {
	try {
		// Clear all auth-related cookies
		cookies.delete('access_token', { path: '/' });
		cookies.delete('refresh_token', { path: '/' });
		cookies.delete('oauth_code_verifier', { path: '/' });

		// Get redirect URL from query params or default to login
		const redirectTo = url.searchParams.get('redirect') || '/login';

		// Redirect to login page
		throw redirect(302, redirectTo);

	} catch (error) {
		console.error('Logout error:', error);
		// Even if there's an error, redirect to login
		throw redirect(302, '/login?error=logout_failed');
	}
};
