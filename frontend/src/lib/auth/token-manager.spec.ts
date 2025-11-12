import { describe, it, expect, beforeEach, vi, afterEach, type MockedFunction } from 'vitest';
import { tokenManager } from './token-manager';
import { encryptToken, decryptToken } from './token-encryption';

// Mock browser environment - sessionStorage is already mocked in vitest-setup-server.ts
const mockBrowser = window.sessionStorage as any;

// Mock the encryption functions
vi.mock('./token-encryption', () => ({
	encryptToken: vi.fn(),
	decryptToken: vi.fn()
}));

const mockEncryptToken = vi.mocked(encryptToken);
const mockDecryptToken = vi.mocked(decryptToken);

// Mock browser detection
vi.mock('$app/environment', () => ({
	browser: true
}));

describe('Token Manager', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		// Reset token manager state
		tokenManager.clearAll();
		// Reset sessionStorage mocks
		mockBrowser.setItem.mockReset();
		mockBrowser.removeItem.mockReset();
		// Note: Don't reset getItem here as tests set it up individually
		// Set up encryption mocks
		mockEncryptToken.mockResolvedValue('encrypted-token');
		mockDecryptToken.mockResolvedValue('decrypted-data');
	});

	afterEach(() => {
		tokenManager.clearAll();
	});

	describe('setAccessToken and getAccessToken', () => {
		it('should store and retrieve access token', () => {
			const token = 'test-access-token';
			const expiresIn = 3600; // 1 hour

			tokenManager.setAccessToken(token, expiresIn);

			expect(tokenManager.getAccessToken()).toBe(token);
		});

		it('should return null when token is expired', () => {
			const token = 'test-access-token';
			const expiresIn = -1; // Already expired

			tokenManager.setAccessToken(token, expiresIn);

			expect(tokenManager.getAccessToken()).toBeNull();
		});

		it('should return null when no token is set', () => {
			expect(tokenManager.getAccessToken()).toBeNull();
		});
	});

	describe('isAccessTokenExpiringSoon', () => {
		it('should return true when token expires within 2 minutes', () => {
			const token = 'test-access-token';
			const expiresIn = 119; // 1 minute 59 seconds

			tokenManager.setAccessToken(token, expiresIn);

			expect(tokenManager.isAccessTokenExpiringSoon()).toBe(true);
		});

		it('should return false when token expires after 2 minutes', () => {
			const token = 'test-access-token';
			const expiresIn = 121; // 2 minutes 1 second

			tokenManager.setAccessToken(token, expiresIn);

			expect(tokenManager.isAccessTokenExpiringSoon()).toBe(false);
		});

		it('should return false when no token is set', () => {
			expect(tokenManager.isAccessTokenExpiringSoon()).toBe(false);
		});
	});

	describe('setRefreshToken and getRefreshToken', () => {
		it('should store and retrieve refresh token', async () => {
			const token = 'test-refresh-token';

			// Mock sessionStorage to return the encrypted token for refresh_token key
			mockBrowser.getItem.mockImplementation((key: string) => {
				if (key === 'refresh_token') return 'encrypted-token';
				return null;
			});

			await tokenManager.setRefreshToken(token);
			const retrieved = await tokenManager.getRefreshToken();

			expect(retrieved).toBe('decrypted-data');
		});

		it('should return null when no refresh token is stored', async () => {
			mockBrowser.getItem.mockReturnValue(null);

			const retrieved = await tokenManager.getRefreshToken();

			expect(retrieved).toBeNull();
		});

		it('should handle storage errors gracefully', async () => {
			mockBrowser.setItem.mockImplementation((key: string, value: string) => {
				throw new Error('Storage error');
			});

			await expect(tokenManager.setRefreshToken('token')).resolves.not.toThrow();

			mockBrowser.getItem.mockImplementation((key: string) => {
				throw new Error('Storage error');
			});

			const result = await tokenManager.getRefreshToken();
			expect(result).toBeNull();
		});
	});

	describe('setUserData and getUserData', () => {
		it('should store and retrieve user data', async () => {
			const userData = JSON.stringify({ id: '1', email: 'test@example.com' });

			// Mock sessionStorage to return the encrypted data for user_data key
			mockBrowser.getItem.mockImplementation((key: string) => {
				if (key === 'user_data') return 'encrypted-token';
				return null;
			});

			await tokenManager.setUserData(userData);
			const retrieved = await tokenManager.getUserData();

			expect(retrieved).toBe('decrypted-data');
		});

		it('should return null when no user data is stored', async () => {
			mockBrowser.getItem.mockReturnValue(null);

			const retrieved = await tokenManager.getUserData();

			expect(retrieved).toBeNull();
		});

		it('should handle storage errors gracefully', async () => {
			mockBrowser.setItem.mockImplementation((key: string, value: string) => {
				throw new Error('Storage error');
			});

			await expect(tokenManager.setUserData('data')).resolves.not.toThrow();

			mockBrowser.getItem.mockImplementation((key: string) => {
				throw new Error('Storage error');
			});

			const result = await tokenManager.getUserData();
			expect(result).toBeNull();
		});
	});

	describe('clearAll', () => {
		it('should clear all tokens and data', async () => {
			// Set up data
			tokenManager.setAccessToken('access-token', 3600);
			await tokenManager.setRefreshToken('refresh-token');
			await tokenManager.setUserData('user-data');

			// Mock sessionStorage to return encrypted data initially
			mockBrowser.getItem.mockImplementation((key: string) => {
				if (key === 'refresh_token' || key === 'user_data') return 'encrypted-token';
				return null;
			});

			// Verify data is set
			expect(tokenManager.getAccessToken()).toBe('access-token');
			expect(await tokenManager.getRefreshToken()).toBe('decrypted-data');
			expect(await tokenManager.getUserData()).toBe('decrypted-data');

			// Clear all
			tokenManager.clearAll();

			// Mock sessionStorage to return null after clearing
			mockBrowser.getItem.mockReturnValue(null);

			// Verify all are cleared
			expect(tokenManager.getAccessToken()).toBeNull();
			expect(await tokenManager.getRefreshToken()).toBeNull();
			expect(await tokenManager.getUserData()).toBeNull();
		});

		it('should handle storage errors during clear', () => {
			mockBrowser.removeItem.mockImplementation((key: string) => {
				throw new Error('Storage error');
			});

			expect(() => tokenManager.clearAll()).not.toThrow();
		});
	});

	describe('hasValidSession', () => {
		it('should return true when refresh token exists', async () => {
			// Mock sessionStorage to return encrypted token
			mockBrowser.getItem.mockImplementation((key: string) => {
				if (key === 'refresh_token') return 'encrypted-token';
				return null;
			});

			await tokenManager.setRefreshToken('refresh-token');

			const hasSession = await tokenManager.hasValidSession();

			expect(hasSession).toBe(true);
		});

		it('should return false when no refresh token exists', async () => {
			mockBrowser.getItem.mockReturnValue(null);

			const hasSession = await tokenManager.hasValidSession();

			expect(hasSession).toBe(false);
		});

		it('should handle storage errors', async () => {
			mockBrowser.getItem.mockImplementation(() => {
				throw new Error('Storage error');
			});

			const hasSession = await tokenManager.hasValidSession();

			expect(hasSession).toBe(false);
		});
	});

	describe('getTimeUntilExpiration', () => {
		it('should return remaining time when token is set', () => {
			tokenManager.setAccessToken('token', 3600); // 1 hour

			const timeUntilExpiration = tokenManager.getTimeUntilExpiration();

			expect(timeUntilExpiration).toBeGreaterThan(3500); // Should be close to 3600
			expect(timeUntilExpiration).toBeLessThanOrEqual(3600);
		});

		it('should return null when no token is set', () => {
			const timeUntilExpiration = tokenManager.getTimeUntilExpiration();

			expect(timeUntilExpiration).toBeNull();
		});
	});
});
