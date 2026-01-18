import { PUBLIC_API_BASE_URL } from '$env/static/public';
import type { ApiResponse } from '$lib/types';
import { tokenManager } from '$lib/auth/token-manager';
import { getCurrentTenantSlug } from '$lib/tenant';

// Base API configuration
const API_BASE_URL = PUBLIC_API_BASE_URL;

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
			'Content-Type': 'application/json',
			...(options.headers as Record<string, string>)
		};

		// Add X-Tenant-ID header if tenant context is available
		// Priority: 1. Explicitly set tenant, 2. Auto-detected from subdomain/storage
		const tenantSlug = this.tenantSlug ?? getCurrentTenantSlug();
		if (tenantSlug) {
			headers['X-Tenant-ID'] = tenantSlug;
		}

		const config: RequestInit = {
			headers,
			...options
		};

		// Add auth token if available, but NOT for auth endpoints that don't require authentication
		const isAuthEndpoint =
			endpoint.startsWith('/auth/login') ||
			endpoint.startsWith('/auth/register') ||
			endpoint.startsWith('/auth/oauth/authorize') ||
			endpoint.startsWith('/auth/oauth/callback') ||
			endpoint.startsWith('/auth/refresh') ||
			endpoint.startsWith('/auth/oauth/refresh');

		if (!isAuthEndpoint) {
			const token = tokenManager.getAccessToken();
			if (token) {
				(config.headers as Record<string, string>)['Authorization'] = `Bearer ${token}`;
			}
		}

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
						tokenManager.clearAll();
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
				data = await response.json();
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
		return this.request<T>(endpoint, {
			method: 'POST',
			body: data ? JSON.stringify(data) : undefined,
			headers: options?.headers
		});
	}

	async put<T>(endpoint: string, data?: Record<string, unknown>): Promise<ApiResponse<T>> {
		return this.request<T>(endpoint, {
			method: 'PUT',
			body: data ? JSON.stringify(data) : undefined
		});
	}

	async patch<T>(endpoint: string, data?: Record<string, unknown>): Promise<ApiResponse<T>> {
		return this.request<T>(endpoint, {
			method: 'PATCH',
			body: data ? JSON.stringify(data) : undefined
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
