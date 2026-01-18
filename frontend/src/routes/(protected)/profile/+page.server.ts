import type { PageServerLoad } from './$types';

/**
 * Profile Page Server Load
 *
 * Server-side load function for the profile page.
 * Authentication is already verified by parent +layout.server.ts.
 * This function just passes user data to the page.
 */
export const load: PageServerLoad = async ({ parent }) => {
	// Get user from parent layout (which gets it from locals)
	const { user } = await parent();

	return {
		user
	};
};
