// ⚠️ CRITICAL: This project uses Svelte 5 runes exclusively.
//    - Use $state (NOT writable stores)
//    - Always consult MCP documentation before changes
//    - See .svelte-instructions.md for guidelines
import type { AuthStore, User, Tenant } from '$lib/types';

// Auth state using Svelte 5 runes
export const authState = $state<AuthStore>({
	user: null,
	tenant: null,
	isAuthenticated: false,
	isLoading: true
});

export const authStore = {
	setUser: (user: User | null) => {
		authState.user = user;
		authState.isAuthenticated = !!user;
	},

	setTenant: (tenant: Tenant | null) => {
		authState.tenant = tenant;
	},

	setLoading: (loading: boolean) => {
		authState.isLoading = loading;
	},

	logout: () => {
		if (typeof localStorage !== 'undefined') {
			localStorage.removeItem('auth_token');
		}
		authState.user = null;
		authState.tenant = null;
		authState.isAuthenticated = false;
		authState.isLoading = false;
	},

	initialize: () => {
		// Check for stored auth data (JWT, etc.)
		// Let useAuth hook manage loading state for async operations
		// This prevents redirects from triggering before profile validation completes
	},
};
