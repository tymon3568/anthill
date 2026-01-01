import { redirect } from '@sveltejs/kit';
import { validateAndParseToken, shouldRefreshToken } from '$lib/auth/jwt';
import { handleAuthError, createAuthError, AuthErrorCode } from '$lib/auth/errors';
import type { Handle } from '@sveltejs/kit';
import { dev } from '$app/environment';

/**
 * Parse tenant slug from hostname/subdomain
 *
 * Examples:
 * - acme.localhost:5173 -> "acme"
 * - acme.anthill.example.com -> "acme"
 * - localhost:5173 -> null
 */
function parseTenantFromHost(host: string): string | null {
	// Remove port if present
	const hostname = host.split(':')[0];

	// Handle localhost specifically
	if (hostname === 'localhost' || hostname === '127.0.0.1') {
		return null;
	}

	// Check for *.localhost pattern (e.g., acme.localhost)
	if (hostname.endsWith('.localhost')) {
		const subdomain = hostname.replace('.localhost', '');
		if (subdomain && subdomain !== 'www') {
			return subdomain;
		}
		return null;
	}

	// For production domains (tenant.domain.tld)
	const parts = hostname.split('.');
	if (parts.length >= 3) {
		const subdomain = parts[0];
		// Ignore www subdomain
		if (subdomain && subdomain !== 'www') {
			return subdomain;
		}
	}

	return null;
}

// Protected routes that require authentication
const protectedRoutes = ['/dashboard', '/inventory', '/orders', '/settings', '/profile'];

// Public routes that don't require authentication
const publicRoutes = ['/login', '/register'];

// Public API routes (exact paths only - use Set for O(1) lookup)
const publicApiRoutes = new Set([
	'/api/v1/auth/login',
	'/api/v1/auth/register',
	'/api/v1/auth/oauth/authorize',
	'/api/v1/auth/oauth/callback',
	'/api/v1/auth/oauth/refresh'
]);

function isProtectedRoute(pathname: string): boolean {
	return protectedRoutes.some((route) => pathname.startsWith(route));
}

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

	// Detect tenant from subdomain or X-Tenant-ID header
	const host = request.headers.get('host') || url.host;
	const headerTenantId = request.headers.get('x-tenant-id');
	const subdomainTenant = parseTenantFromHost(host);

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
		const accessToken = cookies.get('access_token');

		if (!accessToken) {
			// No token, redirect to login
			throw redirect(302, `/login?redirect=${encodeURIComponent(pathname)}`);
		}

		// Check if token needs refresh
		let currentAccessToken = accessToken;
		if (shouldRefreshToken(accessToken)) {
			try {
				// Attempt to refresh token
				const refreshToken = cookies.get('refresh_token');
				if (refreshToken) {
					const refreshResponse = await fetch(`${url.origin}/api/v1/auth/oauth/refresh`, {
						method: 'POST',
						headers: {
							Cookie: `refresh_token=${refreshToken}`
						}
					});

					if (refreshResponse.ok) {
						const refreshData = await refreshResponse.json();
						// Update cookies with new tokens
						cookies.set('access_token', refreshData.access_token, {
							path: '/',
							httpOnly: true,
							secure: true,
							sameSite: 'strict',
							maxAge: refreshData.expires_in || 3600
						});

						// Update refresh_token cookie if rotated by backend
						if (refreshData.refresh_token) {
							cookies.set('refresh_token', refreshData.refresh_token, {
								path: '/',
								httpOnly: true,
								secure: true,
								sameSite: 'strict',
								maxAge: 30 * 24 * 60 * 60 // 30 days default
							});
						}

						// Use the new access token for validation
						currentAccessToken = refreshData.access_token;
					} else {
						// Refresh failed, redirect to login
						handleAuthError(
							createAuthError(AuthErrorCode.SESSION_EXPIRED),
							`/login?redirect=${encodeURIComponent(pathname)}`
						);
					}
				} else {
					// No refresh token, redirect to login
					handleAuthError(
						createAuthError(AuthErrorCode.NO_SESSION),
						`/login?redirect=${encodeURIComponent(pathname)}`
					);
				}
			} catch (error) {
				// Refresh failed, redirect to login
				handleAuthError(
					createAuthError(AuthErrorCode.REFRESH_FAILED),
					`/login?redirect=${encodeURIComponent(pathname)}`
				);
			}
		}

		// Validate token and extract user info
		const userInfo = await validateAndParseToken(currentAccessToken, true); // Enable signature verification server-side
		if (!userInfo) {
			// Invalid token, redirect to login
			cookies.delete('access_token', { path: '/' });
			cookies.delete('refresh_token', { path: '/' });
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
