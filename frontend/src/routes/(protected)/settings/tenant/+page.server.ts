import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals }) => {
	// Check if user is owner - tenant settings are owner-only
	const user = locals.user;
	if (!user) {
		throw redirect(302, '/auth/login');
	}

	// Check if user has owner role
	const isOwner = user.role === 'owner';

	// Server-side authorization: redirect non-owners
	if (!isOwner) {
		throw redirect(302, '/settings');
	}

	return {
		user,
		isOwner: true
	};
};
