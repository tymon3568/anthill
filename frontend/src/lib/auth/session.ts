import { authApi, type EmailAuthResponse, type EmailUserInfo } from '$lib/api/auth';
import { tokenManager } from './token-manager';

// Session management utilities
export class AuthSession {
	private static readonly ACCESS_TOKEN_KEY = 'access_token';
	private static readonly REFRESH_TOKEN_KEY = 'refresh_token';
	private static readonly USER_KEY = 'user';

	// Get stored access token
	static getAccessToken(): string | null {
		if (typeof window === 'undefined') return null;
		return localStorage.getItem(this.ACCESS_TOKEN_KEY);
	}

	// Get stored refresh token
	static getRefreshToken(): string | null {
		if (typeof window === 'undefined') return null;
		return localStorage.getItem(this.REFRESH_TOKEN_KEY);
	}

	// Get stored user info
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

	// Store session data
	static setSession(data: EmailAuthResponse): void {
		if (typeof window === 'undefined') return;

		localStorage.setItem(this.ACCESS_TOKEN_KEY, data.access_token);
		localStorage.setItem(this.REFRESH_TOKEN_KEY, data.refresh_token);
		localStorage.setItem(this.USER_KEY, JSON.stringify(data.user));

		// Also update tokenManager for API client compatibility
		tokenManager.setAccessToken(data.access_token, data.expires_in);
		tokenManager.setRefreshToken(data.refresh_token);
	}

	// Clear session data
	static clearSession(): void {
		if (typeof window === 'undefined') return;

		localStorage.removeItem(this.ACCESS_TOKEN_KEY);
		localStorage.removeItem(this.REFRESH_TOKEN_KEY);
		localStorage.removeItem(this.USER_KEY);

		// Also clear tokenManager
		tokenManager.clearAll();
	}

	// Check if user is authenticated
	static isAuthenticated(): boolean {
		return this.getAccessToken() !== null && this.getUser() !== null;
	}

	// Refresh the access token
	static async refreshToken(): Promise<boolean> {
		const refreshToken = this.getRefreshToken();
		if (!refreshToken) return false;

		try {
			const response = await authApi.refreshEmailToken(refreshToken);
			if (response.success && response.data) {
				this.setSession(response.data);
				return true;
			}
		} catch (error) {
			console.error('Token refresh failed:', error);
		}

		// If refresh fails, clear session
		this.clearSession();
		return false;
	}

	// Logout user
	static async logout(): Promise<void> {
		const refreshToken = this.getRefreshToken();

		// Try to logout on server (don't block on this)
		if (refreshToken) {
			try {
				await authApi.emailLogout(refreshToken);
			} catch (error) {
				console.error('Server logout failed:', error);
			}
		}

		// Always clear local session
		this.clearSession();
	}

	// Validate current session
	static async validateSession(): Promise<boolean> {
		if (!this.isAuthenticated()) return false;

		// For now, just check if tokens exist
		// In a real app, you might want to validate with the server
		return true;
	}
}

// Helper functions for components
export function getAuthHeaders(): Record<string, string> {
	const token = AuthSession.getAccessToken();
	return token ? { Authorization: `Bearer ${token}` } : {};
}

export function requireAuth(): boolean {
	if (!AuthSession.isAuthenticated()) {
		// In a real SvelteKit app, you'd use goto() here
		// But since this is a utility, we'll return false
		return false;
	}
	return true;
}
