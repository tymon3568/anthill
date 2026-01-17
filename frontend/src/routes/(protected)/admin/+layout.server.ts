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
		throw redirect(302, `/login?redirect=${encodeURIComponent(url.pathname)}`);
	}

	// Check for admin role by examining user's groups
	// Strict matching: exact 'admin'/'owner' or suffix pattern '_admins'/'_owners'
	const isAdmin = user.groups?.some(
		(group) =>
			group === 'admin' ||
			group === 'owner' ||
			group.endsWith('_admins') ||
			group.endsWith('_owners')
	);

	if (!isAdmin) {
		// Not an admin - redirect to dashboard with error message
		throw redirect(302, '/dashboard?error=unauthorized');
	}

	return {
		isAdmin: true
	};
};
