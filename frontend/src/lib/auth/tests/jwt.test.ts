import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
	decodeJwtPayload,
	validateAndParseToken,
	extractTenantFromGroups,
	isTokenExpired,
	getTokenExpiry,
	shouldRefreshToken
} from '../jwt';

// Mock fetch for JWT signature verification
global.fetch = vi.fn();

describe('JWT Utilities', () => {
	// Create a valid JWT for testing
	const createMockJwt = (expOffset: number = 3600) => {
		const futureExp = Math.floor(Date.now() / 1000) + expOffset; // exp in seconds
		const payload = {
			sub: 'user-123',
			email: 'user@example.com',
			groups: ['tenant_acme_users', 'tenant_xyz_users'],
			exp: futureExp,
			iat: Math.floor(Date.now() / 1000) - 60
		};
		const header = 'eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9';
		const payloadB64 = btoa(JSON.stringify(payload)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
		const signature = 'signature';
		return `${header}.${payloadB64}.${signature}`;
	};

	const mockJwt = createMockJwt(); // Valid JWT expiring in 1 hour

	const mockPayload = {
		sub: 'user-123',
		email: 'user@example.com',
		groups: ['tenant_acme_users', 'tenant_xyz_users'],
		exp: Math.floor(Date.now() / 1000) + 3600, // Future timestamp
		iat: Math.floor(Date.now() / 1000) - 60
	};

	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('decodeJwtPayload', () => {
		it('should decode valid JWT payload', () => {
			// Create a proper JWT with base64url encoded payload
			const header = 'eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9';
			const payload = btoa(JSON.stringify(mockPayload)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
			const signature = 'signature';
			const validJwt = `${header}.${payload}.${signature}`;

			const result = decodeJwtPayload(validJwt);
			expect(result).toEqual(mockPayload);
		});

		it('should return null for invalid JWT', () => {
			const result = decodeJwtPayload('invalid.jwt');
			expect(result).toBeNull();
		});

		it('should handle malformed base64', () => {
			const malformedJwt = 'header.invalid_payload.signature';
			const result = decodeJwtPayload(malformedJwt);
			expect(result).toBeNull();
		});
	});

	describe('validateAndParseToken', () => {
		it('should validate and parse valid token', async () => {
			const result = await validateAndParseToken(mockJwt, false); // Don't verify signature in test
			expect(result).toBeDefined();
			expect(result?.userId).toBe('user-123');
			expect(result?.email).toBe('user@example.com');
			expect(result?.tenantId).toBe('acme');
		});

		it('should return null for expired token', async () => {
			const expiredPayload = { ...mockPayload, exp: Math.floor(Date.now() / 1000) - 3600 };
			const expiredJwt = 'header.' + btoa(JSON.stringify(expiredPayload)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '') + '.signature';

			const result = await validateAndParseToken(expiredJwt, false);
			expect(result).toBeNull();
		});

		it('should return null for token without required claims', async () => {
			const invalidPayload = { groups: [] }; // Missing sub and email
			const invalidJwt = 'header.' + btoa(JSON.stringify(invalidPayload)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '') + '.signature';

			const result = await validateAndParseToken(invalidJwt, false);
			expect(result).toBeNull();
		});
	});

	describe('extractTenantFromGroups', () => {
		it('should extract tenant from groups', () => {
			const groups = ['tenant_acme_users', 'tenant_xyz_admins', 'other_group'];
			const result = extractTenantFromGroups(groups);
			expect(result).toBe('acme');
		});

		it('should return undefined when no tenant group found', () => {
			const groups = ['admin_users', 'regular_users'];
			const result = extractTenantFromGroups(groups);
			expect(result).toBeUndefined();
		});

		it('should handle empty groups array', () => {
			const result = extractTenantFromGroups([]);
			expect(result).toBeUndefined();
		});

		it('should handle undefined groups', () => {
			const result = extractTenantFromGroups(undefined as any);
			expect(result).toBeUndefined();
		});
	});

	describe('isTokenExpired', () => {
		it('should return true for expired token', () => {
			const expiredPayload = { exp: Math.floor(Date.now() / 1000) - 3600 };
			const expiredJwt = 'header.' + btoa(JSON.stringify(expiredPayload)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '') + '.signature';

			const result = isTokenExpired(expiredJwt);
			expect(result).toBe(true);
		});

		it('should return false for valid token', () => {
			const result = isTokenExpired(mockJwt);
			expect(result).toBe(false);
		});

		it('should return true for token without exp claim', () => {
			const noExpJwt = 'header.' + btoa(JSON.stringify({ sub: 'user' })).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '') + '.signature';

			const result = isTokenExpired(noExpJwt);
			expect(result).toBe(true);
		});
	});

	describe('getTokenExpiry', () => {
		it('should return expiry timestamp', () => {
			const result = getTokenExpiry(mockJwt);
			expect(result).toBeGreaterThan(Date.now()); // Should be in the future
			expect(typeof result).toBe('number');
		});

		it('should return null for invalid token', () => {
			const result = getTokenExpiry('invalid.jwt');
			expect(result).toBeNull();
		});
	});

	describe('shouldRefreshToken', () => {
		it('should return true for token expiring soon', () => {
			// Token expiring in 2 minutes
			const soonExpiry = Math.floor(Date.now() / 1000) + 120;
			const soonPayload = { ...mockPayload, exp: soonExpiry };
			const soonJwt = 'header.' + btoa(JSON.stringify(soonPayload)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '') + '.signature';

			const result = shouldRefreshToken(soonJwt);
			expect(result).toBe(true);
		});

		it('should return false for token with plenty of time', () => {
			const result = shouldRefreshToken(mockJwt);
			expect(result).toBe(false);
		});

		it('should return true for invalid token', () => {
			const result = shouldRefreshToken('invalid.jwt');
			expect(result).toBe(true);
		});
	});
});
