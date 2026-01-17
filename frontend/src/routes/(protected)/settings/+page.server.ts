import type { PageServerLoad } from './$types';

/**
 * Server-side load function for settings page
 * Passes user data from locals to the page
 */
export const load: PageServerLoad = async ({ locals }) => {
	return {
		user: locals.user
	};
};
