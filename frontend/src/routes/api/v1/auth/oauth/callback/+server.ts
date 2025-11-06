import { redirect } from '@sveltejs/kit';
import { env } from '$env/dynamic/public';
import { validateAndParseToken } from '$lib/auth/jwt';
import { createAuthError, AuthErrorCode } from '$lib/auth/errors';
import type { RequestHandler } from './$types';
import type { Cookies } from '@sveltejs/kit';

interface TokenResponse {
	access_token: string;
	refresh_token?: string;
	expires_in?: number;
	token_type?: string;
}

// OAuth2 Configuration from environment variables
const KANIDM_BASE_URL = (env as any).PUBLIC_KANIDM_ISSUER_URL;
const CLIENT_ID = (env as any).PUBLIC_KANIDM_CLIENT_ID;
const REDIRECT_URI = (env as any).PUBLIC_KANIDM_REDIRECT_URI;

export const GET: RequestHandler = async ({ url, cookies, locals }) => {
	try {
		const code = url.searchParams.get('code');
		const state = url.searchParams.get('state');
		const error = url.searchParams.get('error');

		// Validate state parameter for CSRF protection
		const storedState = cookies.get('oauth_state');
		if (!state || !storedState || state !== storedState) {
			throw createAuthError(AuthErrorCode.INVALID_STATE, 'Invalid state parameter');
		}
		cookies.delete('oauth_state', { path: '/' });

		// Handle OAuth2 errors from Kanidm
		if (error) {
			console.error('OAuth2 callback error:', error);
			throw redirect(302, `/login?error=${error}`);
		}

		// Validate required parameters
		if (!code) {
			throw createAuthError(AuthErrorCode.INVALID_CODE);
		}

		// Get code verifier from cookie
		const codeVerifier = cookies.get('oauth_code_verifier');
		if (!codeVerifier) {
			throw createAuthError(AuthErrorCode.MISSING_VERIFIER);
		}

		// Exchange authorization code for tokens
		const tokenResponse = await exchangeCodeForTokens(code, codeVerifier);

		// Validate and parse JWT token
		const userInfo = validateAndParseToken(tokenResponse.access_token);

		if (!userInfo) {
			throw createAuthError(AuthErrorCode.INVALID_TOKEN);
		}

		// Store tokens securely in httpOnly cookies
		storeTokensSecurely(cookies, tokenResponse);

		// Update auth store with user information
		// Note: This would typically be handled by the auth store
		// For now, we'll redirect to dashboard

		// Clear the code verifier cookie
		cookies.delete('oauth_code_verifier', { path: '/' });

		// Redirect to dashboard on success
		throw redirect(302, '/dashboard');

	} catch (error) {
		console.error('OAuth2 callback error:', error);

		// Clear any stored tokens on error
		cookies.delete('access_token', { path: '/' });
		cookies.delete('refresh_token', { path: '/' });

		// Handle auth errors with proper error codes
		if (error instanceof Error && 'code' in error) {
			const authError = error as any;
			throw redirect(302, `/login?error=${authError.code}&message=${encodeURIComponent(authError.message)}`);
		} else {
			throw redirect(302, '/login?error=callback_failed');
		}
	}
};

// Exchange authorization code for access and refresh tokens
async function exchangeCodeForTokens(code: string, codeVerifier: string) {
	const tokenUrl = new URL('/oauth2/token', KANIDM_BASE_URL);

	const response = await fetch(tokenUrl.toString(), {
		method: 'POST',
		headers: {
			'Content-Type': 'application/x-www-form-urlencoded',
		},
		body: new URLSearchParams({
			grant_type: 'authorization_code',
			client_id: CLIENT_ID,
			code: code,
			redirect_uri: REDIRECT_URI,
			code_verifier: codeVerifier,
		}),
	});

	if (!response.ok) {
		const errorData = await response.text();
		throw createAuthError(AuthErrorCode.TOKEN_EXCHANGE_FAILED, `Token exchange failed: ${response.status} ${errorData}`);
	}

	const tokenData = await response.json();

	if (!tokenData.access_token) {
		throw createAuthError(AuthErrorCode.TOKEN_EXCHANGE_FAILED, 'No access token received');
	}

	return {
		access_token: tokenData.access_token,
		refresh_token: tokenData.refresh_token,
		expires_in: tokenData.expires_in,
		token_type: tokenData.token_type,
	};
}

// Store tokens securely in httpOnly cookies
function storeTokensSecurely(cookies: Cookies, tokenResponse: TokenResponse) {
	const maxAge = tokenResponse.expires_in || 3600; // Default 1 hour

	cookies.set('access_token', tokenResponse.access_token, {
		path: '/',
		httpOnly: true,
		secure: true,
		sameSite: 'strict',
		maxAge: maxAge,
	});

	if (tokenResponse.refresh_token) {
		cookies.set('refresh_token', tokenResponse.refresh_token, {
			path: '/',
			httpOnly: true,
			secure: true,
			sameSite: 'strict',
			maxAge: 30 * 24 * 60 * 60, // 30 days
		});
	}
}
