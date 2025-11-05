import { describe, it, expect, beforeEach, vi } from 'vitest';
import { authState, authStore } from './auth.svelte';
import type { User, Tenant } from '$lib/types';

// Mock localStorage
const localStorageMock = {
	getItem: vi.fn(),
	setItem: vi.fn(),
	removeItem: vi.fn(),
	clear: vi.fn(),
};
Object.defineProperty(global, 'localStorage', {
	value: localStorageMock,
});

describe('Auth Store', () => {
	beforeEach(() => {
		// Reset auth state before each test
		authState.user = null;
		authState.tenant = null;
		authState.isAuthenticated = false;
		authState.isLoading = true;

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

			authStore.setUser(user);

			expect(authState.user).toEqual(user);
			expect(authState.isAuthenticated).toBe(true);
		});

		it('should set user to null and isAuthenticated to false', () => {
			authStore.setUser(null);

			expect(authState.user).toBeNull();
			expect(authState.isAuthenticated).toBe(false);
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

			authStore.setTenant(tenant);

			expect(authState.tenant).toEqual(tenant);
		});

		it('should set tenant to null', () => {
			authStore.setTenant(null);

			expect(authState.tenant).toBeNull();
		});
	});

	describe('setLoading', () => {
		it('should set loading state to true', () => {
			authStore.setLoading(true);

			expect(authState.isLoading).toBe(true);
		});

		it('should set loading state to false', () => {
			authStore.setLoading(false);

			expect(authState.isLoading).toBe(false);
		});
	});

	describe('logout', () => {
		beforeEach(() => {
			// Set up initial state
			authState.user = {
				id: '1',
				email: 'user@example.com',
				name: 'John Doe',
				role: 'user',
				tenantId: 'tenant-1',
				createdAt: '2025-01-01T00:00:00Z',
				updatedAt: '2025-01-01T00:00:00Z'
			};
			authState.tenant = {
				id: 'tenant-1',
				name: 'Test Tenant',
				domain: 'test.com',
				createdAt: '2025-01-01T00:00:00Z',
				updatedAt: '2025-01-01T00:00:00Z'
			};
			authState.isAuthenticated = true;
			authState.isLoading = false;
		});

		it('should clear auth token from localStorage', () => {
			authStore.logout();

			expect(localStorageMock.removeItem).toHaveBeenCalledWith('auth_token');
		});

		it('should reset all auth state', () => {
			authStore.logout();

			expect(authState.user).toBeNull();
			expect(authState.tenant).toBeNull();
			expect(authState.isAuthenticated).toBe(false);
			expect(authState.isLoading).toBe(false);
		});
	});

	describe('initialize', () => {
		it('should not throw error', () => {
			expect(() => authStore.initialize()).not.toThrow();
		});

		it('should be callable', () => {
			authStore.initialize();
			// Initialize is currently a no-op, just ensuring it doesn't break
			expect(authState.isLoading).toBe(true); // Initial state
		});
	});

	describe('initial state', () => {
		it('should have correct initial values', () => {
			expect(authState.user).toBeNull();
			expect(authState.tenant).toBeNull();
			expect(authState.isAuthenticated).toBe(false);
			expect(authState.isLoading).toBe(true);
		});
	});
});
