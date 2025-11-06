// ⚠️ CRITICAL: This project uses Svelte 5 runes exclusively.
//    - Use $state (NOT writable stores)
//    - Always consult MCP documentation before changes
//    - See .svelte-instructions.md for guidelines
import type { AuthStore, User, Tenant } from '$lib/types';
import { validateAndParseToken, shouldRefreshToken } from '$lib/auth/jwt';
import { browser } from '$app/environment';

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

	initialize: () => {
		// Client-side initialization
		if (browser) {
			authStore.initializeFromStorage();
		}
	},

	initializeFromStorage: async () => {
		if (!browser) return;

		authStore.setLoading(true);
		try {
			// Token validation happens server-side in hooks.server.ts
			// Client gets user info from server-side locals
			// This initialization is not needed for httpOnly cookie flow
		} catch (error) {
			console.error('Auth initialization error:', error);
		} finally {
			authStore.setLoading(false);
		}
	},

	logout: () => {
		// Token cleanup happens server-side via /api/v1/auth/logout
		// Client just clears local state
		authState.user = null;
		authState.tenant = null;
		authState.isAuthenticated = false;
		authState.isLoading = false;
	},

	// Check if user has permission for an action
	hasPermission: (resource: string, action: string): boolean => {
		// TODO: Implement Casbin permission checking via backend API
		// For now, just check if user is authenticated and has basic role
		// Full permission checking will be implemented in backend services
		if (!authState.isAuthenticated || !authState.user) return false;

		// Basic role-based permissions (will be replaced with Casbin)
		const userRole = authState.user.role;
		switch (action) {
			case 'read':
				return ['admin', 'manager', 'user'].includes(userRole);
			case 'write':
			case 'create':
				return ['admin', 'manager'].includes(userRole);
			case 'delete':
			case 'admin':
				return userRole === 'admin';
			default:
				return false;
		}
	},

	// Get current tenant context
	getTenantId: (): string | null => {
		return authState.tenant?.id || null;
	}
};
