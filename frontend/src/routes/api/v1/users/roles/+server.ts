import { json, error, type RequestHandler } from '@sveltejs/kit';
import { authApi } from '$lib/api/auth';

// GET /api/v1/users/roles - Get current user's roles and groups
export const GET: RequestHandler = async ({ locals, cookies }) => {
	try {
		const result = await authApi.getUserRoles();

		if (!result.success || !result.data) {
			throw error(401, 'Unable to retrieve user roles');
		}

		return json(result.data);
	} catch (err) {
		console.error('Failed to get user roles:', err);
		throw error(500, 'Failed to retrieve user roles');
	}
};
