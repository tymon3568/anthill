/**
 * Inventory Service API Proxy
 *
 * This proxy forwards all /api/v1/inventory/* requests to the Inventory Service.
 * It reads httpOnly cookies from the frontend domain and forwards them as
 * Authorization headers to the backend.
 *
 * Token Refresh: If token is expired or about to expire, this proxy will
 * automatically refresh it before forwarding the request.
 */
import { json, type RequestHandler } from '@sveltejs/kit';
import { PUBLIC_INVENTORY_SERVICE_URL, PUBLIC_USER_SERVICE_URL } from '$env/static/public';
import { dev } from '$app/environment';
import { shouldRefreshToken, isTokenExpired } from '$lib/auth/jwt';

const BACKEND_URL = PUBLIC_INVENTORY_SERVICE_URL;
const USER_SERVICE_URL = PUBLIC_USER_SERVICE_URL || 'http://localhost:8000';

/**
 * Attempt to refresh the access token using the refresh token
 */
async function refreshAccessToken(
	cookies: import('@sveltejs/kit').Cookies
): Promise<string | null> {
	const refreshToken = cookies.get('refresh_token');
	if (!refreshToken) {
		if (dev) console.log('[inventory-proxy] No refresh token available');
		return null;
	}

	try {
		if (dev) console.log('[inventory-proxy] Attempting token refresh...');

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

					if (dev) console.log('[inventory-proxy] Token refreshed successfully');

					// Also update refresh token if provided
					if (refreshData.refresh_token) {
						cookies.set('refresh_token', refreshData.refresh_token, {
							path: '/',
							httpOnly: true,
							secure: !dev,
							sameSite: 'lax',
							maxAge: 30 * 24 * 60 * 60
						});
					}

					return refreshData.access_token;
				}
			}
		} else {
			const errorText = await refreshResponse.text().catch(() => 'Unknown error');
			if (dev) {
				console.log(
					`[inventory-proxy] Token refresh failed: ${refreshResponse.status} - ${errorText}`
				);
			}

			// If refresh token is invalid/expired, clear cookies
			if (refreshResponse.status === 401 || refreshResponse.status === 403) {
				cookies.delete('access_token', { path: '/' });
				cookies.delete('refresh_token', { path: '/' });
			}
		}
	} catch (error) {
		console.error('[inventory-proxy] Token refresh error:', error);
	}

	return null;
}

async function proxyRequest(
	request: Request,
	cookies: import('@sveltejs/kit').Cookies,
	method: string,
	path: string,
	url: URL
): Promise<Response> {
	try {
		// Get access token from httpOnly cookie
		let accessToken = cookies.get('access_token');
		const refreshToken = cookies.get('refresh_token');

		// Debug logging in development
		if (dev) {
			console.log(
				`[inventory-proxy] ${method} /api/v1/inventory/${path} - token: ${accessToken ? 'present' : 'missing'}`
			);
		}

		// Check if token needs refresh (expired or about to expire)
		if (accessToken && (isTokenExpired(accessToken) || shouldRefreshToken(accessToken))) {
			if (dev) console.log('[inventory-proxy] Token expired or expiring soon, refreshing...');
			const newToken = await refreshAccessToken(cookies);
			if (newToken) {
				accessToken = newToken;
			} else {
				// Token refresh failed
				if (dev) console.log('[inventory-proxy] Token refresh failed, continuing with old token');
			}
		}

		// If no access token but have refresh token, try to get a new one
		if (!accessToken && refreshToken) {
			if (dev) console.log('[inventory-proxy] No access token, attempting refresh...');
			const refreshedToken = await refreshAccessToken(cookies);
			if (refreshedToken) {
				accessToken = refreshedToken;
			}
		}

		// If still no access token, return 401
		if (!accessToken) {
			return json(
				{
					error: 'Authentication required',
					code: 'NO_SESSION',
					message: 'Please log in to access this resource'
				},
				{ status: 401 }
			);
		}

		// Build headers
		const headers: HeadersInit = {};

		// Forward Content-Type if present
		const contentType = request.headers.get('Content-Type');
		if (contentType) {
			headers['Content-Type'] = contentType;
		}

		// Forward X-Tenant-ID if present
		const tenantId = request.headers.get('X-Tenant-ID');
		if (tenantId) {
			headers['X-Tenant-ID'] = tenantId;
		}

		// Forward X-Idempotency-Key if present (required for POST requests)
		const idempotencyKey = request.headers.get('X-Idempotency-Key');
		if (idempotencyKey) {
			headers['X-Idempotency-Key'] = idempotencyKey;
		}

		// Add Authorization header from cookie
		headers['Authorization'] = `Bearer ${accessToken}`;

		// Build request options
		const options: RequestInit = {
			method,
			headers
		};

		// Forward body for non-GET requests
		let requestBody: ArrayBuffer | undefined;
		if (method !== 'GET' && method !== 'HEAD') {
			requestBody = await request.arrayBuffer();
			if (requestBody.byteLength > 0) {
				options.body = requestBody;
			}
		}

		// Forward to inventory backend - include query string if present
		const queryString = url.search;
		const backendUrl = `${BACKEND_URL}/api/v1/inventory/${path}${queryString}`;

		if (dev) {
			console.log(`[inventory-proxy] Forwarding to: ${backendUrl}`);
		}

		let response = await fetch(backendUrl, options);

		// If we get 401 (token expired on backend), try to refresh and retry once
		if (response.status === 401) {
			if (dev) console.log('[inventory-proxy] Got 401, attempting token refresh and retry...');

			const newToken = await refreshAccessToken(cookies);
			if (newToken) {
				// Retry with new token
				headers['Authorization'] = `Bearer ${newToken}`;

				const retryOptions: RequestInit = {
					method,
					headers
				};

				if (requestBody && requestBody.byteLength > 0) {
					retryOptions.body = requestBody;
				}

				response = await fetch(backendUrl, retryOptions);

				if (dev) {
					console.log(`[inventory-proxy] Retry response status: ${response.status}`);
				}
			} else {
				// Refresh failed, return 401 with clear message
				return json(
					{
						error: 'Session expired',
						code: 'SESSION_EXPIRED',
						message: 'Your session has expired. Please log in again.'
					},
					{ status: 401 }
				);
			}
		}

		// Handle 204 No Content
		if (response.status === 204) {
			return new Response(null, { status: 204, headers: response.headers });
		}

		// Get response body
		const responseText = await response.text();

		if (dev && response.status >= 400) {
			console.log(
				`[inventory-proxy] Response ${response.status}: ${responseText.substring(0, 200)}`
			);
		}

		// Return response with same status and headers
		return new Response(responseText, {
			status: response.status,
			headers: {
				'Content-Type': response.headers.get('Content-Type') || 'application/json'
			}
		});
	} catch (error) {
		console.error('Inventory API proxy error:', error);
		console.error('Error details:', {
			name: error instanceof Error ? error.name : 'unknown',
			message: error instanceof Error ? error.message : String(error),
			cause: error instanceof Error ? (error as Error & { cause?: unknown }).cause : undefined
		});
		return json({ error: 'Internal server error' }, { status: 500 });
	}
}

export const GET: RequestHandler = async ({ request, cookies, params, url }) => {
	const path = params.path || '';
	return proxyRequest(request, cookies, 'GET', path, url);
};

export const POST: RequestHandler = async ({ request, cookies, params, url }) => {
	const path = params.path || '';
	return proxyRequest(request, cookies, 'POST', path, url);
};

export const PUT: RequestHandler = async ({ request, cookies, params, url }) => {
	const path = params.path || '';
	return proxyRequest(request, cookies, 'PUT', path, url);
};

export const PATCH: RequestHandler = async ({ request, cookies, params, url }) => {
	const path = params.path || '';
	return proxyRequest(request, cookies, 'PATCH', path, url);
};

export const DELETE: RequestHandler = async ({ request, cookies, params, url }) => {
	const path = params.path || '';
	return proxyRequest(request, cookies, 'DELETE', path, url);
};
