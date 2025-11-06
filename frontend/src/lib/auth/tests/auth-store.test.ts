import { describe, it, expect, beforeEach } from 'vitest';
import { AuthLogic } from '../auth-logic';

describe('Auth Store Logic', () => {
	let authLogic: AuthLogic;

	beforeEach(() => {
		authLogic = new AuthLogic();
	});

	describe('hasPermission', () => {
		it('should return false when not authenticated', () => {
			const result = authLogic.hasPermission('users', 'read');
			expect(result).toBe(false);
		});

		it('should return false when user has no role', () => {
			// Set user to null to simulate no authentication
			authLogic.setUser(null);

			const result = authLogic.hasPermission('users', 'read');
			expect(result).toBe(false);
		});

		it('should allow read access for all authenticated users', () => {
			authLogic.setUser({
				id: 'user-1',
				email: 'user@example.com',
				name: 'Test User',
				role: 'user',
				tenantId: 'tenant-1',
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString()
			});

			expect(authLogic.hasPermission('users', 'read')).toBe(true);
			expect(authLogic.hasPermission('products', 'read')).toBe(true);
			expect(authLogic.hasPermission('orders', 'read')).toBe(true);
		});

		it('should allow write access for managers and admins', () => {
			const roles = ['manager', 'admin'];

			roles.forEach(role => {
				authLogic.setUser({
					id: 'user-1',
					email: 'user@example.com',
					name: 'Test User',
					role: role as any,
					tenantId: 'tenant-1',
					createdAt: new Date().toISOString(),
					updatedAt: new Date().toISOString()
				});

				expect(authLogic.hasPermission('users', 'write')).toBe(true);
				expect(authLogic.hasPermission('products', 'create')).toBe(true);
			});
		});

		it('should deny write access for regular users', () => {
			authLogic.setUser({
				id: 'user-1',
				email: 'user@example.com',
				name: 'Test User',
				role: 'user',
				tenantId: 'tenant-1',
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString()
			});

			expect(authLogic.hasPermission('users', 'write')).toBe(false);
			expect(authLogic.hasPermission('products', 'create')).toBe(false);
		});

		it('should allow admin access only for admins', () => {
			authLogic.setUser({
				id: 'user-1',
				email: 'user@example.com',
				name: 'Test User',
				role: 'admin',
				tenantId: 'tenant-1',
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString()
			});

			expect(authLogic.hasPermission('users', 'delete')).toBe(true);
			expect(authLogic.hasPermission('system', 'admin')).toBe(true);
		});

		it('should deny admin access for non-admins', () => {
			const nonAdminRoles = ['user', 'manager'];

			nonAdminRoles.forEach(role => {
				authLogic.setUser({
					id: 'user-1',
					email: 'user@example.com',
					name: 'Test User',
					role: role as any,
					tenantId: 'tenant-1',
					createdAt: new Date().toISOString(),
					updatedAt: new Date().toISOString()
				});

				expect(authLogic.hasPermission('users', 'delete')).toBe(false);
				expect(authLogic.hasPermission('system', 'admin')).toBe(false);
			});
		});

		it('should return false for unknown actions', () => {
			authLogic.setUser({
				id: 'user-1',
				email: 'user@example.com',
				name: 'Test User',
				role: 'admin',
				tenantId: 'tenant-1',
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString()
			});

			expect(authLogic.hasPermission('users', 'unknown')).toBe(false);
		});
	});

	describe('getTenantId', () => {
		it('should return tenant ID when tenant is set', () => {
			authLogic.setTenant({
				id: 'tenant-123',
				name: 'Test Tenant',
				domain: 'test.example.com',
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString()
			});

			expect(authLogic.getTenantId()).toBe('tenant-123');
		});

		it('should return null when no tenant is set', () => {
			expect(authLogic.getTenantId()).toBe(null);
		});
	});

	describe('logout', () => {
		it('should clear all auth state', () => {
			// Set up auth state
			authLogic.setUser({
				id: 'user-1',
				email: 'user@example.com',
				name: 'Test User',
				role: 'admin',
				tenantId: 'tenant-1',
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString()
			});

			authLogic.setTenant({
				id: 'tenant-1',
				name: 'Test Tenant',
				domain: 'test.example.com',
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString()
			});

			// Verify state is set
			expect(authLogic.getState().isAuthenticated).toBe(true);
			expect(authLogic.getState().user).toBeDefined();
			expect(authLogic.getState().tenant).toBeDefined();

			// Logout
			authLogic.logout();

			// Verify state is cleared
			expect(authLogic.getState().isAuthenticated).toBe(false);
			expect(authLogic.getState().user).toBe(null);
			expect(authLogic.getState().tenant).toBe(null);
			expect(authLogic.getState().isLoading).toBe(false);
		});
	});
});
