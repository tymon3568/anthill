import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals }) => {
	// Check if user is owner - tenant settings are owner-only
	const user = locals.user;
	if (!user) {
		throw redirect(302, '/auth/login');
	}

	// Check if user has owner role by examining groups
	// Strict matching: exact 'owner' or suffix pattern '_owners'
	const isOwner = user.groups?.some((group) => group === 'owner' || group.endsWith('_owners'));

	return {
		user,
		isOwner: isOwner ?? false
	};
};
