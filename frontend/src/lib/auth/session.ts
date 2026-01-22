import { type EmailAuthResponse, type EmailUserInfo } from '$lib/api/auth';

/**
 * Session management utilities
 *
 * SECURITY NOTE: Tokens are stored in httpOnly cookies by the backend.
 * This class only manages user info in localStorage for UI purposes.
 * Tokens are NEVER accessible to JavaScript - they are automatically
 * sent by the browser via cookies when credentials: 'include' is used.
 */
export class AuthSession {
	private static readonly USER_KEY = 'user_data';

	/**
	 * Get stored user info (safe to store in localStorage for UI)
	 * This does NOT contain tokens - only user display information
	 */
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

	/**
	 * Store session data after login/register
	 * Only stores user info for UI display - tokens are handled by httpOnly cookies
	 */
	static setSession(data: EmailAuthResponse): void {
		if (typeof window === 'undefined') return;

		// Only store non-sensitive user data in localStorage for UI display
		// SECURITY: Tokens are NOT stored here - they are in httpOnly cookies
		// set by the backend and automatically sent by the browser
		localStorage.setItem(this.USER_KEY, JSON.stringify(data.user));
	}

	/**
	 * Clear session data on logout
	 * Note: The backend clears httpOnly cookies via the logout endpoint
	 */
	static clearSession(): void {
		if (typeof window === 'undefined') return;

		// Clear user info from localStorage
		localStorage.removeItem(this.USER_KEY);
	}

	/**
	 * Check if server has signaled session invalidation
	 * This happens when refresh token fails with permanent errors like USER_NOT_FOUND
	 */
	private static checkSessionInvalidated(): boolean {
		if (typeof window === 'undefined') return false;

		const sessionInvalidatedCookie = document.cookie
			.split('; ')
			.find((row) => row.startsWith('session_invalidated='));

		if (sessionInvalidatedCookie) {
			// Clear the signal cookie
			document.cookie = 'session_invalidated=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';

			// Clear all auth-related localStorage
			localStorage.removeItem(this.USER_KEY);
			localStorage.removeItem('anthill_tenant_slug');

			console.log('[AuthSession] Session invalidated by server, cleared local state');
			return true;
		}
		return false;
	}

	/**
	 * Check if user has local session data
	 * Note: This only checks localStorage - actual auth is validated server-side via cookies
	 */
	static isAuthenticated(): boolean {
		// First check if server has signaled session invalidation
		if (this.checkSessionInvalidated()) {
			return false;
		}
		return this.getUser() !== null;
	}

	/**
	 * Refresh the access token
	 * The backend reads refresh_token from httpOnly cookie and sets new cookies
	 */
	static async refreshToken(): Promise<boolean> {
		try {
			// Server will use refresh_token from httpOnly cookie
			// and set new access_token/refresh_token cookies in response
			const response = await fetch('/api/v1/auth/refresh', {
				method: 'POST',
				credentials: 'include' // Include httpOnly cookies
			});

			if (response.ok) {
				const data = await response.json();
				// Update user info if provided in response
				if (data.user) {
					localStorage.setItem(this.USER_KEY, JSON.stringify(data.user));
				}
				return true;
			}
		} catch (error) {
			console.error('Token refresh failed:', error);
		}

		// If refresh fails, clear local session data
		this.clearSession();
		return false;
	}

	/**
	 * Logout user
	 * Calls server endpoint which clears httpOnly cookies
	 */
	static async logout(): Promise<void> {
		try {
			// Server endpoint will:
			// 1. Read refresh_token from httpOnly cookie
			// 2. Revoke the session in database
			// 3. Clear httpOnly cookies in response
			await fetch('/api/v1/auth/logout', {
				method: 'POST',
				credentials: 'include' // Include httpOnly cookies
			});
		} catch (error) {
			console.error('Server logout failed:', error);
		}

		// Always clear local session data
		this.clearSession();
	}

	/**
	 * Check if user appears authenticated (has local user data)
	 * Actual token validation happens server-side via httpOnly cookies
	 */
	static async validateSession(): Promise<boolean> {
		return this.isAuthenticated();
	}
}

/**
 * Check if user is authenticated (has local session data)
 * Actual authentication is validated server-side via httpOnly cookies
 */
export function requireAuth(): boolean {
	return AuthSession.isAuthenticated();
}
