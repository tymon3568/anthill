import { json, error } from '@sveltejs/kit';
import { VITE_KANIDM_ISSUER_URL, VITE_KANIDM_CLIENT_ID } from '$env/static/public';
import { createAuthError, AuthErrorCode } from '$lib/auth/errors';
import type { RequestHandler } from './$types';

// OAuth2 Configuration from environment variables
const KANIDM_BASE_URL = VITE_KANIDM_ISSUER_URL;
const CLIENT_ID = VITE_KANIDM_CLIENT_ID;

export const POST: RequestHandler = async ({ request, cookies }) => {
	try {
		// Get refresh token from httpOnly cookie
		const refreshToken = cookies.get('refresh_token');

		if (!refreshToken) {
			throw error(401, JSON.stringify(createAuthError(AuthErrorCode.NO_SESSION)));
		}

		// Exchange refresh token for new access token
		const tokenResponse = await refreshAccessToken(refreshToken);

		// Store new tokens securely
		storeTokensSecurely(cookies, tokenResponse);

		return json({
			success: true,
			access_token: tokenResponse.access_token,
			expires_in: tokenResponse.expires_in,
			token_type: tokenResponse.token_type
		});

	} catch (err) {
		console.error('Token refresh error:', err);

		// Clear invalid tokens
		cookies.delete('access_token', { path: '/' });
		cookies.delete('refresh_token', { path: '/' });

		const authError = err instanceof Error && 'code' in err
			? err as any
			: createAuthError(AuthErrorCode.REFRESH_FAILED);
		throw error(authError.statusCode, JSON.stringify(authError));
	}
};

// Exchange refresh token for new access token
async function refreshAccessToken(refreshToken: string) {
	const tokenUrl = new URL('/oauth2/token', KANIDM_BASE_URL);

	const response = await fetch(tokenUrl.toString(), {
		method: 'POST',
		headers: {
			'Content-Type': 'application/x-www-form-urlencoded',
		},
		body: new URLSearchParams({
			grant_type: 'refresh_token',
			client_id: CLIENT_ID,
			refresh_token: refreshToken,
		}),
	});

	if (!response.ok) {
		const errorData = await response.text();
		throw createAuthError(AuthErrorCode.REFRESH_FAILED, `Token refresh failed: ${response.status} ${errorData}`);
	}

	const tokenData = await response.json();

	if (!tokenData.access_token) {
		throw createAuthError(AuthErrorCode.REFRESH_FAILED, 'No access token received from refresh');
	}

	return {
		access_token: tokenData.access_token,
		refresh_token: tokenData.refresh_token || refreshToken, // Keep old refresh token if not provided
		expires_in: tokenData.expires_in,
		token_type: tokenData.token_type,
	};
}

// Store tokens securely in httpOnly cookies
function storeTokensSecurely(cookies: any, tokenResponse: any) {
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
