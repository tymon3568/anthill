import { json, error, redirect } from '@sveltejs/kit';
import { env } from '$env/dynamic/public';
import type { RequestHandler } from './$types';
import type { OAuth2AuthorizeReq, OAuth2AuthorizeResp } from '$lib/api/auth';

// Get backend user-service URL from environment
const USER_SERVICE_URL = (env as any).PUBLIC_USER_SERVICE_URL || 'http://localhost:8000';

export const POST: RequestHandler = async ({ request, url }) => {
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

		// Store code_verifier in session for later use in callback
		// This is needed for PKCE verification

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
