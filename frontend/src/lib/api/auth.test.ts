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

describe('Auth API Client', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('Email/Password Authentication', () => {
		it('should login with email credentials', async () => {
			const mockResponse = {
				success: true,
				data: {
					access_token: 'token',
					refresh_token: 'refresh',
					token_type: 'Bearer',
					expires_in: 3600,
					user: { id: 'user-1', email: 'user@example.com', tenant_id: 'tenant-1', role: 'user' }
				}
			};

			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await authApi.emailLogin({ email: 'user@example.com', password: 'password' });

			expect(apiClient.post).toHaveBeenCalledWith('/auth/login', {
				email: 'user@example.com',
				password: 'password'
			});
			expect(result).toEqual(mockResponse);
		});

		it('should register with email', async () => {
			const mockResponse = {
				success: true,
				data: {
					access_token: 'token',
					refresh_token: 'refresh',
					token_type: 'Bearer',
					expires_in: 3600,
					user: { id: 'user-1', email: 'new@example.com', tenant_id: 'tenant-1', role: 'user' }
				}
			};

			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await authApi.emailRegister({
				email: 'new@example.com',
				password: 'password',
				full_name: 'New User'
			});

			expect(apiClient.post).toHaveBeenCalledWith('/auth/register', {
				email: 'new@example.com',
				password: 'password',
				full_name: 'New User'
			});
			expect(result).toEqual(mockResponse);
		});

		it('should refresh email token', async () => {
			const mockResponse = {
				success: true,
				data: { access_token: 'new_token', refresh_token: 'new_refresh' }
			};

			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await authApi.refreshEmailToken('old_refresh_token');

			expect(apiClient.post).toHaveBeenCalledWith('/auth/refresh', {
				refresh_token: 'old_refresh_token'
			});
			expect(result).toEqual(mockResponse);
		});

		it('should logout', async () => {
			const mockResponse = { success: true, data: undefined };

			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await authApi.logout('/dashboard');

			expect(apiClient.post).toHaveBeenCalledWith('/auth/logout?redirect=%2Fdashboard');
			expect(result).toEqual(mockResponse);
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

	describe('User Profile', () => {
		it('should get user profile', async () => {
			const mockProfile = {
				id: 'user-1',
				email: 'user@example.com',
				full_name: 'John Doe'
			};

			const mockResponse = { success: true, data: mockProfile };

			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await authApi.getProfile();

			expect(apiClient.get).toHaveBeenCalledWith('/auth/profile');
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

			expect(apiClient.put).toHaveBeenCalledWith('/auth/profile', updateData);
			expect(result).toEqual(mockResponse);
		});
	});

	describe('User Preferences', () => {
		it('should get user preferences', async () => {
			const mockPreferences = {
				language: 'en',
				timezone: 'UTC',
				notification_preferences: {}
			};

			const mockResponse = { success: true, data: mockPreferences };

			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await authApi.getPreferences();

			expect(apiClient.get).toHaveBeenCalledWith('/auth/preferences');
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

			expect(apiClient.put).toHaveBeenCalledWith('/auth/preferences', updateData);
			expect(result).toEqual(mockResponse);
		});
	});

	describe('Permissions', () => {
		it('should check user permission', async () => {
			const mockResponse = { success: true, data: { allowed: true } };

			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await authApi.checkPermission('products', 'read');

			expect(apiClient.get).toHaveBeenCalledWith(
				'/auth/permissions/check?resource=products&action=read'
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

			expect(apiClient.get).toHaveBeenCalledWith('/auth/permissions');
			expect(result).toEqual(mockResponse);
		});

		it('should get user roles', async () => {
			const mockResponse = {
				success: true,
				data: { roles: ['user'], groups: ['users'] }
			};

			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await authApi.getUserRoles();

			expect(apiClient.get).toHaveBeenCalledWith('/auth/roles');
			expect(result).toEqual(mockResponse);
		});
	});

	describe('Session Management', () => {
		it('should validate session', async () => {
			const mockResponse = { success: true, data: { valid: true } };

			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await authApi.validateSession();

			expect(apiClient.get).toHaveBeenCalledWith('/auth/session/validate');
			expect(result).toEqual(mockResponse);
		});

		it('should get session info', async () => {
			const mockResponse = {
				success: true,
				data: { user: { id: 'user-1' }, expires_at: '2024-01-01T00:00:00Z' }
			};

			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await authApi.getSessionInfo();

			expect(apiClient.get).toHaveBeenCalledWith('/auth/session');
			expect(result).toEqual(mockResponse);
		});

		it('should get active sessions', async () => {
			const mockResponse = {
				success: true,
				data: { sessions: [{ id: 'session-1', created_at: '2024-01-01' }] }
			};

			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await authApi.getActiveSessions();

			expect(apiClient.get).toHaveBeenCalledWith('/auth/sessions');
			expect(result).toEqual(mockResponse);
		});

		it('should terminate session', async () => {
			const mockResponse = { success: true, data: undefined };

			vi.mocked(apiClient.delete).mockResolvedValue(mockResponse);

			const result = await authApi.terminateSession('session-1');

			expect(apiClient.delete).toHaveBeenCalledWith('/auth/sessions/session-1');
			expect(result).toEqual(mockResponse);
		});

		it('should end all sessions', async () => {
			const mockResponse = { success: true, data: undefined };

			vi.mocked(apiClient.delete).mockResolvedValue(mockResponse);

			const result = await authApi.endAllSessions();

			expect(apiClient.delete).toHaveBeenCalledWith('/auth/sessions');
			expect(result).toEqual(mockResponse);
		});
	});
});
