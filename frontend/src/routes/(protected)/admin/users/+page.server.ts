import type { PageServerLoad } from './$types';

/**
 * Server-side load function for admin users page
 * Passes admin context to the page
 */
export const load: PageServerLoad = async ({ locals }) => {
	return {
		user: locals.user
	};
};
