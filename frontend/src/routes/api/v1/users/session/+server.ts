import { json, error, type RequestHandler } from '@sveltejs/kit';
import { authApi } from '$lib/api/auth';

// GET /api/v1/users/session - Get current session info
export const GET: RequestHandler = async ({ locals, cookies }) => {
	try {
		const result = await authApi.getSessionInfo();

		if (!result.success || !result.data) {
			throw error(401, 'Unable to retrieve session information');
		}

		return json(result.data);
	} catch (err) {
		console.error('Failed to get session info:', err);
		throw error(500, 'Failed to retrieve session information');
	}
};
