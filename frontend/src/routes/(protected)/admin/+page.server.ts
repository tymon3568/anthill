import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

/**
 * Admin Index Page
 *
 * Redirects to the Users management page by default.
 */
export const load: PageServerLoad = async () => {
	redirect(302, '/admin/users');
};
