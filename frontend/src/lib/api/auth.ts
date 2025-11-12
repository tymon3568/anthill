import { apiClient } from './client';
import type { ApiResponse, User, LoginForm } from '$lib/types';

// Email/Password authentication DTOs
export interface EmailLoginRequest {
	email: string;
	password: string;
}

export interface EmailRegisterRequest {
	email: string;
	password: string;
	full_name: string;
	tenant_name?: string;
}

export interface EmailAuthResponse {
	access_token: string;
	refresh_token: string;
	token_type: string;
	expires_in: number;
	user: EmailUserInfo;
}

export interface EmailUserInfo {
	id: string;
	email: string;
	full_name?: string;
	tenant_id: string;
	role: string;
	created_at: string;
}

export interface RefreshTokenRequest {
	refresh_token: string;
}

// OAuth2 DTOs matching backend OpenAPI specification
export interface OAuth2AuthorizeReq {
	state?: string;
}

export interface OAuth2AuthorizeResp {
	authorization_url: string;
	state: string;
	code_verifier: string;
}

export interface OAuth2CallbackReq {
	code: string;
	state: string;
	code_verifier: string;
}

export interface OAuth2CallbackResp {
	access_token: string;
	refresh_token?: string;
	token_type: string;
	expires_in?: number;
	user: KanidmUserInfo;
	tenant?: TenantInfo;
}

export interface KanidmUserInfo {
	kanidm_user_id: string;
	email?: string;
	preferred_username?: string;
	groups: string[];
}

export interface TenantInfo {
	tenant_id: string;
	name: string;
	slug: string;
	role: string;
}

export interface OAuth2RefreshReq {
	refresh_token: string;
}

export interface OAuth2RefreshResp {
	access_token: string;
	refresh_token?: string;
	token_type: string;
	expires_in?: number;
}

// Legacy interfaces for backward compatibility
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
	username?: string;
	display_name?: string;
	role: string;
	tenant_id: string;
	created_at: string;
	updated_at: string;
	preferences?: Record<string, any>;
	// Additional profile fields for preferences
	language?: string;
	timezone?: string;
	date_format?: string;
	time_format?: string;
	notification_preferences?: {
		email_notifications: boolean;
		push_notifications: boolean;
		sms_notifications: boolean;
	};
	profile_visibility?: string;
	show_email?: boolean;
	show_phone?: boolean;
}

export const authApi = {
	// Email/Password Authentication Methods
	async emailLogin(credentials: EmailLoginRequest): Promise<ApiResponse<EmailAuthResponse>> {
		return apiClient.post<EmailAuthResponse>(
			'/auth/login',
			credentials as unknown as Record<string, unknown>
		);
	},

	async emailRegister(userData: EmailRegisterRequest): Promise<ApiResponse<EmailAuthResponse>> {
		return apiClient.post<EmailAuthResponse>(
			'/auth/register',
			userData as unknown as Record<string, unknown>
		);
	},

	async refreshEmailToken(refreshToken: string): Promise<ApiResponse<EmailAuthResponse>> {
		return apiClient.post<EmailAuthResponse>('/auth/refresh', { refresh_token: refreshToken });
	},

	async emailLogout(refreshToken: string): Promise<ApiResponse<void>> {
		return apiClient.post<void>('/auth/logout', { refresh_token: refreshToken });
	},

	// OAuth2 Flow Methods - matching OpenAPI specification
	async initiateOAuth2Authorize(
		req: OAuth2AuthorizeReq = {}
	): Promise<ApiResponse<OAuth2AuthorizeResp>> {
		return apiClient.post<OAuth2AuthorizeResp>(
			'/auth/oauth/authorize',
			req as unknown as Record<string, unknown>
		);
	},

	async handleOAuth2Callback(req: OAuth2CallbackReq): Promise<ApiResponse<OAuth2CallbackResp>> {
		return apiClient.post<OAuth2CallbackResp>(
			'/auth/oauth/callback',
			req as unknown as Record<string, unknown>
		);
	},

	async refreshOAuth2Token(req: OAuth2RefreshReq): Promise<ApiResponse<OAuth2RefreshResp>> {
		return apiClient.post<OAuth2RefreshResp>(
			'/auth/oauth/refresh',
			req as unknown as Record<string, unknown>
		);
	},

	// Legacy OAuth2 methods for backward compatibility
	async initiateOAuth2Login(): Promise<never> {
		// This will redirect to Kanidm OAuth2 authorize endpoint
		// The actual implementation is in the server endpoint
		window.location.href = '/api/v1/auth/oauth/authorize';
		// Return a promise that never resolves since navigation is in progress
		return new Promise<never>(() => {
			// Intentionally left unresolved; navigation is in progress.
		});
	},

	async handleOAuth2CallbackLegacy(
		code: string,
		state: string
	): Promise<ApiResponse<OAuth2Tokens>> {
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

	// User Profile Methods
	async getProfile(): Promise<ApiResponse<UserProfile>> {
		return apiClient.get<UserProfile>('/auth/profile');
	},

	async updateProfile(profileData: Partial<UserProfile>): Promise<ApiResponse<UserProfile>> {
		return apiClient.put<UserProfile>(
			'/auth/profile',
			profileData as unknown as Record<string, unknown>
		);
	},

	// User Preferences Methods
	async getPreferences(): Promise<ApiResponse<Record<string, any>>> {
		return apiClient.get<Record<string, any>>('/auth/preferences');
	},

	async updatePreferences(
		preferences: Record<string, any>
	): Promise<ApiResponse<Record<string, any>>> {
		return apiClient.put<Record<string, any>>('/auth/preferences', preferences);
	},

	// Permission Methods
	async checkPermission(resource: string, action: string): Promise<ApiResponse<boolean>> {
		return apiClient.get<boolean>(
			`/auth/permissions/check?resource=${encodeURIComponent(resource)}&action=${encodeURIComponent(action)}`
		);
	},

	async getUserPermissions(): Promise<ApiResponse<string[]>> {
		return apiClient.get<string[]>('/auth/permissions');
	},

	// Role Methods
	async getUserRoles(): Promise<ApiResponse<string[]>> {
		return apiClient.get<string[]>('/auth/roles');
	},

	// Session Methods
	async validateSession(): Promise<ApiResponse<boolean>> {
		return apiClient.get<boolean>('/auth/session/validate');
	},

	async getSessionInfo(): Promise<ApiResponse<any>> {
		return apiClient.get<any>('/auth/session');
	},

	async getActiveSessions(): Promise<ApiResponse<any[]>> {
		return apiClient.get<any[]>('/auth/sessions');
	},

	async terminateSession(sessionId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`/auth/sessions/${sessionId}`);
	},

	async endAllSessions(): Promise<ApiResponse<void>> {
		return apiClient.delete<void>('/auth/sessions');
	},

	// Legacy methods (for backward compatibility)
	async login(credentials: LoginForm): Promise<ApiResponse<AuthResponse>> {
		return apiClient.post<AuthResponse>(
			'/auth/login',
			credentials as unknown as Record<string, unknown>
		);
	},

	async register(userData: {
		full_name: string;
		email: string;
		password: string;
		tenant_name?: string;
	}): Promise<ApiResponse<AuthResponse>> {
		return apiClient.post<AuthResponse>(
			'/auth/register',
			userData as unknown as Record<string, unknown>
		);
	},

	async refreshTokenLegacy(refreshToken: string): Promise<ApiResponse<AuthResponse>> {
		return apiClient.post<AuthResponse>('/auth/refresh', { refresh_token: refreshToken });
	},

	async logoutLegacy(refreshToken?: { refresh_token: string }): Promise<ApiResponse<void>> {
		const data = refreshToken ? refreshToken : {};
		return apiClient.post('/auth/logout', data);
	}
};
