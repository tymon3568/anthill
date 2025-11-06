// ⚠️ CRITICAL: This project uses Svelte 5 runes exclusively.
//    - Use $state (NOT writable stores)
//    - Always consult MCP documentation before changes
//    - See .svelte-instructions.md for guidelines
import type { AuthStore, User, Tenant } from '$lib/types';
import { validateAndParseToken, shouldRefreshToken } from '$lib/auth/jwt';
import { authLogic } from '$lib/auth/auth-logic';

// Browser check with fallback for testing
const getBrowser = (): boolean => {
	try {
		// This will work in SvelteKit environment
		const { browser } = require('$app/environment');
		return browser;
	} catch {
		// Fallback for testing environments
		return typeof window !== 'undefined';
	}
};

const browser = getBrowser();

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
