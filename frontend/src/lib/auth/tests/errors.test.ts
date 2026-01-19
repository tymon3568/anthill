import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock SvelteKit redirect before importing
vi.mock('@sveltejs/kit', () => ({
	redirect: vi.fn()
}));

// Import after mocking
import { AuthError, AuthErrorCode, createAuthError, handleAuthError } from '../errors';
import { redirect } from '@sveltejs/kit';

// Get the mocked redirect function
const mockRedirect = vi.mocked(redirect);

describe('AuthError Class', () => {
	it('should create AuthError with correct properties', () => {
		const error = new AuthError(AuthErrorCode.INVALID_TOKEN, 'Test message', 401);

		expect(error.message).toBe('Test message');
		expect(error.code).toBe(AuthErrorCode.INVALID_TOKEN);
		expect(error.status).toBe(401);
		expect(error.name).toBe('AuthError');
	});
});

describe('createAuthError', () => {
	it('should create error with default message', () => {
		const error = createAuthError(AuthErrorCode.INVALID_TOKEN);

		expect(error.code).toBe(AuthErrorCode.INVALID_TOKEN);
		expect(error.message).toBe('Invalid authentication token');
		expect(error.status).toBe(401);
	});

	it('should create error with custom message', () => {
		const error = createAuthError(AuthErrorCode.NETWORK_ERROR, 'Custom network error');

		expect(error.code).toBe(AuthErrorCode.NETWORK_ERROR);
		expect(error.message).toBe('Custom network error');
		expect(error.status).toBe(401); // Default status
	});

	it('should handle all error codes', () => {
		Object.values(AuthErrorCode).forEach((code) => {
			const error = createAuthError(code);
			expect(error.code).toBe(code);
			expect(error.message).toBeDefined();
			expect(error.status).toBeDefined();
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
		expect(mockRedirect).toHaveBeenCalledWith(
			302,
			'/login?error=invalid_token&message=Invalid+authentication+token'
		);
	});

	it('should handle generic Error instances', () => {
		const error = new Error('Network fetch failed');

		expect(() => {
			handleAuthError(error, '/login');
		}).toThrow();
		expect(mockRedirect).toHaveBeenCalledWith(
			302,
			'/login?error=network_error&message=Network+fetch+failed'
		);
	});

	it('should handle unknown errors', () => {
		expect(() => {
			handleAuthError('string error', '/login');
		}).toThrow();
		expect(mockRedirect).toHaveBeenCalledWith(
			302,
			'/login?error=network_error&message=string+error'
		);
	});

	it('should use default redirect path', () => {
		expect(() => {
			handleAuthError(new Error('test'), '/custom-login');
		}).toThrow();
		expect(mockRedirect).toHaveBeenCalledWith(
			302,
			'/custom-login?error=network_error&message=test'
		);
	});
});
