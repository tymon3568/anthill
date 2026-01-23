/**
 * Catch-all API Proxy
 *
 * This proxy forwards all API requests to the backend with proper authentication.
 * It reads httpOnly cookies from the frontend domain and forwards them as
 * Authorization headers to the backend.
 *
 * This solves the cross-origin cookie issue where:
 * - Cookies are set on frontend domain (localhost:5173)
 * - Backend is on a different origin (localhost:8000)
 * - Browser doesn't send cookies cross-origin without proper CORS setup
 *
 * By proxying through SvelteKit, we can:
 * 1. Read cookies from the request (same-origin)
 * 2. Forward the access_token as Authorization header to backend
 * 3. Return the response to the client
 */
import { json, type RequestHandler } from '@sveltejs/kit';
import { PUBLIC_USER_SERVICE_URL } from '$env/static/public';
import { dev } from '$app/environment';

const BACKEND_URL = PUBLIC_USER_SERVICE_URL;

async function proxyRequest(
	request: Request,
	cookies: import('@sveltejs/kit').Cookies,
	method: string,
	path: string,
	url: URL
): Promise<Response> {
	try {
		// Get access token from httpOnly cookie
		const accessToken = cookies.get('access_token');

		// Debug logging in development
		if (dev) {
			console.log(
				`[proxy] ${method} /api/v1/${path} - token: ${accessToken ? 'present' : 'missing'}`
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

		// Add Authorization header from cookie
		if (accessToken) {
			headers['Authorization'] = `Bearer ${accessToken}`;
		}

		// Build request options
		const options: RequestInit = {
			method,
			headers
		};

		// Forward body for non-GET requests
		if (method !== 'GET' && method !== 'HEAD') {
			// For multipart form data (file uploads), forward the raw body
			// Using arrayBuffer() preserves binary data correctly
			const body = await request.arrayBuffer();
			if (body.byteLength > 0) {
				options.body = body;
			}
		}

		// Forward to backend - include query string if present
		const queryString = url.search; // includes the '?' if present
		const backendUrl = `${BACKEND_URL}/api/v1/${path}${queryString}`;

		if (dev) {
			console.log(`[proxy] Forwarding to: ${backendUrl}`);
			console.log(`[proxy] Headers:`, JSON.stringify(headers));
		}

		const response = await fetch(backendUrl, options);

		// Handle 204 No Content - return empty response without body
		if (response.status === 204) {
			if (dev) {
				console.log(`[proxy] Response 204 No Content`);
			}
			return new Response(null, { status: 204 });
		}

		// Get response body
		const responseText = await response.text();

		if (dev && response.status >= 400) {
			console.log(`[proxy] Response ${response.status}: ${responseText.substring(0, 200)}`);
		}

		// Return response with same status and headers
		return new Response(responseText, {
			status: response.status,
			headers: {
				'Content-Type': response.headers.get('Content-Type') || 'application/json'
			}
		});
	} catch (error) {
		console.error('API proxy error:', error);
		return json({ error: 'Internal server error' }, { status: 500 });
	}
}

export const GET: RequestHandler = async ({ request, cookies, params, url }) => {
	const path = params.path || '';
	console.log(`[catch-all proxy] GET received for path: ${path}, full URL: ${url.pathname}`);
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
