import { PUBLIC_API_BASE_URL } from '$env/static/public';
import type { ApiResponse } from '$lib/types';
import { tokenManager } from '$lib/auth/token-manager';
// Base API configuration
const API_BASE_URL = PUBLIC_API_BASE_URL;

// Track if we're currently refreshing to avoid duplicate refresh calls
let isRefreshing = false;
let refreshPromise: Promise<string | null> | null = null;
let refreshSubscribers: Array<(token: string) => void> = [];

// Subscribe to token refresh completion
function subscribeTokenRefresh(cb: (token: string) => void) {
	refreshSubscribers.push(cb);
}

// Notify all subscribers when refresh completes
function onTokenRefreshed(token: string) {
	refreshSubscribers.forEach(cb => cb(token));
	refreshSubscribers = [];
}

// Generic API client
class ApiClient {
	private baseURL: string;

	constructor(baseURL: string = API_BASE_URL) {
		this.baseURL = baseURL;
	}

	private async refreshAccessToken(): Promise<string | null> {
		// If refresh is already in progress, return the existing promise
		if (refreshPromise) {
			return refreshPromise;
		}

		// Start new refresh
		refreshPromise = this.performTokenRefresh();
		try {
			return await refreshPromise;
		} finally {
			// Clean up after refresh completes
			refreshPromise = null;
		}
	}

	private async performTokenRefresh(): Promise<string | null> {
		const refreshToken = await tokenManager.getRefreshToken();
		if (!refreshToken) return null;

		try {
			// Direct API call to avoid circular dependency
			const response = await fetch(`${this.baseURL}/auth/refresh`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({ refresh_token: refreshToken })
			});

			if (!response.ok) {
				throw new Error(`HTTP ${response.status}`);
			}

			const result = await response.json();
			if (result.success && result.data) {
				const expiresIn = result.data.expires_in || 900; // Default 15 minutes
				tokenManager.setAccessToken(result.data.access_token, expiresIn);
				await tokenManager.setRefreshToken(result.data.refresh_token);
				return result.data.access_token;
			}
		} catch (error) {
			console.error('Token refresh failed:', error);
			// Clear invalid tokens
			tokenManager.clearAll();
		}
		return null;
	}

	private async request<T>(endpoint: string, options: RequestInit = {}): Promise<ApiResponse<T>> {
		const url = `${this.baseURL}${endpoint}`;

		const config: RequestInit = {
			headers: {
				'Content-Type': 'application/json',
				...options.headers
			},
			...options
		};

		// Create AbortController for timeout
		const controller = new AbortController();
		const timeoutId = setTimeout(() => controller.abort(), 5000); // 5 second timeout

		config.signal = controller.signal;

		// Check if token needs refresh before request
		if (tokenManager.isAccessTokenExpiringSoon()) {
			const newToken = await this.refreshAccessToken();
			if (newToken) {
				onTokenRefreshed(newToken);
			}
		}

		// Add auth token if available
		let token = tokenManager.getAccessToken();
		if (token) {
			config.headers = {
				...config.headers,
				Authorization: `Bearer ${token}`
			};
		}

		try {
			const response = await fetch(url, config);

			// Clear timeout on successful response
			clearTimeout(timeoutId);

			// Handle 401 Unauthorized - token expired
			if (response.status === 401) {
				// Try to refresh token once
				const newToken = await this.refreshAccessToken();

				if (newToken) {
					// Retry original request with new token
					config.headers = {
						...config.headers,
						Authorization: `Bearer ${newToken}`
					};
					onTokenRefreshed(newToken);
					return this.request<T>(endpoint, options);
				} else {
					// Refresh failed, redirect to login
					if (typeof window !== 'undefined') {
						window.location.href = '/login?error=session_expired';
					}
					return {
						success: false,
						error: 'Session expired'
					};
				}
			}

			if (!response.ok) {
				const errorData = await response.json().catch(() => ({ message: 'Network error' }));
				return {
					success: false,
					error: errorData.message || `HTTP ${response.status}`
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
			// Clear timeout on error
			clearTimeout(timeoutId);

			// Handle timeout specifically
			if (error instanceof Error && error.name === 'AbortError') {
				return {
					success: false,
					error: 'Request timeout - please try again'
				};
			}

			return {
				success: false,
				error: error instanceof Error ? error.message : 'Unknown error'
			};
		}
	}

	async get<T>(endpoint: string): Promise<ApiResponse<T>> {
		return this.request<T>(endpoint, { method: 'GET' });
	}

	async post<T>(endpoint: string, data?: Record<string, unknown>): Promise<ApiResponse<T>> {
		return this.request<T>(endpoint, {
			method: 'POST',
			body: data ? JSON.stringify(data) : undefined
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
