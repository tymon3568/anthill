import { describe, it, expect, beforeEach, vi } from 'vitest';
import type { User, Tenant } from '$lib/types';
import { AuthLogic } from '$lib/auth/auth-logic';

// Mock localStorage for logout tests
const localStorageMock = {
	getItem: vi.fn(),
	setItem: vi.fn(),
	removeItem: vi.fn(),
	clear: vi.fn(),
};
Object.defineProperty(global, 'localStorage', {
	value: localStorageMock,
});

describe('Auth Store (AuthLogic)', () => {
	let authLogic: AuthLogic;

	beforeEach(() => {
		authLogic = new AuthLogic();
		// Reset localStorage mocks
		vi.clearAllMocks();
	});

	describe('setUser', () => {
		it('should set user and update isAuthenticated to true', () => {
			const user: User = {
				id: '1',
				email: 'user@example.com',
				name: 'John Doe',
				role: 'user',
				tenantId: 'tenant-1',
				createdAt: '2025-01-01T00:00:00Z',
				updatedAt: '2025-01-01T00:00:00Z'
			};

			authLogic.setUser(user);

			const state = authLogic.getState();
			expect(state.user).toEqual(user);
			expect(state.isAuthenticated).toBe(true);
		});

		it('should set user to null and isAuthenticated to false', () => {
			authLogic.setUser(null);

			const state = authLogic.getState();
			expect(state.user).toBeNull();
			expect(state.isAuthenticated).toBe(false);
		});
	});

	describe('setTenant', () => {
		it('should set tenant', () => {
			const tenant: Tenant = {
				id: 'tenant-1',
				name: 'Test Tenant',
				domain: 'test.com',
				createdAt: '2025-01-01T00:00:00Z',
				updatedAt: '2025-01-01T00:00:00Z'
			};

			authLogic.setTenant(tenant);

			const state = authLogic.getState();
			expect(state.tenant).toEqual(tenant);
		});

		it('should set tenant to null', () => {
			authLogic.setTenant(null);

			const state = authLogic.getState();
			expect(state.tenant).toBeNull();
		});
	});

	describe('setLoading', () => {
		it('should set loading state to true', () => {
			authLogic.setLoading(true);

			const state = authLogic.getState();
			expect(state.isLoading).toBe(true);
		});

		it('should set loading state to false', () => {
			authLogic.setLoading(false);

			const state = authLogic.getState();
			expect(state.isLoading).toBe(false);
		});
	});

	describe('logout', () => {
		beforeEach(() => {
			// Set up initial state
			const user: User = {
				id: '1',
				email: 'user@example.com',
				name: 'John Doe',
				role: 'user',
				tenantId: 'tenant-1',
				createdAt: '2025-01-01T00:00:00Z',
				updatedAt: '2025-01-01T00:00:00Z'
			};
			const tenant: Tenant = {
				id: 'tenant-1',
				name: 'Test Tenant',
				domain: 'test.com',
				createdAt: '2025-01-01T00:00:00Z',
				updatedAt: '2025-01-01T00:00:00Z'
			};
			authLogic.setUser(user);
			authLogic.setTenant(tenant);
			authLogic.setLoading(false);
		});

		it('should reset all auth state', () => {
			authLogic.logout();

			const state = authLogic.getState();
			expect(state.user).toBeNull();
			expect(state.tenant).toBeNull();
			expect(state.isAuthenticated).toBe(false);
			expect(state.isLoading).toBe(false);
		});
	});

	describe('hasPermission', () => {
		it('should return false for unauthenticated users', () => {
			authLogic.setUser(null);

			expect(authLogic.hasPermission('users', 'read')).toBe(false);
			expect(authLogic.hasPermission('users', 'write')).toBe(false);
		});

		it('should allow read access for all authenticated users', () => {
			const user: User = {
				id: '1',
				email: 'user@example.com',
				name: 'John Doe',
				role: 'user',
				tenantId: 'tenant-1',
				createdAt: '2025-01-01T00:00:00Z',
				updatedAt: '2025-01-01T00:00:00Z'
			};
			authLogic.setUser(user);

			expect(authLogic.hasPermission('users', 'read')).toBe(true);
		});

		it('should allow write access for admin and manager roles', () => {
			const adminUser: User = {
				id: '1',
				email: 'admin@example.com',
				name: 'Admin User',
				role: 'admin',
				tenantId: 'tenant-1',
				createdAt: '2025-01-01T00:00:00Z',
				updatedAt: '2025-01-01T00:00:00Z'
			};
			const managerUser: User = {
				id: '2',
				email: 'manager@example.com',
				name: 'Manager User',
				role: 'manager',
				tenantId: 'tenant-1',
				createdAt: '2025-01-01T00:00:00Z',
				updatedAt: '2025-01-01T00:00:00Z'
			};
			const regularUser: User = {
				id: '3',
				email: 'user@example.com',
				name: 'Regular User',
				role: 'user',
				tenantId: 'tenant-1',
				createdAt: '2025-01-01T00:00:00Z',
				updatedAt: '2025-01-01T00:00:00Z'
			};

			authLogic.setUser(adminUser);
			expect(authLogic.hasPermission('users', 'write')).toBe(true);

			authLogic.setUser(managerUser);
			expect(authLogic.hasPermission('users', 'write')).toBe(true);

			authLogic.setUser(regularUser);
			expect(authLogic.hasPermission('users', 'write')).toBe(false);
		});

		it('should only allow admin actions for admin role', () => {
			const adminUser: User = {
				id: '1',
				email: 'admin@example.com',
				name: 'Admin User',
				role: 'admin',
				tenantId: 'tenant-1',
				createdAt: '2025-01-01T00:00:00Z',
				updatedAt: '2025-01-01T00:00:00Z'
			};
			const managerUser: User = {
				id: '2',
				email: 'manager@example.com',
				name: 'Manager User',
				role: 'manager',
				tenantId: 'tenant-1',
				createdAt: '2025-01-01T00:00:00Z',
				updatedAt: '2025-01-01T00:00:00Z'
			};

			authLogic.setUser(adminUser);
			expect(authLogic.hasPermission('users', 'admin')).toBe(true);
			expect(authLogic.hasPermission('users', 'delete')).toBe(true);

			authLogic.setUser(managerUser);
			expect(authLogic.hasPermission('users', 'admin')).toBe(false);
			expect(authLogic.hasPermission('users', 'delete')).toBe(false);
		});
	});

	describe('getTenantId', () => {
		it('should return tenant ID when tenant is set', () => {
			const tenant: Tenant = {
				id: 'tenant-1',
				name: 'Test Tenant',
				domain: 'test.com',
				createdAt: '2025-01-01T00:00:00Z',
				updatedAt: '2025-01-01T00:00:00Z'
			};

			authLogic.setTenant(tenant);
			expect(authLogic.getTenantId()).toBe('tenant-1');
		});

		it('should return null when no tenant is set', () => {
			authLogic.setTenant(null);
			expect(authLogic.getTenantId()).toBeNull();
		});
	});

	describe('initial state', () => {
		it('should have correct initial values', () => {
			const state = authLogic.getState();
			expect(state.user).toBeNull();
			expect(state.tenant).toBeNull();
			expect(state.isAuthenticated).toBe(false);
			expect(state.isLoading).toBe(true);
		});
	});
});
