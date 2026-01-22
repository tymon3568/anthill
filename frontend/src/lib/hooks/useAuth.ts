import { onMount } from 'svelte';
import { authState, authStore } from '$lib/stores/auth.svelte';
import { AuthSession } from '$lib/auth/session';
import type { User } from '$lib/types';

// Check and handle session invalidation signal from server
function handleSessionInvalidation(): boolean {
	const sessionInvalidatedCookie = document.cookie
		.split('; ')
		.find((row) => row.startsWith('session_invalidated='));

	if (sessionInvalidatedCookie) {
		// Clear the signal cookie
		document.cookie = 'session_invalidated=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';

		// Clear all auth-related localStorage
		localStorage.removeItem('user_data');
		localStorage.removeItem('anthill_tenant_slug');

		// Clear any other auth cookies that might be accessible
		document.cookie = 'user_data=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';

		console.log('[useAuth] Session invalidated by server, cleared local state');
		return true;
	}
	return false;
}

// Custom hook for auth initialization
export function useAuth() {
	onMount(async () => {
		// Set loading to true while initializing
		authStore.setLoading(true);

		try {
			// Check if server signaled that session is invalid (e.g., user deleted, session revoked)
			// This happens when refresh token fails with permanent errors like USER_NOT_FOUND
			const wasInvalidated = handleSessionInvalidation();
			if (wasInvalidated) {
				// Session was invalidated, don't try to restore from storage
				authStore.setUser(null);
				authStore.setLoading(false);
				return;
			}

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
