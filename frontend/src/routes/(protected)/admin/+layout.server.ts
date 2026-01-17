import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';

/**
 * Admin Layout Server Load
 *
 * Server-side protection for admin routes.
 * Checks if the current user has admin role and redirects to dashboard if not.
 *
 * User info is available in `locals.user` (set by hooks.server.ts).
 * The user's role is determined by their Kanidm groups.
 */
export const load: LayoutServerLoad = async ({ locals, url }) => {
	// Get user from locals (set by hooks.server.ts during auth)
	const user = locals.user;

	if (!user) {
		// Not authenticated - redirect to login
		redirect(302, `/login?redirect=${encodeURIComponent(url.pathname)}`);
	}

	// Check for admin role by examining user's groups
	// Groups like 'tenant_xxx_admins' or 'admin' indicate admin access
	const isAdmin = user.groups?.some((group) => group.includes('admin') || group.includes('owner'));

	if (!isAdmin) {
		// Not an admin - redirect to dashboard with error message
		redirect(302, '/dashboard?error=unauthorized');
	}

	return {
		isAdmin: true
	};
};
