import { PUBLIC_API_BASE_URL } from '$env/static/public';
import type { ApiResponse } from '$lib/types';
import { tokenManager } from '$lib/auth/token-manager';
import { authApi } from './auth';
// Base API configuration
const API_BASE_URL = PUBLIC_API_BASE_URL;
const DEFAULT_TIMEOUT = 5000; // 5 seconds

// Track if we're currently refreshing to avoid duplicate refresh calls
let isRefreshing = false;
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

	/**
	 * Create an AbortSignal that times out after specified milliseconds
	 */
	private createTimeoutSignal(timeoutMs: number): AbortSignal {
		const controller = new AbortController();
		setTimeout(() => controller.abort(), timeoutMs);
		return controller.signal;
	}

	private async refreshAccessToken(): Promise<string | null> {
		const refreshToken = tokenManager.getRefreshToken();
		if (!refreshToken) return null;

		try {
			const result = await authApi.refreshTokenLegacy(refreshToken);
			if (result.success && result.data) {
				const expiresIn = result.data.expires_in || 900; // Default 15 minutes
				tokenManager.setAccessToken(result.data.access_token, expiresIn);
				tokenManager.setRefreshToken(result.data.refresh_token);
				return result.data.access_token;
			}
		} catch (error) {
			console.error('Token refresh failed:', error);
			// Clear invalid tokens
			tokenManager.clearAll();
		}
		return null;
	}

	private async request<T>(
		endpoint: string,
		options: RequestInit & { timeout?: number } = {}
	): Promise<ApiResponse<T>> {
		const url = `${this.baseURL}${endpoint}`;
		const timeout = options.timeout ?? DEFAULT_TIMEOUT;

		// Extract custom timeout option and prepare fetch config
		const { timeout: _, ...fetchOptions } = options;

		const config: RequestInit = {
			headers: {
				'Content-Type': 'application/json',
				...fetchOptions.headers
			},
			signal: this.createTimeoutSignal(timeout),
			...fetchOptions
		};

		// Check if token needs refresh before request
		if (tokenManager.isAccessTokenExpiringSoon() && !isRefreshing) {
			isRefreshing = true;
			const newToken = await this.refreshAccessToken();
			isRefreshing = false;
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

			// Handle 401 Unauthorized - token expired
			if (response.status === 401 && !isRefreshing) {
				isRefreshing = true;
				const newToken = await this.refreshAccessToken();
				isRefreshing = false;

				if (newToken) {
					// Retry original request with new token
					config.headers = {
						...config.headers,
						Authorization: `Bearer ${newToken}`
					};
					// Create new timeout signal for retry
					config.signal = this.createTimeoutSignal(timeout);
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

				// Map HTTP status codes to user-friendly messages
				if (response.status === 500) {
					return {
						success: false,
						error: 'Service temporarily unavailable. Please try again later.'
					};
				}

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
			// Handle timeout specifically
			if (error instanceof Error && error.name === 'AbortError') {
				return {
					success: false,
					error: 'Request timeout. Please check your connection and try again.'
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
