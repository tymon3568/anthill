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
		// Parse request body as OAuth2RefreshReq
		const body: OAuth2RefreshReq = await request.json();

		// Forward request to backend user-service
		const response = await fetch(`${USER_SERVICE_URL}/api/v1/auth/oauth/refresh`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(body)
		});

		if (!response.ok) {
			const errorData = await response.json().catch(() => ({ message: 'Unknown error' }));

			// Clear invalid tokens
			cookies.delete('access_token', { path: '/' });
			cookies.delete('refresh_token', { path: '/' });

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
