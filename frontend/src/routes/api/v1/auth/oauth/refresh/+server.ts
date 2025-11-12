import { json, error } from '@sveltejs/kit';
import { env } from '$env/dynamic/public';
import type { RequestHandler } from './$types';
import type { Cookies } from '@sveltejs/kit';
import type { OAuth2RefreshReq, OAuth2RefreshResp } from '$lib/api/auth';

// Get backend user-service URL from environment
// In production, this MUST be set. In development, we allow fallback to localhost.
function getUserServiceUrl(): string {
	if (env.PUBLIC_USER_SERVICE_URL) {
		return env.PUBLIC_USER_SERVICE_URL;
	}

	// Only allow fallback in development
	if (env.PUBLIC_APP_ENV === 'development') {
		console.warn('PUBLIC_USER_SERVICE_URL not set, using development fallback: http://localhost:8000');
		return 'http://localhost:8000';
	}

	// Production must fail loudly if misconfigured
	throw new Error('PUBLIC_USER_SERVICE_URL environment variable is required in production');
}

const USER_SERVICE_URL = getUserServiceUrl();

export const POST: RequestHandler = async ({ request, cookies }) => {
	try {
		// Support both cookie-based and body-based refresh token flows
		let refreshToken: string | undefined;

		// Try to parse JSON body first (for explicit refresh_token in request)
		try {
			const contentLength = request.headers.get('content-length');
			if (contentLength && parseInt(contentLength) > 0) {
				const body: OAuth2RefreshReq = await request.json();
				refreshToken = body.refresh_token;
			}
		} catch (parseError) {
			// JSON parsing failed or no body - will fall back to cookies
			console.debug('No JSON body or parse failed, checking cookies for refresh_token');
		}

		// If no refresh_token in body, read from httpOnly cookie
		if (!refreshToken) {
			refreshToken = cookies.get('refresh_token');
		}

		// If still no refresh_token, fail
		if (!refreshToken) {
			throw error(
				401,
				JSON.stringify({
					code: 'MISSING_REFRESH_TOKEN',
					message: 'No refresh token provided in body or cookies'
				})
			);
		}

		// Synthesize the payload for backend
		const payload: OAuth2RefreshReq = { refresh_token: refreshToken };

		// Forward request to backend user-service
		const response = await fetch(`${USER_SERVICE_URL}/api/v1/auth/oauth/refresh`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(payload)
		});

		if (!response.ok) {
			const errorData = await response.json().catch(() => ({ message: 'Unknown error' }));

			// Clear all auth-related cookies on refresh failure
			cookies.delete('access_token', { path: '/' });
			cookies.delete('refresh_token', { path: '/' });
			cookies.delete('user_data', { path: '/' });

			throw error(response.status, JSON.stringify(errorData));
		}

		const data: OAuth2RefreshResp = await response.json();

		// Store new tokens in httpOnly cookies if present
		if (data.access_token) {
			const maxAge = data.expires_in || 3600; // Default 1 hour
			cookies.set('access_token', data.access_token, {
				path: '/',
				httpOnly: true,
				secure: true,
				sameSite: 'strict',
				maxAge: maxAge
			});
		}

		if (data.refresh_token) {
			cookies.set('refresh_token', data.refresh_token, {
				path: '/',
				httpOnly: true,
				secure: true,
				sameSite: 'strict',
				maxAge: 30 * 24 * 60 * 60 // 30 days
			});
		}

		return json(data);
	} catch (err) {
		console.error('OAuth2 refresh error:', err);

		if (err && typeof err === 'object' && 'status' in err) {
			throw err; // Re-throw SvelteKit errors
		}

		throw error(
			500,
			JSON.stringify({
				code: 'OAUTH_REFRESH_FAILED',
				message: 'Failed to refresh OAuth2 tokens'
			})
		);
	}
};
