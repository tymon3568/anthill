import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';

/**
 * Protected Layout Server Load
 *
 * Server-side protection for all routes in the (protected) group.
 * This ensures authentication is verified on the server before any page loads.
 *
 * User info is available in `locals.user` (set by hooks.server.ts during auth).
 *
 * Security: This provides defense-in-depth alongside hooks.server.ts.
 * Even if hooks miss a route, this layout will catch it.
 */
export const load: LayoutServerLoad = async ({ locals, url }) => {
	// Get user from locals (set by hooks.server.ts during auth)
	const user = locals.user;

	if (!user) {
		// Not authenticated - redirect to login
		throw redirect(302, `/login?redirect=${encodeURIComponent(url.pathname)}`);
	}

	// Return user data to all child routes
	return {
		user
	};
};
