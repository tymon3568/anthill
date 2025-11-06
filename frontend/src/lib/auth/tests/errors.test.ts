import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock SvelteKit redirect
const mockRedirect = vi.fn();
vi.mock('@sveltejs/kit', () => ({
	redirect: mockRedirect
}));

import { AuthError, AuthErrorCode, createAuthError, handleAuthError } from '../errors';

describe('AuthError Class', () => {
	it('should create AuthError with correct properties', () => {
		const error = new AuthError('Test message', AuthErrorCode.INVALID_TOKEN, 401);

		expect(error.message).toBe('Test message');
		expect(error.code).toBe(AuthErrorCode.INVALID_TOKEN);
		expect(error.statusCode).toBe(401);
		expect(error.name).toBe('AuthError');
	});
});

describe('createAuthError', () => {
	it('should create error with default message', () => {
		const error = createAuthError(AuthErrorCode.INVALID_TOKEN);

		expect(error.code).toBe(AuthErrorCode.INVALID_TOKEN);
		expect(error.message).toBe('Invalid or malformed token');
		expect(error.statusCode).toBe(401);
	});

	it('should create error with custom message', () => {
		const error = createAuthError(AuthErrorCode.NETWORK_ERROR, 'Custom network error');

		expect(error.code).toBe(AuthErrorCode.NETWORK_ERROR);
		expect(error.message).toBe('Custom network error');
		expect(error.statusCode).toBe(503);
	});

	it('should handle all error codes', () => {
		Object.values(AuthErrorCode).forEach(code => {
			const error = createAuthError(code);
			expect(error.code).toBe(code);
			expect(error.message).toBeDefined();
			expect(error.statusCode).toBeDefined();
		});
	});
});

describe('handleAuthError', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should handle AuthError instances', () => {
		const authError = createAuthError(AuthErrorCode.INVALID_TOKEN);

		expect(() => {
			handleAuthError(authError, '/login');
		}).toThrow();
		expect(mockRedirect).toHaveBeenCalledWith(302, '/login?error=invalid_token&message=Invalid%2520or%2520malformed%2520token');
	});

	it('should handle generic Error instances', () => {
		const error = new Error('Network fetch failed');

		expect(() => {
			handleAuthError(error, '/login');
		}).toThrow();
		expect(mockRedirect).toHaveBeenCalledWith(302, '/login?error=network_error&message=Network%2520error%2520occurred');
	});

	it('should handle unknown errors', () => {
		expect(() => {
			handleAuthError('string error', '/login');
		}).toThrow();
		expect(mockRedirect).toHaveBeenCalledWith(302, '/login?error=unknown_error&message=An%2520unexpected%2520error%2520occurred');
	});

	it('should use default redirect path', () => {
		expect(() => {
			handleAuthError(new Error('test'), '/custom-login');
		}).toThrow();
		expect(mockRedirect).toHaveBeenCalledWith(302, '/custom-login?error=unknown_error&message=test');
	});
});
