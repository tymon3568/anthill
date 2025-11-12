import type { AuthStore, User, Tenant } from '$lib/types';
import { shouldRefreshToken } from '$lib/auth/jwt';
import { authLogic } from '$lib/auth/auth-logic';
import { authApi, type EmailAuthResponse } from '$lib/api/auth';
import { AuthSession } from '$lib/auth/session';

// Browser detection - fallback for testing environments
import { browser } from '$app/environment';

// Auth state using Svelte 5 runes - mirrors the logic state
export const authState = $state<AuthStore>({
	user: null,
	tenant: null,
	isAuthenticated: false,
	isLoading: false
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
	// Email/Password authentication methods
	async emailLogin(email: string, password: string): Promise<{ success: boolean; error?: string }> {
		authStore.setLoading(true);
		try {
			const response = await authApi.emailLogin({ email, password });

			if (response.success && response.data) {
				// Store session using our session manager
				AuthSession.setSession(response.data);

				// Map to User interface
				const user: User = {
					id: response.data.user.id,
					email: response.data.user.email,
					name: response.data.user.full_name || response.data.user.email,
					role: (response.data.user.role as 'admin' | 'manager' | 'user') || 'user',
					tenantId: response.data.user.tenant_id,
					createdAt: response.data.user.created_at,
					updatedAt: response.data.user.created_at
				};

				authStore.setUser(user);
				return { success: true };
			} else {
				return { success: false, error: 'Login failed' };
			}
		} catch (error) {
			console.error('Email login error:', error);
			return { success: false, error: error instanceof Error ? error.message : 'Login failed' };
		} finally {
			authStore.setLoading(false);
		}
	},

	async emailRegister(
		email: string,
		password: string,
		fullName: string,
		tenantName?: string
	): Promise<{ success: boolean; error?: string }> {
		authStore.setLoading(true);
		try {
			const response = await authApi.emailRegister({
				email,
				password,
				full_name: fullName,
				tenant_name: tenantName
			});

			if (response.success && response.data) {
				// Store session using our session manager
				AuthSession.setSession(response.data);

				// Map to User interface
				const user: User = {
					id: response.data.user.id,
					email: response.data.user.email,
					name: response.data.user.full_name || response.data.user.email,
					role: (response.data.user.role as 'admin' | 'manager' | 'user') || 'user',
					tenantId: response.data.user.tenant_id,
					createdAt: response.data.user.created_at,
					updatedAt: response.data.user.created_at
				};

				authStore.setUser(user);
				return { success: true };
			} else {
				return { success: false, error: 'Registration failed' };
			}
		} catch (error) {
			console.error('Email register error:', error);
			return {
				success: false,
				error: error instanceof Error ? error.message : 'Registration failed'
			};
		} finally {
			authStore.setLoading(false);
		}
	},

	async emailLogout(): Promise<void> {
		try {
			await AuthSession.logout();
		} catch (error) {
			console.error('Logout error:', error);
		} finally {
			authLogic.logout();
			syncState();
		}
	},

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

	initialize: async () => {
		// Client-side initialization
		if (browser) {
			// Make this awaitable so callers can avoid racing with storage init
			await authStore.initializeFromStorage();
		}
	},

	initializeFromStorage: async () => {
		if (!browser) return;

		// Check for email/password session first
		if (AuthSession.isAuthenticated()) {
			const user = AuthSession.getUser();
			if (user) {
				const mappedUser: User = {
					id: user.id,
					email: user.email,
					name: user.full_name || user.email,
					role: (user.role as 'admin' | 'manager' | 'user') || 'user',
					tenantId: user.tenant_id,
					createdAt: user.created_at,
					updatedAt: user.created_at
				};
				authStore.setUser(mappedUser);
				return;
			}
		}

		// Fallback to OAuth2 initialization
		try {
			// Read user data from cookie set by OAuth2 callback
			const userDataCookie = document.cookie
				.split('; ')
				.find((row) => row.startsWith('user_data='));

			if (userDataCookie) {
				const userDataStr = decodeURIComponent(userDataCookie.split('=')[1]);
				const userData = JSON.parse(userDataStr);

				// Map OAuth2 user data to our User interface
				const user: User = {
					id: userData.kanidm_user_id,
					email: userData.email,
					name: userData.preferred_username || userData.email,
					role: 'user', // Default role, could be determined from groups
					tenantId: userData.tenant?.tenant_id || '',
					createdAt: new Date().toISOString(), // We don't have this from OAuth2
					updatedAt: new Date().toISOString(),
					kanidm_user_id: userData.kanidm_user_id,
					preferred_username: userData.preferred_username,
					groups: userData.groups
				};

				const tenant: Tenant | null = userData.tenant
					? {
							id: userData.tenant.tenant_id,
							name: userData.tenant.name,
							slug: userData.tenant.slug,
							createdAt: new Date().toISOString(),
							updatedAt: new Date().toISOString()
						}
					: null;

				authStore.setUser(user);
				authStore.setTenant(tenant);
			} else {
				authStore.setUser(null);
				authStore.setTenant(null);
			}
		} catch (error) {
			console.error('Auth initialization error:', error);
			authStore.setUser(null);
			authStore.setTenant(null);
		}
	},

	logout: () => {
		// Clear email/password session
		AuthSession.clearSession();

		// Clear OAuth2 cookies
		if (browser) {
			document.cookie =
				'access_token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT; secure; samesite=strict';
			document.cookie =
				'refresh_token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT; secure; samesite=strict';
			document.cookie =
				'user_data=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT; secure; samesite=strict';
		}

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
