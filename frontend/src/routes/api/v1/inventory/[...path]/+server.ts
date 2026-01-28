/**
 * Inventory Service API Proxy
 *
 * This proxy forwards all /api/v1/inventory/* requests to the Inventory Service.
 * It reads httpOnly cookies from the frontend domain and forwards them as
 * Authorization headers to the backend.
 */
import { json, type RequestHandler } from '@sveltejs/kit';
import { PUBLIC_INVENTORY_SERVICE_URL } from '$env/static/public';
import { dev } from '$app/environment';

const BACKEND_URL = PUBLIC_INVENTORY_SERVICE_URL;

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
				`[inventory-proxy] ${method} /api/v1/inventory/${path} - token: ${accessToken ? 'present' : 'missing'}`
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
			const body = await request.arrayBuffer();
			if (body.byteLength > 0) {
				options.body = body;
			}
		}

		// Forward to inventory backend - include query string if present
		const queryString = url.search;
		const backendUrl = `${BACKEND_URL}/api/v1/inventory/${path}${queryString}`;

		if (dev) {
			console.log(`[inventory-proxy] Forwarding to: ${backendUrl}`);
		}

		const response = await fetch(backendUrl, options);

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
