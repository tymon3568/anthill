import { redirect, json } from '@sveltejs/kit';
import { validateAndParseToken, shouldRefreshToken } from '$lib/auth/jwt';
import { handleAuthError, createAuthError, AuthErrorCode } from '$lib/auth/errors';
import type { Handle } from '@sveltejs/kit';
import { dev } from '$app/environment';
import { parseTenantFromHostname } from '$lib/tenant';
import { env } from '$env/dynamic/public';

// Check if request is an API route (should return JSON errors, not redirects)
function isApiRoute(pathname: string): boolean {
	return pathname.startsWith('/api/');
}

// Get backend URL
const USER_SERVICE_URL = env.PUBLIC_USER_SERVICE_URL || 'http://localhost:8000';

// Public routes that don't require authentication
const publicRoutes = [
	'/login',
	'/register',
	'/forgot-password',
	'/reset-password',
	'/verify-email'
];

// Public API routes (exact paths only - use Set for O(1) lookup)
const publicApiRoutes = new Set([
	'/api/v1/auth/login',
	'/api/v1/auth/register',
	'/api/v1/auth/refresh',
	'/api/v1/auth/oauth/authorize',
	'/api/v1/auth/oauth/callback',
	'/api/v1/auth/oauth/refresh',
	'/api/v1/auth/forgot-password',
	'/api/v1/auth/validate-reset-token',
	'/api/v1/auth/reset-password',
	'/api/v1/auth/resend-verification',
	'/api/v1/auth/verify-email'
]);

function isPublicRoute(pathname: string): boolean {
	// Exact match for root
	if (pathname === '/') return true;

	// Check public routes (prefix matching for pages)
	if (publicRoutes.some((route) => pathname.startsWith(route))) return true;

	// Check public API routes (exact matching only)
	if (publicApiRoutes.has(pathname)) return true;

	return false;
}

export const handle: Handle = async ({ event, resolve }) => {
	const { url, cookies, locals, request } = event;
	const pathname = url.pathname;

	// Debug logging in development
	if (dev && !pathname.startsWith('/_app/') && !pathname.startsWith('/favicon')) {
		console.log(
			`[hooks] ${pathname} - cookies: access_token=${cookies.get('access_token') ? 'present' : 'missing'}, refresh_token=${cookies.get('refresh_token') ? 'present' : 'missing'}`
		);
	}

	// Detect tenant from subdomain or X-Tenant-ID header
	const host = request.headers.get('host') || url.host;
	// Remove port before parsing hostname
	const hostname = host.split(':')[0];
	const headerTenantId = request.headers.get('X-Tenant-ID');
	const subdomainTenant = parseTenantFromHostname(hostname);

	// Priority: X-Tenant-ID header > subdomain
	const tenantSlug = headerTenantId || subdomainTenant;

	// Store tenant context in locals for use in load functions
	locals.tenantSlug = tenantSlug;

	// Resolve response first
	const response = await (async () => {
		// Skip auth check for public routes and static assets
		if (
			isPublicRoute(pathname) ||
			pathname.startsWith('/favicon') ||
			pathname.startsWith('/_app/')
		) {
			return resolve(event);
		}

		// Get access token from httpOnly cookie
		let accessToken = cookies.get('access_token');
		const refreshToken = cookies.get('refresh_token');

		// If no access token but refresh token exists, try to refresh first
		if (!accessToken && refreshToken) {
			try {
				// Call backend directly (not through frontend API) to avoid cookie issues
				const refreshResponse = await fetch(`${USER_SERVICE_URL}/api/v1/auth/refresh`, {
					method: 'POST',
					headers: {
						'Content-Type': 'application/json'
					},
					body: JSON.stringify({ refresh_token: refreshToken })
				});

				if (refreshResponse.ok) {
					const contentType = refreshResponse.headers.get('content-type');
					if (contentType?.includes('application/json')) {
						const refreshData = await refreshResponse.json();
						// Update cookies with new tokens
						if (refreshData.access_token) {
							cookies.set('access_token', refreshData.access_token, {
								path: '/',
								httpOnly: true,
								secure: !dev,
								sameSite: 'lax',
								maxAge: refreshData.expires_in || 3600
							});
							accessToken = refreshData.access_token;
						}

						if (refreshData.refresh_token) {
							cookies.set('refresh_token', refreshData.refresh_token, {
								path: '/',
								httpOnly: true,
								secure: !dev,
								sameSite: 'lax',
								maxAge: 30 * 24 * 60 * 60
							});
						}
					}
				} else {
					// Refresh failed - log error
					if (dev) {
						const errorText = await refreshResponse.text().catch(() => 'Unknown error');
						console.log(
							`[hooks] Refresh failed with status ${refreshResponse.status}: ${errorText}`
						);
					}
				}
			} catch (error) {
				console.error('[hooks] Token refresh failed:', error);
			}
		}

		if (!accessToken) {
			// No token and refresh failed
			if (isApiRoute(pathname)) {
				// API routes: return JSON error
				return json({ error: 'Authentication required', code: 'NO_SESSION' }, { status: 401 });
			}
			// Page routes: redirect to login
			throw redirect(302, `/login?redirect=${encodeURIComponent(pathname)}`);
		}

		// Check if token needs refresh (about to expire)
		let currentAccessToken = accessToken;
		if (shouldRefreshToken(accessToken) && refreshToken) {
			try {
				// Call backend directly
				const refreshResponse = await fetch(`${USER_SERVICE_URL}/api/v1/auth/refresh`, {
					method: 'POST',
					headers: {
						'Content-Type': 'application/json'
					},
					body: JSON.stringify({ refresh_token: refreshToken })
				});

				if (refreshResponse.ok) {
					const contentType = refreshResponse.headers.get('content-type');
					if (contentType?.includes('application/json')) {
						const refreshData = await refreshResponse.json();
						// Update cookies with new tokens
						if (refreshData.access_token) {
							cookies.set('access_token', refreshData.access_token, {
								path: '/',
								httpOnly: true,
								secure: !dev,
								sameSite: 'lax',
								maxAge: refreshData.expires_in || 3600
							});
							currentAccessToken = refreshData.access_token;
						}

						if (refreshData.refresh_token) {
							cookies.set('refresh_token', refreshData.refresh_token, {
								path: '/',
								httpOnly: true,
								secure: !dev,
								sameSite: 'lax',
								maxAge: 30 * 24 * 60 * 60
							});
						}
					}
				}
				// If refresh fails, continue with existing token (it might still be valid)
			} catch (error) {
				// Log error but continue with existing token
				if (dev) {
					console.error('[hooks] Token refresh error:', error);
				}
			}
		}

		// Validate token and extract user info
		// Note: We use verifySignature=false because tokens are from our own backend
		// The backend already verified credentials and signed the token with JWT_SECRET
		// Server-side signature verification would require sharing JWT_SECRET with frontend
		// which is not recommended. Instead, we trust the httpOnly cookie mechanism.
		const userInfo = await validateAndParseToken(currentAccessToken, false);
		if (!userInfo) {
			// Invalid token
			cookies.delete('access_token', { path: '/' });
			cookies.delete('refresh_token', { path: '/' });

			if (isApiRoute(pathname)) {
				// API routes: return JSON error
				return json(
					{ error: 'Invalid authentication token', code: 'INVALID_TOKEN' },
					{ status: 401 }
				);
			}
			// Page routes: redirect to login
			handleAuthError(
				createAuthError(AuthErrorCode.INVALID_TOKEN),
				`/login?redirect=${encodeURIComponent(pathname)}`
			);
		}

		// Store user info in locals for use in load functions and endpoints
		locals.user = userInfo;

		// Continue with request
		return resolve(event);
	})();

	// Set CSP header based on environment
	const csp = dev
		? // Development CSP - more permissive
			[
				"default-src 'self'",
				"script-src 'self' 'unsafe-inline' 'unsafe-eval'",
				"style-src 'self' 'unsafe-inline'",
				"img-src 'self' data: https:",
				"font-src 'self' data:",
				"connect-src 'self' https: http://localhost:8000 http://127.0.0.1:8000 ws://localhost:5173 ws://localhost:5174 wss:",
				"object-src 'none'",
				"base-uri 'self'",
				"form-action 'self'"
			].join('; ')
		: // Production CSP - strict
			[
				"default-src 'self'",
				"script-src 'self' 'unsafe-inline'", // SvelteKit needs unsafe-inline for hydration
				"style-src 'self' 'unsafe-inline'",
				"img-src 'self' data: https:",
				"font-src 'self' data:",
				"connect-src 'self' https: wss:",
				"object-src 'none'",
				"base-uri 'self'",
				"form-action 'self'"
			].join('; ');

	response.headers.set('Content-Security-Policy', csp);

	return response;
};
