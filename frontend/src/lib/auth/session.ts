import { type EmailAuthResponse, type EmailUserInfo } from '$lib/api/auth';
import { tokenManager } from './token-manager';

/**
 * Session management utilities
 *
 * SECURITY NOTE: Tokens are stored in httpOnly cookies by the server for security.
 * This class only manages user info in localStorage for UI purposes.
 * Never store tokens in localStorage as they are vulnerable to XSS attacks.
 */
export class AuthSession {
	private static readonly USER_KEY = 'user_data';

	// Tokens are managed by httpOnly cookies on the server
	// These methods exist for backwards compatibility but should not be used
	static getAccessToken(): string | null {
		console.warn(
			'getAccessToken: Tokens should be accessed from httpOnly cookies, not localStorage'
		);
		return null;
	}

	static getRefreshToken(): string | null {
		console.warn(
			'getRefreshToken: Tokens should be accessed from httpOnly cookies, not localStorage'
		);
		return null;
	}

	// Get stored user info (safe to store in localStorage)
	static getUser(): EmailUserInfo | null {
		if (typeof window === 'undefined') return null;
		const userJson = localStorage.getItem(this.USER_KEY);
		if (!userJson) return null;

		try {
			return JSON.parse(userJson);
		} catch {
			return null;
		}
	}

	// Store session data - only stores user info, tokens are in httpOnly cookies
	static setSession(data: EmailAuthResponse): void {
		if (typeof window === 'undefined') return;

		// Only store non-sensitive user data
		localStorage.setItem(this.USER_KEY, JSON.stringify(data.user));

		// Update tokenManager with tokens for API client (uses memory/sessionStorage)
		tokenManager.setAccessToken(data.access_token, data.expires_in);
		tokenManager.setRefreshToken(data.refresh_token);
	}

	// Clear session data
	static clearSession(): void {
		if (typeof window === 'undefined') return;

		localStorage.removeItem(this.USER_KEY);

		// Also clear tokenManager
		tokenManager.clearAll();
	}

	// Check if user is authenticated (checks for user data presence)
	// Actual token validation is done server-side via httpOnly cookies
	static isAuthenticated(): boolean {
		return this.getUser() !== null;
	}

	// Refresh the access token - delegates to server endpoint
	static async refreshToken(): Promise<boolean> {
		try {
			// Server will use refresh_token from httpOnly cookie
			const response = await fetch('/api/v1/auth/oauth/refresh', {
				method: 'POST',
				credentials: 'include' // Include httpOnly cookies
			});

			if (response.ok) {
				const data = await response.json();
				// Update user info if provided
				if (data.user) {
					localStorage.setItem(this.USER_KEY, JSON.stringify(data.user));
				}
				return true;
			}
		} catch (error) {
			console.error('Token refresh failed:', error);
		}

		// If refresh fails, clear session
		this.clearSession();
		return false;
	}

	// Logout user - calls server endpoint which clears httpOnly cookies
	static async logout(): Promise<void> {
		try {
			// Server endpoint will clear httpOnly cookies
			await fetch('/api/v1/auth/logout', {
				method: 'POST',
				credentials: 'include'
			});
		} catch (error) {
			console.error('Server logout failed:', error);
		}

		// Always clear local session data
		this.clearSession();
	}

	// Validate current session - checks for user data
	static async validateSession(): Promise<boolean> {
		return this.isAuthenticated();
	}
}

// Helper functions for components
export function getAuthHeaders(): Record<string, string> {
	// Tokens are in httpOnly cookies, no need to manually add Authorization header
	// The browser automatically includes cookies with requests
	console.warn(
		'getAuthHeaders: Use credentials: "include" instead of manual Authorization headers'
	);
	return {};
}

export function requireAuth(): boolean {
	if (!AuthSession.isAuthenticated()) {
		return false;
	}
	return true;
}
