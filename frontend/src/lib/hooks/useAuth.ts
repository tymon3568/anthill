import { onMount } from 'svelte';
import { authState, authStore } from '$lib/stores/auth.svelte';
import { authApi } from '$lib/api/auth';
import type { User } from '$lib/types';
import type { UserProfile } from '$lib/api/auth';

// Convert UserProfile to User type
function mapUserProfileToUser(profile: UserProfile): User {
	return {
		id: profile.id,
		email: profile.email,
		name: profile.display_name || profile.username || profile.email,
		role: (profile.role as 'admin' | 'manager' | 'user') || 'user',
		tenantId: profile.tenant_id,
		createdAt: profile.created_at,
		updatedAt: profile.updated_at
	};
}

// Custom hook for auth initialization
export function useAuth() {
	onMount(() => {
		// Initialize auth state
		authStore.initialize();

		// Try to get user profile if we have a token
		const token = localStorage.getItem('auth_token');
		if (token && !authState.user) {
			authApi.getProfile()
				.then((result) => {
					if (result.success && result.data) {
						authStore.setUser(mapUserProfileToUser(result.data));
					} else {
						// Token invalid, clear it
						localStorage.removeItem('auth_token');
						localStorage.removeItem('refresh_token');
					}
				})
				.catch(() => {
					// Network or other error - clear tokens
					localStorage.removeItem('auth_token');
					localStorage.removeItem('refresh_token');
				})
				.finally(() => {
					authStore.setLoading(false);
				});
		} else {
			authStore.setLoading(false);
		}
	});

	return {
		user: authState.user,
		tenant: authState.tenant,
		isAuthenticated: authState.isAuthenticated,
		isLoading: authState.isLoading,
		login: async (email: string, password: string) => {
			authStore.setLoading(true);
			try {
				const result = await authApi.login({ email, password });
				if (result.success && result.data) {
					localStorage.setItem('auth_token', result.data.access_token);
					localStorage.setItem('refresh_token', result.data.refresh_token);
					authStore.setUser(result.data.user);
					return { success: true };
				} else {
					return { success: false, error: result.error };
				}
			} catch {
				return { success: false, error: 'Login failed' };
			} finally {
				authStore.setLoading(false);
			}
		},
		register: async (userData: { name: string; email: string; password: string; confirmPassword: string }) => {
			authStore.setLoading(true);
			try {
				const result = await authApi.register({
					name: userData.name,
					email: userData.email,
					password: userData.password
				});
				if (result.success && result.data) {
					// Registration successful, but don't auto-login
					// User should login manually after registration
					return { success: true };
				} else {
					return { success: false, error: result.error };
				}
			} catch (err) {
				return { success: false, error: err instanceof Error ? err.message : 'Registration failed' };
			} finally {
				authStore.setLoading(false);
			}
		},
		logout: () => {
			localStorage.removeItem('auth_token');
			localStorage.removeItem('refresh_token');
			authStore.logout();
		}
	};
}
