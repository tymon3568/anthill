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

	describe('OAuth2 Flow', () => {
		it('should initiate OAuth2 login', () => {
			// This method redirects, so we can't easily test it
			// The actual redirect is handled by the browser
			expect(typeof authApi.initiateOAuth2Login).toBe('function');
		});

		it('should handle OAuth2 callback', async () => {
			const mockResponse = {
				success: true,
				data: { access_token: 'token', refresh_token: 'refresh' }
			};

			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await authApi.handleOAuth2Callback('code', 'state');

			expect(apiClient.get).toHaveBeenCalledWith('/auth/oauth/callback?code=code&state=state');
			expect(result).toEqual(mockResponse);
		});

		it('should refresh token', async () => {
			const mockResponse = {
				success: true,
				data: { access_token: 'new_token', refresh_token: 'new_refresh' }
			};

			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await authApi.refreshToken();

			expect(apiClient.post).toHaveBeenCalledWith('/auth/oauth/refresh');
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

			expect(apiClient.get).toHaveBeenCalledWith('/users/profile');
			expect(result).toEqual(mockResponse);
		});

		it('should update user profile', async () => {
			const updateData = { full_name: 'Jane Doe' };
			const mockResponse = {
				success: true,
				data: { id: 'user-1', email: 'user@example.com', full_name: 'Jane Doe' }
			};

			vi.mocked(apiClient.put).mockResolvedValue(mockResponse);

			const result = await authApi.updateProfile(updateData);

			expect(apiClient.put).toHaveBeenCalledWith('/users/profile', updateData);
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

	describe('Permissions', () => {
		it('should check user permission', async () => {
			const mockResponse = { success: true, data: { allowed: true } };

			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await authApi.checkPermission('products', 'read');

			expect(apiClient.get).toHaveBeenCalledWith('/users/permissions/check?resource=products&action=read');
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

	describe('Session Management', () => {
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
