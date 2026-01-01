import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import {
	parseTenantFromHostname,
	getCurrentTenantSlug,
	setPersistedTenantSlug,
	clearPersistedTenantSlug,
	hasTenantContext,
	getTenantContext
} from './index';

// Mock $app/environment
vi.mock('$app/environment', () => ({
	browser: true
}));

describe('Tenant Context Utilities', () => {
	describe('parseTenantFromHostname', () => {
		it('should return null for plain localhost', () => {
			expect(parseTenantFromHostname('localhost')).toBeNull();
			expect(parseTenantFromHostname('localhost:5173')).toBeNull();
		});

		it('should return null for 127.0.0.1', () => {
			expect(parseTenantFromHostname('127.0.0.1')).toBeNull();
			expect(parseTenantFromHostname('127.0.0.1:8000')).toBeNull();
		});

		it('should extract subdomain from *.localhost pattern', () => {
			expect(parseTenantFromHostname('acme.localhost')).toBe('acme');
			expect(parseTenantFromHostname('acme.localhost:5173')).toBe('acme');
			expect(parseTenantFromHostname('demo.localhost:8000')).toBe('demo');
		});

		it('should ignore www subdomain for localhost', () => {
			expect(parseTenantFromHostname('www.localhost')).toBeNull();
		});

		it('should extract subdomain from production domains', () => {
			expect(parseTenantFromHostname('acme.anthill.example.com')).toBe('acme');
			expect(parseTenantFromHostname('tenant1.app.anthill.io')).toBe('tenant1');
		});

		it('should ignore www subdomain for production domains', () => {
			expect(parseTenantFromHostname('www.anthill.example.com')).toBeNull();
		});

		it('should return null for domains without subdomain', () => {
			expect(parseTenantFromHostname('anthill.com')).toBeNull();
			expect(parseTenantFromHostname('example.com')).toBeNull();
		});

		it('should handle empty string', () => {
			expect(parseTenantFromHostname('')).toBeNull();
		});

		it('should handle port in hostname', () => {
			expect(parseTenantFromHostname('acme.localhost:3000')).toBe('acme');
			expect(parseTenantFromHostname('tenant.anthill.io:443')).toBe('tenant');
		});
	});

	describe('localStorage operations', () => {
		const originalLocalStorage = globalThis.localStorage;
		let mockStorage: Record<string, string>;

		beforeEach(() => {
			mockStorage = {};
			const mockLocalStorage = {
				getItem: vi.fn((key: string) => mockStorage[key] ?? null),
				setItem: vi.fn((key: string, value: string) => {
					mockStorage[key] = value;
				}),
				removeItem: vi.fn((key: string) => {
					delete mockStorage[key];
				}),
				clear: vi.fn(() => {
					mockStorage = {};
				}),
				length: 0,
				key: vi.fn(() => null)
			};
			Object.defineProperty(globalThis, 'localStorage', {
				value: mockLocalStorage,
				writable: true
			});
		});

		afterEach(() => {
			Object.defineProperty(globalThis, 'localStorage', {
				value: originalLocalStorage,
				writable: true
			});
		});

		describe('setPersistedTenantSlug', () => {
			it('should store tenant slug in localStorage', () => {
				setPersistedTenantSlug('acme');
				expect(localStorage.setItem).toHaveBeenCalledWith('anthill_tenant_slug', 'acme');
			});
		});

		describe('clearPersistedTenantSlug', () => {
			it('should remove tenant slug from localStorage', () => {
				clearPersistedTenantSlug();
				expect(localStorage.removeItem).toHaveBeenCalledWith('anthill_tenant_slug');
			});
		});
	});

	describe('getCurrentTenantSlug', () => {
		const originalWindow = globalThis.window;
		const originalLocalStorage = globalThis.localStorage;

		beforeEach(() => {
			// Reset mocks
			vi.resetAllMocks();
		});

		afterEach(() => {
			Object.defineProperty(globalThis, 'window', {
				value: originalWindow,
				writable: true
			});
			Object.defineProperty(globalThis, 'localStorage', {
				value: originalLocalStorage,
				writable: true
			});
		});

		it('should return subdomain tenant if available', () => {
			// Mock window.location
			Object.defineProperty(globalThis, 'window', {
				value: {
					location: {
						hostname: 'acme.localhost'
					}
				},
				writable: true
			});

			const result = getCurrentTenantSlug();
			expect(result).toBe('acme');
		});

		it('should fall back to localStorage if no subdomain', () => {
			Object.defineProperty(globalThis, 'window', {
				value: {
					location: {
						hostname: 'localhost'
					}
				},
				writable: true
			});

			const mockLocalStorage = {
				getItem: vi.fn(() => 'stored-tenant'),
				setItem: vi.fn(),
				removeItem: vi.fn(),
				clear: vi.fn(),
				length: 0,
				key: vi.fn(() => null)
			};
			Object.defineProperty(globalThis, 'localStorage', {
				value: mockLocalStorage,
				writable: true
			});

			const result = getCurrentTenantSlug();
			expect(result).toBe('stored-tenant');
		});

		it('should return null if no tenant available', () => {
			Object.defineProperty(globalThis, 'window', {
				value: {
					location: {
						hostname: 'localhost'
					}
				},
				writable: true
			});

			const mockLocalStorage = {
				getItem: vi.fn(() => null),
				setItem: vi.fn(),
				removeItem: vi.fn(),
				clear: vi.fn(),
				length: 0,
				key: vi.fn(() => null)
			};
			Object.defineProperty(globalThis, 'localStorage', {
				value: mockLocalStorage,
				writable: true
			});

			const result = getCurrentTenantSlug();
			expect(result).toBeNull();
		});
	});

	describe('hasTenantContext', () => {
		beforeEach(() => {
			Object.defineProperty(globalThis, 'window', {
				value: {
					location: {
						hostname: 'acme.localhost'
					}
				},
				writable: true
			});
		});

		it('should return true when tenant is available', () => {
			expect(hasTenantContext()).toBe(true);
		});

		it('should return false when no tenant is available', () => {
			Object.defineProperty(globalThis, 'window', {
				value: {
					location: {
						hostname: 'localhost'
					}
				},
				writable: true
			});

			const mockLocalStorage = {
				getItem: vi.fn(() => null),
				setItem: vi.fn(),
				removeItem: vi.fn(),
				clear: vi.fn(),
				length: 0,
				key: vi.fn(() => null)
			};
			Object.defineProperty(globalThis, 'localStorage', {
				value: mockLocalStorage,
				writable: true
			});

			expect(hasTenantContext()).toBe(false);
		});
	});

	describe('getTenantContext', () => {
		const originalWindow = globalThis.window;
		const originalLocalStorage = globalThis.localStorage;

		afterEach(() => {
			Object.defineProperty(globalThis, 'window', {
				value: originalWindow,
				writable: true
			});
			Object.defineProperty(globalThis, 'localStorage', {
				value: originalLocalStorage,
				writable: true
			});
		});

		it('should return subdomain context when available', () => {
			Object.defineProperty(globalThis, 'window', {
				value: {
					location: {
						hostname: 'acme.localhost'
					}
				},
				writable: true
			});

			const context = getTenantContext();
			expect(context).toEqual({
				slug: 'acme',
				source: 'subdomain',
				hasContext: true
			});
		});

		it('should return storage context when no subdomain', () => {
			Object.defineProperty(globalThis, 'window', {
				value: {
					location: {
						hostname: 'localhost'
					}
				},
				writable: true
			});

			const mockLocalStorage = {
				getItem: vi.fn(() => 'stored-tenant'),
				setItem: vi.fn(),
				removeItem: vi.fn(),
				clear: vi.fn(),
				length: 0,
				key: vi.fn(() => null)
			};
			Object.defineProperty(globalThis, 'localStorage', {
				value: mockLocalStorage,
				writable: true
			});

			const context = getTenantContext();
			expect(context).toEqual({
				slug: 'stored-tenant',
				source: 'storage',
				hasContext: true
			});
		});

		it('should return empty context when no tenant available', () => {
			Object.defineProperty(globalThis, 'window', {
				value: {
					location: {
						hostname: 'localhost'
					}
				},
				writable: true
			});

			const mockLocalStorage = {
				getItem: vi.fn(() => null),
				setItem: vi.fn(),
				removeItem: vi.fn(),
				clear: vi.fn(),
				length: 0,
				key: vi.fn(() => null)
			};
			Object.defineProperty(globalThis, 'localStorage', {
				value: mockLocalStorage,
				writable: true
			});

			const context = getTenantContext();
			expect(context).toEqual({
				slug: null,
				source: null,
				hasContext: false
			});
		});
	});
});
