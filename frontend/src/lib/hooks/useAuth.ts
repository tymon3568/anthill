import { onMount } from 'svelte';
import { authState, authStore } from '$lib/stores/auth.svelte';
import { authApi } from '$lib/api/auth';
import { AuthSession } from '$lib/auth/session';
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
		role: (profile.role as 'owner' | 'admin' | 'manager' | 'user') || 'user',
		tenantId: profile.tenant_id,
		createdAt: profile.created_at,
		updatedAt: profile.updated_at
	};
}

// Custom hook for auth initialization
export function useAuth() {
	onMount(async () => {
		// Set loading to true while initializing
		authStore.setLoading(true);

		try {
			// Initialize auth state (await to avoid racing with storage init)
			await authStore.initialize();

			// Try to restore session from cookies (set by OAuth2 callback)
			const userDataCookie = document.cookie
				.split('; ')
				.find((row) => row.startsWith('user_data='));

			if (userDataCookie) {
				try {
					const userDataValue = decodeURIComponent(userDataCookie.split('=')[1]);
					const user = JSON.parse(userDataValue) as User;
					authStore.setUser(user);
				} catch (error) {
					console.error('Failed to parse stored user data from cookie:', error);
					// Clear invalid cookie
					document.cookie = 'user_data=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';
				}
			}
		} catch (error) {
			console.error('Failed to initialize auth:', error);
			// Clear any corrupted data
			document.cookie = 'user_data=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';
		} finally {
			// Always set loading to false, even if initialization fails
			authStore.setLoading(false);
		}
	});

	return {
		user: authState.user,
		tenant: authState.tenant,
		isAuthenticated: authState.isAuthenticated,
		isLoading: authState.isLoading,
		login: async () => {
			// OAuth2 login - redirect to authorization endpoint (deprecated)
			authStore.setLoading(true);
			try {
				// This will redirect to OAuth2 authorize endpoint
				// The actual implementation is in the server endpoint
				window.location.href = '/api/v1/auth/oauth/authorize';
				// Return a promise that never resolves since navigation is in progress
				return new Promise<never>(() => {
					// Intentionally left unresolved; navigation is in progress.
				});
			} catch (err) {
				authStore.setLoading(false);
				throw err instanceof Error ? err : new Error('Login failed');
			}
		},
		register: async () => {
			// OAuth2 registration - redirect to authorization endpoint (deprecated)
			authStore.setLoading(true);
			try {
				// This will redirect to OAuth2 authorize endpoint for registration
				// The actual implementation is in the server endpoint
				window.location.href = '/api/v1/auth/oauth/authorize?mode=register';
				// Return a promise that never resolves since navigation is in progress
				return new Promise<never>(() => {
					// Intentionally left unresolved; navigation is in progress.
				});
			} catch (err) {
				authStore.setLoading(false);
				throw err instanceof Error ? err : new Error('Registration failed');
			}
		},
		logout: async () => {
			// Call server logout endpoint to clear httpOnly cookies
			await AuthSession.logout();
			authStore.logout();

			// Clear user data cookie (non-sensitive display data)
			document.cookie = 'user_data=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';

			// Optional: Redirect to external logout endpoint if needed
			// window.location.href = 'https://idm.example.com/ui/logout';
		}
	};
}
