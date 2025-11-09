/**
 * Production-grade token management
 *
 * Strategy:
 * 1. Keep access_token in memory (most secure, lost on page refresh)
 * 2. Keep refresh_token in sessionStorage (survives refresh, cleared on tab close)
 * 3. Never use localStorage (vulnerable to XSS across all tabs)
 * 4. Auto-refresh before token expiration
 * 5. Clear all on logout
 */

import { browser } from '$app/environment';
import { encryptToken, decryptToken } from './token-encryption';

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
		tokenExpiresAt = Date.now() + (expiresIn * 1000);
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
	 * Check if access token is about to expire (within 2 minutes)
	 */
	isAccessTokenExpiringSoon(): boolean {
		if (!browser || !tokenExpiresAt) return false;

		const twoMinutes = 2 * 60 * 1000;
		return Date.now() >= (tokenExpiresAt - twoMinutes);
	},

	/**
	 * Set refresh token in sessionStorage (temporarily unencrypted for debugging)
	 */
	async setRefreshToken(token: string): Promise<void> {
		if (!browser) return;

		try {
			// Temporarily disable encryption for debugging
			sessionStorage.setItem('refresh_token', token);
		} catch (error) {
			console.error('Failed to store refresh token:', error);
		}
	},

	/**
	 * Get refresh token from sessionStorage (temporarily unencrypted for debugging)
	 */
	async getRefreshToken(): Promise<string | null> {
		if (!browser) return null;

		try {
			// Temporarily disable encryption for debugging
			return sessionStorage.getItem('refresh_token');
		} catch (error) {
			console.error('Failed to get refresh token:', error);
			return null;
		}
	},

	/**
	 * Store user data in sessionStorage (temporarily unencrypted for debugging)
	 */
	async setUserData(userData: string): Promise<void> {
		if (!browser) return;

		try {
			// Temporarily disable encryption for debugging
			sessionStorage.setItem('user_data', userData);
		} catch (error) {
			console.error('Failed to store user data:', error);
		}
	},

	/**
	 * Get user data from sessionStorage (temporarily unencrypted for debugging)
	 */
	async getUserData(): Promise<string | null> {
		if (!browser) return null;

		try {
			// Temporarily disable encryption for debugging
			return sessionStorage.getItem('user_data');
		} catch (error) {
			console.error('Failed to get user data:', error);
			return null;
		}
	},

	/**
	 * Clear all tokens and user data (logout)
	 */
	clearAll(): void {
		if (!browser) return;

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
		if (!browser) return false;

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
	}
};
