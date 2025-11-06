import { json, error, type RequestHandler } from '@sveltejs/kit';
import { authApi } from '$lib/api/auth';

// GET /api/v1/users/permissions - Get current user's permissions and roles
export const GET: RequestHandler = async ({ locals, cookies }) => {
	try {
		const result = await authApi.getUserPermissions();

		if (!result.success || !result.data) {
			throw error(401, 'Unable to retrieve user permissions');
		}

		return json(result.data);
	} catch (err) {
		console.error('Failed to get user permissions:', err);
		throw error(500, 'Failed to retrieve user permissions');
	}
};
