import { apiClient } from './client';
import type { ApiResponse, User, LoginForm } from '$lib/types';
import { AuthError, AuthErrorCode, createAuthError } from '$lib/auth/errors';

// Backend UserInfo from login/register response
export interface BackendUserInfo {
	id: string;
	email: string;
	full_name?: string;
	tenant_id: string;
	role: string;
	created_at: string;
}

export interface AuthResponse {
	user: BackendUserInfo;
	access_token: string;
	refresh_token: string;
	token_type: string;
	expires_in: number;
}

export interface OAuth2Tokens {
	access_token: string;
	refresh_token?: string;
	expires_in?: number;
	token_type?: string;
}

export interface UserProfile {
	id: string;
	email: string;
	username: string;
	display_name?: string;
	tenant_id: string;
	roles: string[];
	permissions: string[];
	created_at: string;
	updated_at: string;
	// Extended profile fields from backend spec
	full_name?: string;
	phone?: string;
	avatar_url?: string;
	role?: string;
	email_verified?: boolean;
	bio?: string;
	title?: string;
	department?: string;
	location?: string;
	website_url?: string;
	social_links?: Record<string, string>;
	language?: string;
	timezone?: string;
	date_format?: string;
	time_format?: string;
	notification_preferences?: UserPreferences['notification_preferences'];
	profile_visibility?: UserPreferences['profile_visibility'];
	show_email?: boolean;
	show_phone?: boolean;
	completeness_score?: number;
	verified?: boolean;
	verification_badge?: string | null;
	custom_fields?: Record<string, any>;
}

export interface UserPreferences {
	language: string;
	timezone: string;
	date_format: string;
	time_format: string;
	notification_preferences: {
		email_notifications: boolean;
		push_notifications: boolean;
		sms_notifications: boolean;
		notification_types: {
			order_updates: boolean;
			inventory_alerts: boolean;
			system_announcements: boolean;
			security_alerts: boolean;
			marketing_emails: boolean;
		};
	};
	profile_visibility: 'public' | 'private' | 'team_only';
	show_email: boolean;
	show_phone: boolean;
}

export const authApi = {
	// OAuth2 Flow Methods
	async initiateOAuth2Login(): Promise<never> {
		// This will redirect to Kanidm OAuth2 authorize endpoint
		// The actual implementation is in the server endpoint
		window.location.href = '/api/v1/auth/oauth/authorize';
		// Return a promise that never resolves since navigation is in progress
		return new Promise<never>(() => {
			// Intentionally left unresolved; navigation is in progress.
		});
	},

	async handleOAuth2Callback(code: string, state: string): Promise<ApiResponse<OAuth2Tokens>> {
		// This is handled by the server endpoint, but we can provide a client method
		// for programmatic access if needed
		const params = new URLSearchParams({ code, state });
		return apiClient.get<OAuth2Tokens>(`/auth/oauth/callback?${params.toString()}`);
	},

	async refreshToken(): Promise<ApiResponse<OAuth2Tokens>> {
		return apiClient.post<OAuth2Tokens>('/auth/oauth/refresh');
	},

	async logout(redirectTo?: string): Promise<ApiResponse<void>> {
		const params = redirectTo ? `?redirect=${encodeURIComponent(redirectTo)}` : '';
		return apiClient.post<void>(`/auth/logout${params}`);
	},

	// Legacy methods (for backward compatibility)
	async login(credentials: LoginForm): Promise<ApiResponse<AuthResponse>> {
		return apiClient.post<AuthResponse>('/auth/login', credentials as unknown as Record<string, unknown>);
	},

	async register(userData: { full_name: string; email: string; password: string; tenant_name?: string }): Promise<ApiResponse<AuthResponse>> {
		return apiClient.post<AuthResponse>('/auth/register', userData as unknown as Record<string, unknown>);
	},

	async refreshTokenLegacy(
		refreshToken: string
	): Promise<ApiResponse<AuthResponse>> {
		return apiClient.post<AuthResponse>('/auth/refresh', { refresh_token: refreshToken });
	},

	async logoutLegacy(refreshToken?: { refresh_token: string }): Promise<ApiResponse<void>> {
		const data = refreshToken ? refreshToken : {};
		return apiClient.post('/auth/logout', data);
	},

	// User Profile Management (matches backend API spec)
	async getProfile(): Promise<ApiResponse<UserProfile>> {
		try {
			return await apiClient.get<UserProfile>('/users/profile');
		} catch (error) {
			// Re-throw existing AuthError instances to preserve error codes
			if (error instanceof AuthError) {
				throw error;
			}
			throw createAuthError(AuthErrorCode.PROFILE_FETCH_FAILED, 'Failed to fetch user profile');
		}
	},

	async updateProfile(userData: Partial<UserProfile>): Promise<ApiResponse<UserProfile>> {
		try {
			return await apiClient.put<UserProfile>('/users/profile', userData);
		} catch (error) {
			// Re-throw existing AuthError instances to preserve error codes
			if (error instanceof AuthError) {
				throw error;
			}
			throw createAuthError(AuthErrorCode.PROFILE_UPDATE_FAILED, 'Failed to update user profile');
		}
	},

	async uploadProfileImage(file: File): Promise<ApiResponse<{ image_url: string }>> {
		const formData = new FormData();
		formData.append('image', file);

		// Get API base URL from environment
		const API_BASE_URL = import.meta.env.PUBLIC_API_BASE_URL || 'http://localhost:8000/api/v1';

		// Custom request for file upload
		const response = await fetch(`${API_BASE_URL}/auth/profile/image`, {
			method: 'POST',
			body: formData,
			credentials: 'include', // Include cookies for auth
		});

		if (!response.ok) {
			const errorData = await response.json().catch(() => ({ message: 'Upload failed' }));
			return {
				success: false,
				error: errorData.message || `HTTP ${response.status}`
			};
		}

		const data = await response.json();
		return {
			success: true,
			data
		};
	},

	// User Preferences
	async getPreferences(): Promise<ApiResponse<UserPreferences>> {
		try {
			return await apiClient.get<UserPreferences>('/users/preferences');
		} catch (error) {
			// Re-throw existing AuthError instances to preserve error codes
			if (error instanceof AuthError) {
				throw error;
			}
			throw createAuthError(AuthErrorCode.PREFERENCES_FETCH_FAILED, 'Failed to fetch user preferences');
		}
	},

	async updatePreferences(preferences: Partial<UserPreferences>): Promise<ApiResponse<UserPreferences>> {
		try {
			return await apiClient.put<UserPreferences>('/users/preferences', preferences);
		} catch (error) {
			// Re-throw existing AuthError instances to preserve error codes
			if (error instanceof AuthError) {
				throw error;
			}
			throw createAuthError(AuthErrorCode.PREFERENCES_UPDATE_FAILED, 'Failed to update user preferences');
		}
	},

	// Permission Checking
	async checkPermission(resource: string, action: string): Promise<ApiResponse<{ allowed: boolean }>> {
		try {
			return await apiClient.get<{ allowed: boolean }>(`/users/permissions/check?resource=${resource}&action=${action}`);
		} catch (error) {
			// Re-throw existing AuthError instances to preserve error codes
			if (error instanceof AuthError) {
				throw error;
			}
			throw createAuthError(AuthErrorCode.PERMISSION_CHECK_FAILED, 'Failed to check user permissions');
		}
	},

	async getUserPermissions(): Promise<ApiResponse<{ roles: string[]; permissions: string[] }>> {
		try {
			return await apiClient.get<{ roles: string[]; permissions: string[] }>('/users/permissions');
		} catch (error) {
			// Re-throw existing AuthError instances to preserve error codes
			if (error instanceof AuthError) {
				throw error;
			}
			throw createAuthError(AuthErrorCode.PERMISSION_CHECK_FAILED, 'Failed to fetch user permissions');
		}
	},

	async getUserRoles(): Promise<ApiResponse<{ roles: string[]; groups: string[] }>> {
		try {
			return await apiClient.get<{ roles: string[]; groups: string[] }>('/users/roles');
		} catch (error) {
			// Re-throw existing AuthError instances to preserve error codes
			if (error instanceof AuthError) {
				throw error;
			}
			throw createAuthError(AuthErrorCode.ROLES_FETCH_FAILED, 'Failed to fetch user roles');
		}
	},

	// Session Management
	async validateSession(): Promise<ApiResponse<{ valid: boolean; user?: UserProfile }>> {
		try {
			return await apiClient.get<{ valid: boolean; user?: UserProfile }>('/users/session/validate');
		} catch (error) {
			// Re-throw existing AuthError instances to preserve error codes
			if (error instanceof AuthError) {
				throw error;
			}
			throw createAuthError(AuthErrorCode.SESSION_VALIDATION_FAILED, 'Failed to validate session');
		}
	},

	async getSessionInfo(): Promise<ApiResponse<{ user: UserProfile; expires_at: string }>> {
		try {
			return await apiClient.get<{ user: UserProfile; expires_at: string }>('/users/session');
		} catch (error) {
			// Re-throw existing AuthError instances to preserve error codes
			if (error instanceof AuthError) {
				throw error;
			}
			throw createAuthError(AuthErrorCode.SESSION_VALIDATION_FAILED, 'Failed to get session info');
		}
	},

	async refreshSession(): Promise<ApiResponse<OAuth2Tokens>> {
		return this.refreshToken();
	},

	async endAllSessions(): Promise<ApiResponse<void>> {
		try {
			return await apiClient.delete<void>('/users/sessions');
		} catch (error) {
			// Re-throw existing AuthError instances to preserve error codes
			if (error instanceof AuthError) {
				throw error;
			}
			throw createAuthError(AuthErrorCode.SESSION_TERMINATION_FAILED, 'Failed to end all sessions');
		}
	},

	async getActiveSessions(): Promise<ApiResponse<{ sessions: Array<{ id: string; created_at: string; last_activity: string; user_agent?: string }> }>> {
		try {
			return await apiClient.get<{ sessions: Array<{ id: string; created_at: string; last_activity: string; user_agent?: string }> }>('/users/sessions');
		} catch (error) {
			// Re-throw existing AuthError instances to preserve error codes
			if (error instanceof AuthError) {
				throw error;
			}
			throw createAuthError(AuthErrorCode.SESSION_VALIDATION_FAILED, 'Failed to get active sessions');
		}
	},

	async terminateSession(sessionId: string): Promise<ApiResponse<void>> {
		try {
			return await apiClient.delete<void>(`/users/sessions/${sessionId}`);
		} catch (error) {
			// Re-throw existing AuthError instances to preserve error codes
			if (error instanceof AuthError) {
				throw error;
			}
			throw createAuthError(AuthErrorCode.SESSION_TERMINATION_FAILED, 'Failed to terminate session');
		}
	}
};
