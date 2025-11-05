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
		authState.user = null;
		authState.tenant = null;
		authState.isAuthenticated = false;
		authState.isLoading = false;
	},

	initialize: () => {
		// Check for stored auth data (JWT, etc.)
		const token = localStorage.getItem('auth_token');
		if (token) {
			// TODO: Validate token and set user data
			authState.isLoading = false;
		} else {
			authState.isLoading = false;
		}
	}
};
