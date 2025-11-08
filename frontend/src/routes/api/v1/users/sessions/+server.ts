import { json, error, type RequestHandler } from '@sveltejs/kit';
import { authApi } from '$lib/api/auth';

// GET /api/v1/users/sessions - Get active sessions for current user
export const GET: RequestHandler = async ({ locals, cookies }) => {
	try {
		const result = await authApi.getActiveSessions();

		if (!result.success || !result.data) {
			throw error(401, 'Unable to retrieve active sessions');
		}

		return json(result.data);
	} catch (err) {
		console.error('Failed to get active sessions:', err);
		throw error(500, 'Failed to retrieve active sessions');
	}
};

// DELETE /api/v1/users/sessions - End all sessions for current user
export const DELETE: RequestHandler = async ({ locals, cookies }) => {
	try {
		const result = await authApi.endAllSessions();

		if (!result.success) {
			throw error(500, 'Failed to end all sessions');
		}

		return json({ message: 'All sessions ended successfully' });
	} catch (err) {
		console.error('Failed to end all sessions:', err);
		throw error(500, 'Failed to end all sessions');
	}
};
