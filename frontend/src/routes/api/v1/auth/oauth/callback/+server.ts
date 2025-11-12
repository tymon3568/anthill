import { json, error, redirect } from '@sveltejs/kit';
import { env } from '$env/dynamic/public';
import type { RequestHandler } from './$types';
import type { Cookies } from '@sveltejs/kit';
import type { OAuth2CallbackReq, OAuth2CallbackResp } from '$lib/api/auth';

// Get backend user-service URL from environment
const USER_SERVICE_URL = (env as any).PUBLIC_USER_SERVICE_URL || 'http://localhost:8000';

export const POST: RequestHandler = async ({ request, cookies, url }) => {
	try {
		// Parse request body as OAuth2CallbackReq
		const body: OAuth2CallbackReq = await request.json();

		// Retrieve PKCE parameters from cookies (set during authorize step)
		const code_verifier = cookies.get('oauth_code_verifier');
		const stored_state = cookies.get('oauth_state');

		if (!code_verifier) {
			throw error(400, JSON.stringify({
				code: 'MISSING_CODE_VERIFIER',
				message: 'PKCE code_verifier not found in session'
			}));
		}

		// Verify state matches (CSRF protection)
		if (stored_state && body.state !== stored_state) {
			throw error(400, JSON.stringify({
				code: 'STATE_MISMATCH',
				message: 'OAuth state parameter mismatch'
			}));
		}

		// Include code_verifier in request to backend
		const callbackRequest: OAuth2CallbackReq = {
			...body,
			code_verifier
		};

		// Forward request to backend user-service
		const response = await fetch(`${USER_SERVICE_URL}/api/v1/auth/oauth/callback`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(callbackRequest)
		});

		// Clear PKCE cookies after use (single-use tokens)
		cookies.delete('oauth_code_verifier', { path: '/' });
		cookies.delete('oauth_state', { path: '/' });

		if (!response.ok) {
			const errorData = await response.json().catch(() => ({ message: 'Unknown error' }));
			throw error(response.status, JSON.stringify(errorData));
		}

		const data: OAuth2CallbackResp = await response.json();

		// Store tokens in httpOnly cookies if present
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

		// Store user and tenant information for client-side access
		if (data.user) {
			const userData = {
				kanidm_user_id: data.user.kanidm_user_id,
				email: data.user.email,
				preferred_username: data.user.preferred_username,
				groups: data.user.groups,
				tenant: data.tenant
					? {
							tenant_id: data.tenant.tenant_id,
							name: data.tenant.name,
							slug: data.tenant.slug,
							role: data.tenant.role
						}
					: undefined
			};

			// Store user data in a cookie that can be read by client
			cookies.set('user_data', JSON.stringify(userData), {
				path: '/',
				httpOnly: false, // Allow client-side access
				secure: true,
				sameSite: 'strict',
				maxAge: 30 * 24 * 60 * 60 // 30 days
			});
		}

		// Validate and sanitize redirect parameter (prevent open redirect)
		const redirectParam = url.searchParams.get('redirect');
		let redirectTo = '/dashboard'; // Default safe redirect

		if (redirectParam) {
			// Only allow internal paths (must start with / but not //)
			if (redirectParam.startsWith('/') && !redirectParam.startsWith('//')) {
				// Additional check: ensure it's not a protocol-relative URL
				try {
					const testUrl = new URL(redirectParam, 'http://localhost');
					if (testUrl.protocol === 'http:' || testUrl.protocol === 'https:') {
						redirectTo = redirectParam;
					}
				} catch {
					// Invalid URL, use default
					console.warn('Invalid redirect parameter, using default');
				}
			} else {
				console.warn('Rejected unsafe redirect parameter:', redirectParam);
			}
		}

		throw redirect(302, redirectTo);
	} catch (err) {
		console.error('OAuth2 callback error:', err);

		// Clear any stored tokens on error
		cookies.delete('access_token', { path: '/' });
		cookies.delete('refresh_token', { path: '/' });
		cookies.delete('user_data', { path: '/' });

		if (err && typeof err === 'object' && 'status' in err) {
			throw err; // Re-throw SvelteKit errors
		}

		throw error(
			500,
			JSON.stringify({
				code: 'OAUTH_CALLBACK_FAILED',
				message: 'Failed to complete OAuth2 callback'
			})
		);
	}
};
