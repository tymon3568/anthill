import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals }) => {
	// Check if user is owner - payment settings are owner-only
	const user = locals.user;
	if (!user) {
		throw redirect(302, '/auth/login');
	}

	// Check if user has owner role by examining groups
	// Strict matching: exact 'owner' or suffix pattern '_owners'
	const isOwner =
		user.groups?.some((group) => group === 'owner' || group.endsWith('_owners')) ?? false;

	// Redirect non-owners - payment settings are owner-only
	if (!isOwner) {
		throw redirect(302, '/settings');
	}

	return {
		user,
		isOwner
	};
};
