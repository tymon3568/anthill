import { json, error, type RequestHandler } from '@sveltejs/kit';
import { authApi } from '$lib/api/auth';

// GET /api/v1/users/permissions/check - Check if user has specific permission
export const GET: RequestHandler = async ({ url, locals, cookies }) => {
	try {
		const resource = url.searchParams.get('resource');
		const action = url.searchParams.get('action');

		if (!resource || !action) {
			throw error(400, 'Missing resource or action parameters');
		}

		const result = await authApi.checkPermission(resource, action);

		if (!result.success) {
			throw error(500, 'Failed to check permission');
		}

		return json(result.data);
	} catch (err) {
		console.error('Failed to check permission:', err);
		throw error(500, 'Failed to check user permission');
	}
};
