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
			// Note: In production, tokens are stored in httpOnly cookies
			// This is for development/demo purposes only
			const token = localStorage.getItem('auth_token');
			if (token && !shouldRefreshToken(token)) {
				const userInfo = validateAndParseToken(token);
				if (userInfo) {
					authStore.setUser({
						id: userInfo.userId,
						email: userInfo.email,
						name: userInfo.name || userInfo.email, // Fallback to email if no name
						role: 'user', // Default role, should be determined by groups
						tenantId: userInfo.tenantId || '',
						createdAt: new Date().toISOString(),
						updatedAt: new Date().toISOString()
					});
					if (userInfo.tenantId) {
						authStore.setTenant({
							id: userInfo.tenantId,
							name: userInfo.tenantId,
							domain: `${userInfo.tenantId}.example.com`, // Placeholder
							createdAt: new Date().toISOString(),
							updatedAt: new Date().toISOString()
						});
					}
				}
			}
		} catch (error) {
			console.error('Auth initialization error:', error);
		} finally {
			authStore.setLoading(false);
		}
	},

	logout: () => {
		if (browser) {
			localStorage.removeItem('auth_token');
			localStorage.removeItem('refresh_token');
		}
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
