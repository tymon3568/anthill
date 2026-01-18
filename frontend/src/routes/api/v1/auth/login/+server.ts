/**
 * Auth Login Proxy
 *
 * This proxy forwards login requests to the backend and sets httpOnly cookies
 * on the frontend domain, allowing SvelteKit hooks to read them.
 */
import { json, type RequestHandler } from '@sveltejs/kit';
import { PUBLIC_USER_SERVICE_URL } from '$env/static/public';
import { dev } from '$app/environment';

export const POST: RequestHandler = async ({ request, cookies }) => {
	try {
		const body = await request.json();
		const tenantId = request.headers.get('X-Tenant-ID');

		// Forward request to backend
		const headers: HeadersInit = {
			'Content-Type': 'application/json'
		};

		if (tenantId) {
			headers['X-Tenant-ID'] = tenantId;
		}

		const response = await fetch(`${PUBLIC_USER_SERVICE_URL}/api/v1/auth/login`, {
			method: 'POST',
			headers,
			body: JSON.stringify(body)
		});

		const data = await response.json();

		if (!response.ok) {
			return json(data, { status: response.status });
		}

		// Set httpOnly cookies on the frontend domain
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
		console.error('Login proxy error:', error);
		return json({ error: 'Internal server error' }, { status: 500 });
	}
};
