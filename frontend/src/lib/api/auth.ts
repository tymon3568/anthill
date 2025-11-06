import { apiClient } from './client';
import type { ApiResponse, User, LoginForm } from '$lib/types';

export interface AuthResponse {
	user: User;
	access_token: string;
	refresh_token: string;
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
		throw new Error('Redirecting to OAuth2 provider');
	},

	async handleOAuth2Callback(code: string, state: string): Promise<ApiResponse<OAuth2Tokens>> {
		// This is handled by the server endpoint, but we can provide a client method
		// for programmatic access if needed
		return apiClient.get<OAuth2Tokens>(`/auth/oauth/callback?code=${code}&state=${state}`);
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

	async refreshTokenLegacy(
		refreshToken: string
	): Promise<ApiResponse<{ access_token: string; expires_in: number }>> {
		return apiClient.post('/auth/refresh', { refresh_token: refreshToken });
	},

	async logoutLegacy(): Promise<ApiResponse<void>> {
		return apiClient.post('/auth/logout');
	},

	// User Profile Management (matches backend API spec)
	async getProfile(): Promise<ApiResponse<UserProfile>> {
		return apiClient.get<UserProfile>('/users/profile');
	},

	async updateProfile(userData: Partial<UserProfile>): Promise<ApiResponse<UserProfile>> {
		return apiClient.put<UserProfile>('/users/profile', userData);
	},

	async uploadProfileImage(file: File): Promise<ApiResponse<{ image_url: string }>> {
		const formData = new FormData();
		formData.append('image', file);

		// Get API base URL from environment
		const API_BASE_URL = import.meta.env.PUBLIC_API_BASE_URL || 'http://localhost:3000/api/v1';

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
		return apiClient.get<UserPreferences>('/users/preferences');
	},

	async updatePreferences(preferences: Partial<UserPreferences>): Promise<ApiResponse<UserPreferences>> {
		return apiClient.put<UserPreferences>('/users/preferences', preferences);
	},

	// Permission Checking
	async checkPermission(resource: string, action: string): Promise<ApiResponse<{ allowed: boolean }>> {
		return apiClient.get<{ allowed: boolean }>(`/users/permissions/check?resource=${resource}&action=${action}`);
	},

	async getUserPermissions(): Promise<ApiResponse<{ roles: string[]; permissions: string[] }>> {
		return apiClient.get<{ roles: string[]; permissions: string[] }>('/users/permissions');
	},

	async getUserRoles(): Promise<ApiResponse<{ roles: string[]; groups: string[] }>> {
		return apiClient.get<{ roles: string[]; groups: string[] }>('/users/roles');
	},

	// Session Management
	async validateSession(): Promise<ApiResponse<{ valid: boolean; user?: UserProfile }>> {
		return apiClient.get<{ valid: boolean; user?: UserProfile }>('/auth/session/validate');
	},

	async getSessionInfo(): Promise<ApiResponse<{ user: UserProfile; expires_at: string }>> {
		return apiClient.get<{ user: UserProfile; expires_at: string }>('/auth/session');
	},

	async refreshSession(): Promise<ApiResponse<OAuth2Tokens>> {
		return this.refreshToken();
	},

	async endAllSessions(): Promise<ApiResponse<void>> {
		return apiClient.post<void>('/auth/sessions/end-all');
	},

	async getActiveSessions(): Promise<ApiResponse<{ sessions: Array<{ id: string; created_at: string; last_activity: string; user_agent?: string }> }>> {
		return apiClient.get<{ sessions: Array<{ id: string; created_at: string; last_activity: string; user_agent?: string }> }>('/auth/sessions');
	},

	async terminateSession(sessionId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`/auth/sessions/${sessionId}`);
	}
};
