// Auth action helpers that can be used without triggering useAuth initialization
// Use these in login/register pages to avoid multiple onMount calls

import { authStore } from '$lib/stores/auth.svelte';
import { authApi } from '$lib/api/auth';
import { tokenManager } from '$lib/auth/token-manager';
import type { User } from '$lib/types';

/**
 * Login action - can be used without triggering auth initialization
 */
export async function loginAction(email: string, password: string) {
	authStore.setLoading(true);
	try {
		const result = await authApi.login({ email, password });
		if (result.success && result.data) {
			// Store tokens securely using tokenManager
			tokenManager.setAccessToken(result.data.access_token, result.data.expires_in);
			await tokenManager.setRefreshToken(result.data.refresh_token);

			// Map backend UserInfo to frontend User type
			const user: User = {
				id: result.data.user.id,
				email: result.data.user.email,
				name: result.data.user.full_name || result.data.user.email,
				role: (result.data.user.role as 'admin' | 'manager' | 'user') || 'user',
				tenantId: result.data.user.tenant_id,
				createdAt: result.data.user.created_at,
				updatedAt: result.data.user.created_at
			};

			// Store user data for session persistence
			await tokenManager.setUserData(JSON.stringify(user));

			authStore.setUser(user);

			// Wait a tick to ensure state is updated
			await new Promise(resolve => setTimeout(resolve, 0));

			return { success: true };
		} else {
			throw new Error(result.error || 'Login failed');
		}
	} catch (err) {
		throw err instanceof Error ? err : new Error('Login failed');
	} finally {
		authStore.setLoading(false);
	}
}

/**
 * Register action - can be used without triggering auth initialization
 */
export async function registerAction(userData: {
	name: string;
	email: string;
	password: string;
	confirmPassword: string;
	tenantName?: string
}) {
	authStore.setLoading(true);
	try {
		const result = await authApi.register({
			full_name: userData.name,
			email: userData.email,
			password: userData.password,
			tenant_name: userData.tenantName
		});
		if (result.success && result.data) {
			// Registration successful, but don't auto-login
			return { success: true };
		} else {
			throw new Error(result.error || 'Registration failed');
		}
	} catch (err) {
		throw err instanceof Error ? err : new Error('Registration failed');
	} finally {
		authStore.setLoading(false);
	}
}

/**
 * Logout action - can be used without triggering auth initialization
 */
export async function logoutAction() {
	// Call backend to revoke refresh token
	const refreshToken = await tokenManager.getRefreshToken();
	if (refreshToken) {
		try {
			await authApi.logoutLegacy({ refresh_token: refreshToken });
		} catch (error) {
			console.error('Logout API call failed:', error);
			// Continue with client-side logout even if API fails
		}
	}

	// Clear all tokens and user data
	tokenManager.clearAll();
	authStore.logout();
}
