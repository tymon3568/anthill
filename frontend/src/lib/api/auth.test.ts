import { describe, it, expect, vi, beforeEach } from 'vitest';
import { authApi } from '$lib/api/auth';
import { apiClient } from '$lib/api/client';

// Mock the apiClient
vi.mock('$lib/api/client', () => ({
	apiClient: {
		get: vi.fn(),
		post: vi.fn(),
		put: vi.fn(),
		delete: vi.fn()
	}
}));

// Mock global fetch for authApiClient (uses fetch directly via proxy)
const mockFetch = vi.fn();
global.fetch = mockFetch;

describe('Auth API Client', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockFetch.mockReset();
	});

	describe('Email/Password Authentication (via proxy)', () => {
		it('should login with email credentials via proxy', async () => {
			const mockResponse = {
				access_token: 'token',
				refresh_token: 'refresh',
				token_type: 'Bearer',
				expires_in: 3600,
				user: { id: 'user-1', email: 'user@example.com', tenant_id: 'tenant-1', role: 'user' }
			};

			mockFetch.mockResolvedValue({
				ok: true,
				status: 200,
				headers: new Headers({ 'content-type': 'application/json' }),
				json: async () => mockResponse
			});

			const result = await authApi.emailLogin({ email: 'user@example.com', password: 'password' });

			expect(mockFetch).toHaveBeenCalledWith(
				'/api/v1/auth/login',
				expect.objectContaining({
					method: 'POST',
					credentials: 'include'
				})
			);
			expect(result.success).toBe(true);
			expect(result.data).toEqual(mockResponse);
		});

		it('should register with email via proxy', async () => {
			const mockResponse = {
				access_token: 'token',
				refresh_token: 'refresh',
				token_type: 'Bearer',
				expires_in: 3600,
				user: { id: 'user-1', email: 'new@example.com', tenant_id: 'tenant-1', role: 'user' }
			};

			mockFetch.mockResolvedValue({
				ok: true,
				status: 200,
				headers: new Headers({ 'content-type': 'application/json' }),
				json: async () => mockResponse
			});

			const result = await authApi.emailRegister({
				email: 'new@example.com',
				password: 'password',
				full_name: 'New User'
			});

			expect(mockFetch).toHaveBeenCalledWith(
				'/api/v1/auth/register',
				expect.objectContaining({
					method: 'POST',
					credentials: 'include'
				})
			);
			expect(result.success).toBe(true);
			expect(result.data).toEqual(mockResponse);
		});

		it('should refresh email token via proxy', async () => {
			const mockResponse = { access_token: 'new_token', refresh_token: 'new_refresh' };

			mockFetch.mockResolvedValue({
				ok: true,
				status: 200,
				headers: new Headers({ 'content-type': 'application/json' }),
				json: async () => mockResponse
			});

			const result = await authApi.refreshEmailToken();

			expect(mockFetch).toHaveBeenCalledWith(
				'/api/v1/auth/refresh',
				expect.objectContaining({
					method: 'POST',
					credentials: 'include'
				})
			);
			expect(result.success).toBe(true);
		});

		it('should logout via proxy', async () => {
			mockFetch.mockResolvedValue({
				ok: true,
				status: 204,
				headers: new Headers({ 'content-length': '0' })
			});

			const result = await authApi.logout('/dashboard');

			expect(mockFetch).toHaveBeenCalledWith(
				'/api/v1/auth/logout?redirect=%2Fdashboard',
				expect.objectContaining({
					method: 'POST',
					credentials: 'include'
				})
			);
			expect(result.success).toBe(true);
		});
	});

	describe('OAuth2 Deprecated Methods', () => {
		it('should return error for deprecated OAuth2 callback', async () => {
			const result = await authApi.handleOAuth2CallbackLegacy('code', 'state');

			expect(result.success).toBe(false);
			expect(result.error).toContain('OAuth2 integration has been removed');
		});

		it('should return error for deprecated OAuth2 refresh', async () => {
			const result = await authApi.refreshToken();

			expect(result.success).toBe(false);
			expect(result.error).toContain('OAuth2 integration has been removed');
		});
	});

	describe('User Profile (direct backend call)', () => {
		it('should get user profile', async () => {
			const mockProfile = {
				id: 'user-1',
				email: 'user@example.com',
				full_name: 'John Doe'
			};

			const mockResponse = { success: true, data: mockProfile };

			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await authApi.getProfile();

			expect(apiClient.get).toHaveBeenCalledWith('/users/profile');
			expect(result).toEqual(mockResponse);
		});

		it('should update user profile', async () => {
			const updateData = { display_name: 'Jane Doe' };
			const mockResponse = {
				success: true,
				data: { id: 'user-1', email: 'user@example.com', display_name: 'Jane Doe' }
			};

			vi.mocked(apiClient.put).mockResolvedValue(mockResponse);

			const result = await authApi.updateProfile(updateData);

			expect(apiClient.put).toHaveBeenCalledWith('/users/profile', updateData);
			expect(result).toEqual(mockResponse);
		});
	});

	describe('User Preferences (direct backend call)', () => {
		it('should get user preferences', async () => {
			const mockPreferences = {
				language: 'en',
				timezone: 'UTC',
				notification_preferences: {}
			};

			const mockResponse = { success: true, data: mockPreferences };

			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await authApi.getPreferences();

			expect(apiClient.get).toHaveBeenCalledWith('/users/preferences');
			expect(result).toEqual(mockResponse);
		});

		it('should update user preferences', async () => {
			const updateData = { language: 'es' };
			const mockResponse = {
				success: true,
				data: { language: 'es', timezone: 'UTC' }
			};

			vi.mocked(apiClient.put).mockResolvedValue(mockResponse);

			const result = await authApi.updatePreferences(updateData);

			expect(apiClient.put).toHaveBeenCalledWith('/users/preferences', updateData);
			expect(result).toEqual(mockResponse);
		});
	});

	describe('Permissions (direct backend call)', () => {
		it('should check user permission', async () => {
			const mockResponse = { success: true, data: { allowed: true } };

			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await authApi.checkPermission('products', 'read');

			expect(apiClient.get).toHaveBeenCalledWith(
				'/users/permissions/check?resource=products&action=read'
			);
			expect(result).toEqual(mockResponse);
		});

		it('should get user permissions', async () => {
			const mockResponse = {
				success: true,
				data: { roles: ['user'], permissions: ['read'] }
			};

			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await authApi.getUserPermissions();

			expect(apiClient.get).toHaveBeenCalledWith('/users/permissions');
			expect(result).toEqual(mockResponse);
		});

		it('should get user roles', async () => {
			const mockResponse = {
				success: true,
				data: { roles: ['user'], groups: ['users'] }
			};

			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await authApi.getUserRoles();

			expect(apiClient.get).toHaveBeenCalledWith('/users/roles');
			expect(result).toEqual(mockResponse);
		});
	});

	describe('Session Management (direct backend call)', () => {
		it('should validate session', async () => {
			const mockResponse = { success: true, data: { valid: true } };

			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await authApi.validateSession();

			expect(apiClient.get).toHaveBeenCalledWith('/users/session/validate');
			expect(result).toEqual(mockResponse);
		});

		it('should get session info', async () => {
			const mockResponse = {
				success: true,
				data: { user: { id: 'user-1' }, expires_at: '2024-01-01T00:00:00Z' }
			};

			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await authApi.getSessionInfo();

			expect(apiClient.get).toHaveBeenCalledWith('/users/session');
			expect(result).toEqual(mockResponse);
		});

		it('should get active sessions', async () => {
			const mockResponse = {
				success: true,
				data: { sessions: [{ id: 'session-1', created_at: '2024-01-01' }] }
			};

			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await authApi.getActiveSessions();

			expect(apiClient.get).toHaveBeenCalledWith('/users/sessions');
			expect(result).toEqual(mockResponse);
		});

		it('should terminate session', async () => {
			const mockResponse = { success: true, data: undefined };

			vi.mocked(apiClient.delete).mockResolvedValue(mockResponse);

			const result = await authApi.terminateSession('session-1');

			expect(apiClient.delete).toHaveBeenCalledWith('/users/sessions/session-1');
			expect(result).toEqual(mockResponse);
		});

		it('should end all sessions', async () => {
			const mockResponse = { success: true, data: undefined };

			vi.mocked(apiClient.delete).mockResolvedValue(mockResponse);

			const result = await authApi.endAllSessions();

			expect(apiClient.delete).toHaveBeenCalledWith('/users/sessions');
			expect(result).toEqual(mockResponse);
		});
	});
});
