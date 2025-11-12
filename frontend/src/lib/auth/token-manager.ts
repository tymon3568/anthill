/**
 * Simplified token management for client-side use
 * Uses sessionStorage for security (cleared on tab close)
 */

import { browser } from '$app/environment';

// In-memory token storage (most secure)
let accessToken: string | null = null;
let tokenExpiresAt: number | null = null;

// Token manager interface
export const tokenManager = {
	/**
	 * Set access token in memory
	 */
	setAccessToken(token: string, expiresIn: number): void {
		if (!browser) return;

		accessToken = token;
		// Calculate expiration time (expiresIn is in seconds)
		tokenExpiresAt = Date.now() + expiresIn * 1000;
	},

	/**
	 * Get access token from memory
	 */
	getAccessToken(): string | null {
		if (!browser) return null;

		// Check if token is expired
		if (tokenExpiresAt && Date.now() >= tokenExpiresAt) {
			console.warn('Access token expired, clearing...');
			accessToken = null;
			tokenExpiresAt = null;
			return null;
		}

		return accessToken;
	},

	/**
	 * Set refresh token in sessionStorage
	 * Note: sessionStorage is cleared on tab close, providing some security
	 */
	async setRefreshToken(token: string): Promise<void> {
		if (!browser || typeof sessionStorage === 'undefined') return;

		try {
			sessionStorage.setItem('refresh_token', token);
		} catch (error) {
			console.error('Failed to store refresh token:', error);
		}
	},

	/**
	 * Get refresh token from sessionStorage
	 */
	async getRefreshToken(): Promise<string | null> {
		if (!browser || typeof sessionStorage === 'undefined') return null;

		try {
			const token = sessionStorage.getItem('refresh_token');
			return token;
		} catch (error) {
			console.error('Failed to get refresh token:', error);
			return null;
		}
	},

	/**
	 * Store user data in sessionStorage
	 */
	async setUserData(userData: string): Promise<void> {
		if (!browser || typeof sessionStorage === 'undefined') return;

		try {
			sessionStorage.setItem('user_data', userData);
		} catch (error) {
			console.error('Failed to store user data:', error);
		}
	},

	/**
	 * Get user data from sessionStorage
	 */
	async getUserData(): Promise<string | null> {
		if (!browser || typeof sessionStorage === 'undefined') return null;

		try {
			const userData = sessionStorage.getItem('user_data');
			return userData;
		} catch (error) {
			console.error('Failed to get user data:', error);
			return null;
		}
	},

	/**
	 * Clear all tokens and user data (logout)
	 */
	clearAll(): void {
		if (!browser || typeof sessionStorage === 'undefined') return;

		// Clear memory
		accessToken = null;
		tokenExpiresAt = null;

		// Clear sessionStorage
		try {
			sessionStorage.removeItem('refresh_token');
			sessionStorage.removeItem('user_data');
		} catch (error) {
			console.error('Failed to clear storage:', error);
		}
	},

	/**
	 * Check if user has valid session (has refresh token)
	 */
	async hasValidSession(): Promise<boolean> {
		if (!browser || typeof sessionStorage === 'undefined') return false;

		try {
			const token = await this.getRefreshToken();
			return token !== null;
		} catch {
			return false;
		}
	},

	/**
	 * Get time until token expiration (in seconds)
	 */
	getTimeUntilExpiration(): number | null {
		if (!tokenExpiresAt) return null;

		const remaining = tokenExpiresAt - Date.now();
		return Math.max(0, Math.floor(remaining / 1000));
	},

	/**
	 * Check if access token is expiring soon (within 5 minutes)
	 */
	isAccessTokenExpiringSoon(): boolean {
		const timeUntilExpiration = this.getTimeUntilExpiration();
		if (timeUntilExpiration === null) return false;

		const fiveMinutes = 5 * 60; // 5 minutes in seconds
		return timeUntilExpiration <= fiveMinutes;
	}
};
