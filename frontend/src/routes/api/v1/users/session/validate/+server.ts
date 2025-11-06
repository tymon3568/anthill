import { json, error, type RequestHandler } from '@sveltejs/kit';
import { authApi } from '$lib/api/auth';

// GET /api/v1/users/session/validate - Validate current session
export const GET: RequestHandler = async ({ locals, cookies }) => {
	try {
		const result = await authApi.validateSession();

		if (!result.success || !result.data) {
			return json({ valid: false });
		}

		return json(result.data);
	} catch (err) {
		console.error('Failed to validate session:', err);
		return json({ valid: false });
	}
};
