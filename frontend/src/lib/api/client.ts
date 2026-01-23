import type { ApiResponse } from '$lib/types';
import { getCurrentTenantSlug } from '$lib/tenant';

// Helper function to convert snake_case to camelCase
function snakeToCamel(str: string): string {
	return str.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
}

// Recursively transform all keys in an object from snake_case to camelCase
export function transformKeysToCamelCase<T>(obj: unknown): T {
	if (Array.isArray(obj)) {
		return obj.map(transformKeysToCamelCase) as T;
	}
	if (obj !== null && typeof obj === 'object' && !(obj instanceof Date)) {
		return Object.fromEntries(
			Object.entries(obj as Record<string, unknown>).map(([key, value]) => [
				snakeToCamel(key),
				transformKeysToCamelCase(value)
			])
		) as T;
	}
	return obj as T;
}

// Helper function to convert camelCase to snake_case
function camelToSnake(str: string): string {
	return str.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
}

// Recursively transform all keys in an object from camelCase to snake_case
function transformKeysToSnakeCase<T>(obj: unknown): T {
	if (Array.isArray(obj)) {
		return obj.map(transformKeysToSnakeCase) as T;
	}
	if (obj !== null && typeof obj === 'object' && !(obj instanceof Date)) {
		return Object.fromEntries(
			Object.entries(obj as Record<string, unknown>).map(([key, value]) => [
				camelToSnake(key),
				transformKeysToSnakeCase(value)
			])
		) as T;
	}
	return obj as T;
}

// Base API configuration
// Use relative URL to route through SvelteKit proxy for proper cookie handling
// The proxy at /api/v1/[...path] forwards requests to the backend with auth headers
const API_BASE_URL = '/api/v1';

// Generic API client
class ApiClient {
	private baseURL: string;
	private tenantSlug: string | null = null;

	constructor(baseURL: string = API_BASE_URL) {
		this.baseURL = baseURL;
	}

	/**
	 * Set tenant slug for API requests
	 * This will be sent as X-Tenant-ID header
	 */
	setTenantSlug(slug: string | null): void {
		this.tenantSlug = slug;
	}

	/**
	 * Get current tenant slug
	 */
	getTenantSlug(): string | null {
		return this.tenantSlug;
	}

	private async request<T>(endpoint: string, options: RequestInit = {}): Promise<ApiResponse<T>> {
		const url = `${this.baseURL}${endpoint}`;

		// Build headers with tenant context
		const headers: Record<string, string> = {
			...(options.headers as Record<string, string>)
		};

		// Only set Content-Type for requests with a body
		// This prevents sending Content-Type: application/json for empty POST requests
		if (options.body !== undefined) {
			headers['Content-Type'] = 'application/json';
		}

		// Add X-Tenant-ID header if tenant context is available
		// Priority: 1. Explicitly set tenant, 2. Auto-detected from subdomain/storage
		const tenantSlug = this.tenantSlug ?? getCurrentTenantSlug();
		if (tenantSlug) {
			headers['X-Tenant-ID'] = tenantSlug;
		}

		// Build config without headers from options (we already merged them above)
		const { headers: _optHeaders, ...restOptions } = options;
		const config: RequestInit = {
			...restOptions,
			headers,
			// SECURITY: Use credentials: 'include' to send httpOnly cookies
			// This allows the browser to automatically send auth cookies set by the backend
			credentials: 'include'
		};

		// Check if this is an auth endpoint (for error handling purposes only)
		// These endpoints don't require authentication, so 401 responses are
		// invalid credentials, not session expiry
		const isAuthEndpoint =
			endpoint.startsWith('/auth/login') ||
			endpoint.startsWith('/auth/register') ||
			endpoint.startsWith('/auth/oauth/authorize') ||
			endpoint.startsWith('/auth/oauth/callback') ||
			endpoint.startsWith('/auth/refresh') ||
			endpoint.startsWith('/auth/oauth/refresh') ||
			endpoint.startsWith('/auth/forgot-password') ||
			endpoint.startsWith('/auth/validate-reset-token') ||
			endpoint.startsWith('/auth/reset-password') ||
			endpoint.startsWith('/auth/resend-verification') ||
			endpoint.startsWith('/auth/verify-email');

		// NOTE: We no longer manually add Authorization headers
		// Authentication is handled via httpOnly cookies set by the backend
		// The browser automatically includes cookies with requests when credentials: 'include' is set

		try {
			const response = await fetch(url, config);

			if (!response.ok) {
				// Parse error response first
				const errorData = await response.json().catch(() => ({
					message: 'Network error',
					error: 'Network error'
				}));

				if (response.status === 401) {
					// Only treat as session expired if NOT an auth endpoint
					// For auth endpoints, this is just invalid credentials
					if (!isAuthEndpoint) {
						// Session expired - redirect to login
						// The backend will have already cleared the cookies via logout
						// or the cookies expired naturally
						if (typeof window !== 'undefined') {
							window.location.href = '/login?error=session_expired';
						}
						return {
							success: false,
							error: 'Session expired'
						};
					}
					// For auth endpoints, return the actual error message
					return {
						success: false,
						error: errorData.error || errorData.message || 'Authentication failed'
					};
				}

				return {
					success: false,
					error: errorData.error || errorData.message || `HTTP ${response.status}`
				};
			}

			// Handle 204 No Content or empty responses
			if (response.status === 204 || response.headers.get('content-length') === '0') {
				return { success: true };
			}

			// Parse JSON only if content-type indicates JSON
			const contentType = response.headers.get('content-type') ?? '';
			let data: T;
			if (contentType.includes('application/json')) {
				// Transform snake_case keys from backend to camelCase for frontend
				const rawData = await response.json();
				data = transformKeysToCamelCase<T>(rawData);
			} else {
				data = (await response.text()) as unknown as T;
			}

			return {
				success: true,
				data
			};
		} catch (error) {
			return {
				success: false,
				error: error instanceof Error ? error.message : 'Unknown error'
			};
		}
	}

	async get<T>(endpoint: string): Promise<ApiResponse<T>> {
		return this.request<T>(endpoint, { method: 'GET' });
	}

	async post<T>(
		endpoint: string,
		data?: Record<string, unknown>,
		options?: { headers?: Record<string, string> }
	): Promise<ApiResponse<T>> {
		// Transform camelCase keys to snake_case for backend
		const transformedData = data ? transformKeysToSnakeCase(data) : undefined;
		return this.request<T>(endpoint, {
			method: 'POST',
			body: transformedData ? JSON.stringify(transformedData) : undefined,
			headers: options?.headers
		});
	}

	async put<T>(endpoint: string, data?: Record<string, unknown>): Promise<ApiResponse<T>> {
		// Transform camelCase keys to snake_case for backend
		const transformedData = data ? transformKeysToSnakeCase(data) : undefined;
		return this.request<T>(endpoint, {
			method: 'PUT',
			body: transformedData ? JSON.stringify(transformedData) : undefined
		});
	}

	async patch<T>(endpoint: string, data?: Record<string, unknown>): Promise<ApiResponse<T>> {
		// Transform camelCase keys to snake_case for backend
		const transformedData = data ? transformKeysToSnakeCase(data) : undefined;
		return this.request<T>(endpoint, {
			method: 'PATCH',
			body: transformedData ? JSON.stringify(transformedData) : undefined
		});
	}

	async delete<T>(endpoint: string): Promise<ApiResponse<T>> {
		return this.request<T>(endpoint, { method: 'DELETE' });
	}
}

// Export singleton instance
export const apiClient = new ApiClient();

// Pagination helper
export function createPaginationParams(page: number = 1, limit: number = 10) {
	return new URLSearchParams({
		page: page.toString(),
		limit: limit.toString()
	});
}
