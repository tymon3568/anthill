// ⚠️ CRITICAL: This project uses Svelte 5 runes exclusively.
//    - Use $state (NOT writable stores)
//    - Always consult MCP documentation before changes
//    - See .svelte-instructions.md for guidelines
import type { AuthStore, User, Tenant } from '$lib/types';
import { validateAndParseToken, shouldRefreshToken } from '$lib/auth/jwt';
import { authLogic } from '$lib/auth/auth-logic';

// Browser detection - fallback for testing environments
import { browser } from '$app/environment';

// Auth state using Svelte 5 runes - mirrors the logic state
export const authState = $state<AuthStore>({
	user: null,
	tenant: null,
	isAuthenticated: false,
	isLoading: true
});

// Sync Svelte state with logic state
const syncState = () => {
	const logicState = authLogic.getState();
	authState.user = logicState.user;
	authState.tenant = logicState.tenant;
	authState.isAuthenticated = logicState.isAuthenticated;
	authState.isLoading = logicState.isLoading;
};

export const authStore = {
	setUser: (user: User | null) => {
		authLogic.setUser(user);
		syncState();
	},

	setTenant: (tenant: Tenant | null) => {
		authLogic.setTenant(tenant);
		syncState();
	},

	setLoading: (loading: boolean) => {
		authLogic.setLoading(loading);
		syncState();
	},

	initialize: () => {
		// Client-side initialization is now handled by useAuth hook
		// This method is kept for backward compatibility but does nothing
		// The loading state will be managed by useAuth's onMount
	},

	initializeFromStorage: async () => {
		// This method is deprecated - initialization is now handled by useAuth hook
		// Kept for backward compatibility only
		return;
	},

	logout: () => {
		authLogic.logout();
		syncState();
	},

	// Check if user has permission for an action
	hasPermission: (resource: string, action: string): boolean => {
		return authLogic.hasPermission(resource, action);
	},

	// Get current tenant context
	getTenantId: (): string | null => {
		return authLogic.getTenantId();
	}
};
