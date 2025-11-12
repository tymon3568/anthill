// Pure auth logic that can be tested without Svelte dependencies
import type { User, Tenant } from '$lib/types';

export interface AuthState {
	user: User | null;
	tenant: Tenant | null;
	isAuthenticated: boolean;
	isLoading: boolean;
}

export class AuthLogic {
	private state: AuthState = {
		user: null,
		tenant: null,
		isAuthenticated: false,
		isLoading: false
	};

	// Get current state (for testing)
	getState(): AuthState {
		return { ...this.state };
	}

	// Set user and update authentication status
	setUser(user: User | null): void {
		this.state.user = user;
		this.state.isAuthenticated = !!user;
	}

	// Set tenant
	setTenant(tenant: Tenant | null): void {
		this.state.tenant = tenant;
	}

	// Set loading state
	setLoading(loading: boolean): void {
		this.state.isLoading = loading;
	}

	// Initialize (placeholder for client-side logic)
	initialize(): void {
		// Client-side initialization logic would go here
	}

	// Initialize from storage (placeholder)
	async initializeFromStorage(): Promise<void> {
		// Token validation logic would go here
	}

	// Logout - clear all state
	logout(): void {
		this.state.user = null;
		this.state.tenant = null;
		this.state.isAuthenticated = false;
		this.state.isLoading = false;
	}

	// Check if user has permission for an action
	hasPermission(resource: string, action: string): boolean {
		if (!this.state.isAuthenticated || !this.state.user) return false;

		const userRole = this.state.user.role;
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
	}

	// Get current tenant ID
	getTenantId(): string | null {
		return this.state.tenant?.id || null;
	}
}

// Export singleton instance for use in Svelte store
export const authLogic = new AuthLogic();
