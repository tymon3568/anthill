import { onMount } from 'svelte';
import { authState, authStore } from '$lib/stores/auth.svelte';
import { authApi } from '$lib/api/auth';
import { tokenManager } from '$lib/auth/token-manager';
import type { User } from '$lib/types';
import type { UserProfile } from '$lib/api/auth';

// Backend UserInfo type from AuthResponse
interface BackendUserInfo {
	id: string;
	email: string;
	full_name?: string;
	tenant_id: string;
	role: string;
	created_at: string;
}

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

// Flag to track if auth has been initialized
let isInitialized = false;

// Custom hook for auth initialization
export function useAuth() {
	onMount(async () => {
		// Only initialize once per app session
		if (isInitialized) {
			return;
		}

		isInitialized = true;

		// Set loading state at the start
		authStore.setLoading(true);

		// Try to restore session from sessionStorage
		try {
			if (tokenManager.hasValidSession()) {
				const storedUser = tokenManager.getUserData();

				if (storedUser) {
					try {
						const user = JSON.parse(storedUser) as User;
						authStore.setUser(user);
					} catch (error) {
						console.error('Failed to parse stored user data:', error);
						// Clear invalid data
						tokenManager.clearAll();
						authStore.setUser(null);
					}
				} else {
					// No user data found, clear tokens
					tokenManager.clearAll();
					authStore.setUser(null);
				}
			} else {
				// No valid session, ensure user is cleared
				authStore.setUser(null);
			}
		} catch (error) {
			console.error('Session restoration failed:', error);
			tokenManager.clearAll();
			authStore.setUser(null);
		} finally {
			// Always set loading to false after session check completes
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
					// Store tokens securely using tokenManager
					tokenManager.setAccessToken(result.data.access_token, result.data.expires_in);
					tokenManager.setRefreshToken(result.data.refresh_token);

					// Map backend UserInfo to frontend User type
					const user: User = {
						id: result.data.user.id,
						email: result.data.user.email,
						name: result.data.user.full_name || result.data.user.email,
						role: (result.data.user.role as 'admin' | 'manager' | 'user') || 'user',
						tenantId: result.data.user.tenant_id,
						createdAt: result.data.user.created_at,
						updatedAt: result.data.user.created_at // Backend doesn't return updated_at, use created_at
					};

					// Store user data for session persistence
					tokenManager.setUserData(JSON.stringify(user));

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
		},
		register: async (userData: { name: string; email: string; password: string; confirmPassword: string; tenantName?: string }) => {
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
					// User should login manually after registration
					return { success: true };
				} else {
					throw new Error(result.error || 'Registration failed');
				}
			} finally {
				authStore.setLoading(false);
			}
		},
		logout: async () => {
			// Call backend to revoke refresh token
			const refreshToken = tokenManager.getRefreshToken();
			if (refreshToken) {
				try {
					// Backend expects { refresh_token: string }
					await authApi.logoutLegacy();
				} catch (error) {
					console.error('Logout API call failed:', error);
					// Continue with client-side logout even if API fails
				}
			}

			// Clear all tokens and user data
			tokenManager.clearAll();
			authStore.logout();
		}
	};
}
