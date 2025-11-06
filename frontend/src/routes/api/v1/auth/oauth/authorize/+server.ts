import { redirect } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

// OAuth2 Configuration
const KANIDM_BASE_URL = 'https://idm.example.com'; // TODO: Move to environment variables
const CLIENT_ID = 'anthill-frontend'; // TODO: Move to environment variables
const REDIRECT_URI = 'http://localhost:5173/api/v1/auth/oauth/callback'; // TODO: Move to environment variables

export const GET: RequestHandler = async ({ url, cookies }) => {
	try {
		// Generate PKCE code verifier and challenge for security
		const codeVerifier = generateCodeVerifier();
		const codeChallenge = await generateCodeChallenge(codeVerifier);

		// Store code verifier in httpOnly cookie for later use in callback
		cookies.set('oauth_code_verifier', codeVerifier, {
			path: '/',
			httpOnly: true,
			secure: true,
			sameSite: 'strict',
			maxAge: 600 // 10 minutes
		});

		// Build OAuth2 authorization URL
		const authUrl = new URL('/ui/oauth2', KANIDM_BASE_URL);
		authUrl.searchParams.set('client_id', CLIENT_ID);
		authUrl.searchParams.set('redirect_uri', REDIRECT_URI);
		authUrl.searchParams.set('response_type', 'code');
		authUrl.searchParams.set('scope', 'openid profile email groups');
		authUrl.searchParams.set('code_challenge', codeChallenge);
		authUrl.searchParams.set('code_challenge_method', 'S256');
		authUrl.searchParams.set('state', generateState()); // CSRF protection

		// Redirect to Kanidm OAuth2 authorization endpoint
		throw redirect(302, authUrl.toString());

	} catch (error) {
		console.error('OAuth2 authorize error:', error);
		// Redirect to login page with error
		throw redirect(302, '/login?error=oauth_failed');
	}
};

// Generate cryptographically secure random string for PKCE
function generateCodeVerifier(): string {
	const array = new Uint8Array(32);
	crypto.getRandomValues(array);
	return btoa(String.fromCharCode(...array))
		.replace(/\+/g, '-')
		.replace(/\//g, '_')
		.replace(/=/g, '');
}

// Generate code challenge from verifier using SHA-256
async function generateCodeChallenge(verifier: string): Promise<string> {
	const encoder = new TextEncoder();
	const data = encoder.encode(verifier);
	const hash = await crypto.subtle.digest('SHA-256', data);
	const hashArray = new Uint8Array(hash);
	return btoa(String.fromCharCode(...hashArray))
		.replace(/\+/g, '-')
		.replace(/\//g, '_')
		.replace(/=/g, '');
}

// Generate state parameter for CSRF protection
function generateState(): string {
	const array = new Uint8Array(16);
	crypto.getRandomValues(array);
	return btoa(String.fromCharCode(...array))
		.replace(/\+/g, '-')
		.replace(/\//g, '_')
		.replace(/=/g, '');
}
