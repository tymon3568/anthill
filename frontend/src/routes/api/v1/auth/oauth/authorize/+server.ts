import { json, error, redirect } from '@sveltejs/kit';
import { env } from '$env/dynamic/public';
import type { RequestHandler } from './$types';
import type { OAuth2AuthorizeReq, OAuth2AuthorizeResp } from '$lib/api/auth';

// Get backend user-service URL from environment
// In production, this MUST be set. In development, we allow fallback to localhost.
function getUserServiceUrl(): string {
	if (env.PUBLIC_USER_SERVICE_URL) {
		return env.PUBLIC_USER_SERVICE_URL;
	}

	// Only allow fallback in development
	if (env.PUBLIC_APP_ENV === 'development') {
		console.warn(
			'PUBLIC_USER_SERVICE_URL not set, using development fallback: http://localhost:8000'
		);
		return 'http://localhost:8000';
	}

	// Production must fail loudly if misconfigured
	throw new Error('PUBLIC_USER_SERVICE_URL environment variable is required in production');
}

const USER_SERVICE_URL = getUserServiceUrl();

export const POST: RequestHandler = async ({ request, url, cookies }) => {
	try {
		// Parse request body as OAuth2AuthorizeReq
		const body: OAuth2AuthorizeReq = await request.json();

		// Forward request to backend user-service
		const response = await fetch(`${USER_SERVICE_URL}/api/v1/auth/oauth/authorize`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(body)
		});

		if (!response.ok) {
			const errorData = await response.json().catch(() => ({ message: 'Unknown error' }));
			throw error(response.status, JSON.stringify(errorData));
		}

		const data: OAuth2AuthorizeResp = await response.json();

		// Store PKCE parameters in secure httpOnly cookies for callback verification
		// These are short-lived (5 minutes) and httpOnly for security
		if (data.code_verifier) {
			cookies.set('oauth_code_verifier', data.code_verifier, {
				path: '/',
				httpOnly: true,
				secure: true,
				sameSite: 'lax', // Must be 'lax' to work with OAuth redirect flow
				maxAge: 300 // 5 minutes - short-lived for security
			});
		}

		if (data.state) {
			cookies.set('oauth_state', data.state, {
				path: '/',
				httpOnly: true,
				secure: true,
				sameSite: 'lax', // Must be 'lax' to work with OAuth redirect flow
				maxAge: 300 // 5 minutes - short-lived for security
			});
		}

		// Redirect to Kanidm authorization URL
		throw redirect(302, data.authorization_url);
	} catch (err) {
		console.error('OAuth2 authorize error:', err);

		if (err && typeof err === 'object' && 'status' in err) {
			throw err; // Re-throw SvelteKit errors
		}

		throw error(
			500,
			JSON.stringify({
				code: 'OAUTH_AUTHORIZE_FAILED',
				message: 'Failed to initiate OAuth2 authorization'
			})
		);
	}
};
