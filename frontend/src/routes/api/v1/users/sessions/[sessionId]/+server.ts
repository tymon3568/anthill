import { json, error, type RequestHandler } from '@sveltejs/kit';
import { authApi } from '$lib/api/auth';

// DELETE /api/v1/users/sessions/[sessionId] - Terminate specific session
export const DELETE: RequestHandler = async ({ params, locals, cookies }) => {
	try {
		const sessionId = params.sessionId;

		if (!sessionId) {
			throw error(400, 'Session ID is required');
		}

		const result = await authApi.terminateSession(sessionId);

		if (!result.success) {
			throw error(500, 'Failed to terminate session');
		}

		return json({ message: 'Session terminated successfully' });
	} catch (err) {
		console.error('Failed to terminate session:', err);
		throw error(500, 'Failed to terminate session');
	}
};
