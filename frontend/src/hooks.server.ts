import { redirect } from '@sveltejs/kit';
import { validateAndParseToken, shouldRefreshToken } from '$lib/auth/jwt';
import { handleAuthError, createAuthError, AuthErrorCode } from '$lib/auth/errors';
import type { Handle } from '@sveltejs/kit';

// Protected routes that require authentication
const protectedRoutes = [
	'/dashboard',
	'/inventory',
	'/orders',
	'/settings',
	'/profile'
];

// Public routes that don't require authentication
const publicRoutes = [
	'/',
	'/login',
	'/register',
	'/api/v1/auth'
];

function isProtectedRoute(pathname: string): boolean {
	return protectedRoutes.some(route => pathname.startsWith(route));
}

function isPublicRoute(pathname: string): boolean {
	return publicRoutes.some(route => pathname.startsWith(route)) ||
		   pathname === '/' ||
		   pathname.startsWith('/api/');
}

export const handle: Handle = async ({ event, resolve }) => {
	const { url, cookies, locals } = event;
	const pathname = url.pathname;

	// Skip auth check for public routes and static assets
	if (isPublicRoute(pathname) || pathname.startsWith('/favicon') || pathname.startsWith('/_app/')) {
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
						'Cookie': `refresh_token=${refreshToken}`
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
						maxAge: refreshData.expires_in || 3600,
					});
					// Use the new access token for validation
					currentAccessToken = refreshData.access_token;
				} else {
					// Refresh failed, redirect to login
					handleAuthError(createAuthError(AuthErrorCode.SESSION_EXPIRED), `/login?redirect=${encodeURIComponent(pathname)}`);
				}
			} else {
				// No refresh token, redirect to login
				handleAuthError(createAuthError(AuthErrorCode.NO_SESSION), `/login?redirect=${encodeURIComponent(pathname)}`);
			}
		} catch (error) {
			// Refresh failed, redirect to login
			handleAuthError(createAuthError(AuthErrorCode.REFRESH_FAILED), `/login?redirect=${encodeURIComponent(pathname)}`);
		}
	}

	// Validate token and extract user info
	const userInfo = await validateAndParseToken(currentAccessToken, true); // Enable signature verification server-side
	if (!userInfo) {
		// Invalid token, redirect to login
		cookies.delete('access_token', { path: '/' });
		cookies.delete('refresh_token', { path: '/' });
		handleAuthError(createAuthError(AuthErrorCode.INVALID_TOKEN), `/login?redirect=${encodeURIComponent(pathname)}`);
	}

	// Store user info in locals for use in load functions and endpoints
	locals.user = userInfo;

	// Continue with request
	return resolve(event);
};
