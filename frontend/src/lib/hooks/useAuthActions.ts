// Auth action helpers that can be used without triggering useAuth initialization
// Use these in login/register pages to avoid multiple onMount calls

import { authStore } from '$lib/stores/auth.svelte';
import { authApi } from '$lib/api/auth';
import { AuthSession } from '$lib/auth/session';
import type { User } from '$lib/types';

/**
 * Login action - can be used without triggering auth initialization
 *
 * SECURITY: Tokens are now stored in httpOnly cookies by the backend.
 * We only store user display info in localStorage.
 */
export async function loginAction(email: string, password: string) {
	authStore.setLoading(true);
	try {
		const result = await authApi.login({ email, password });
		if (result.success && result.data) {
			// Store user info for UI display (tokens are in httpOnly cookies)
			AuthSession.setSession(result.data);

			// Map backend UserInfo to frontend User type
			const user: User = {
				id: result.data.user.id,
				email: result.data.user.email,
				name: result.data.user.full_name || result.data.user.email,
				role: (result.data.user.role as 'owner' | 'admin' | 'manager' | 'user') || 'user',
				tenantId: result.data.user.tenant_id,
				createdAt: result.data.user.created_at,
				updatedAt: result.data.user.created_at
			};

			authStore.setUser(user);

			// Wait a tick to ensure state is updated
			await new Promise((resolve) => setTimeout(resolve, 0));

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
 *
 * SECURITY: Tokens are now stored in httpOnly cookies by the backend.
 * We only store user info for display purposes.
 */
export async function registerAction(userData: {
	name: string;
	email: string;
	password: string;
	confirmPassword: string;
	tenantName?: string;
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
			// Store user info for UI display (tokens are in httpOnly cookies)
			AuthSession.setSession(result.data);

			// Map backend UserInfo to frontend User type
			const user: User = {
				id: result.data.user.id,
				email: result.data.user.email,
				name: result.data.user.full_name || result.data.user.email,
				role: (result.data.user.role as 'owner' | 'admin' | 'manager' | 'user') || 'user',
				tenantId: result.data.user.tenant_id,
				createdAt: result.data.user.created_at,
				updatedAt: result.data.user.created_at
			};

			authStore.setUser(user);

			// Return success with data for components that need it
			return { success: true, data: result.data };
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
 *
 * Calls the backend to revoke the session and clear httpOnly cookies.
 */
export async function logoutAction() {
	// Call backend to revoke refresh token and clear httpOnly cookies
	await AuthSession.logout();
	authStore.logout();
}
