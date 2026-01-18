/**
 * Auth Refresh Proxy
 *
 * This proxy forwards refresh token requests to the backend and sets new httpOnly cookies
 * on the frontend domain.
 */
import { json, type RequestHandler } from '@sveltejs/kit';
import { PUBLIC_USER_SERVICE_URL } from '$env/static/public';
import { dev } from '$app/environment';

export const POST: RequestHandler = async ({ cookies }) => {
	try {
		// Get refresh token from cookie
		const refreshToken = cookies.get('refresh_token');

		if (!refreshToken) {
			return json({ error: 'No refresh token' }, { status: 401 });
		}

		// Forward request to backend with refresh token in body
		const response = await fetch(`${PUBLIC_USER_SERVICE_URL}/api/v1/auth/refresh`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ refresh_token: refreshToken })
		});

		const data = await response.json();

		if (!response.ok) {
			// Clear invalid cookies
			cookies.delete('access_token', { path: '/' });
			cookies.delete('refresh_token', { path: '/' });
			return json(data, { status: response.status });
		}

		// Set new httpOnly cookies on the frontend domain
		if (data.access_token) {
			cookies.set('access_token', data.access_token, {
				path: '/',
				httpOnly: true,
				secure: !dev, // Secure in production (HTTPS), not in development
				sameSite: 'lax',
				maxAge: data.expires_in || 900
			});
		}

		if (data.refresh_token) {
			cookies.set('refresh_token', data.refresh_token, {
				path: '/',
				httpOnly: true,
				secure: !dev, // Secure in production (HTTPS), not in development
				sameSite: 'lax',
				maxAge: 60 * 60 * 24 * 7 // 7 days
			});
		}

		// Strip tokens from response - they're in httpOnly cookies now
		// This prevents XSS attacks from accessing tokens via JavaScript
		delete data.access_token;
		delete data.refresh_token;

		return json(data);
	} catch (error) {
		console.error('Refresh proxy error:', error);
		return json({ error: 'Internal server error' }, { status: 500 });
	}
};
