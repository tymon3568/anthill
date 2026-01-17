import { apiClient } from './client';
import type { ApiResponse, LoginForm } from '$lib/types';

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

// OAuth2 types - DEPRECATED: OAuth2/Kanidm integration removed in PR #133
// Kept for backward compatibility with any code that may reference these types
// TODO: Remove in future cleanup once all references are updated

/** @deprecated OAuth2 integration removed - use email/password auth instead */
export interface OAuth2AuthorizeReq {
	state?: string;
}

/** @deprecated OAuth2 integration removed - use email/password auth instead */
export interface OAuth2AuthorizeResp {
	authorization_url: string;
	state: string;
	code_verifier: string;
}

/** @deprecated OAuth2 integration removed - use email/password auth instead */
export interface OAuth2CallbackReq {
	code: string;
	state: string;
	code_verifier: string;
}

/** @deprecated OAuth2 integration removed - use email/password auth instead */
export interface OAuth2CallbackResp {
	access_token: string;
	refresh_token?: string;
	token_type: string;
	expires_in?: number;
	user?: {
		kanidm_user_id?: string;
		email?: string;
		preferred_username?: string;
		groups?: string[];
	};
	tenant?: {
		tenant_id: string;
		name: string;
		slug: string;
		role: string;
	};
}

/** @deprecated OAuth2 integration removed - use email/password auth instead */
export interface OAuth2RefreshReq {
	refresh_token: string;
}

/** @deprecated OAuth2 integration removed - use email/password auth instead */
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

// Session info type for session endpoints
export interface SessionInfo {
	session_id: string;
	user_id: string;
	tenant_id: string;
	ip_address?: string;
	user_agent?: string;
	device_info?: string;
	created_at: string;
	last_used_at: string;
	expires_at: string;
}

// Register user data type - alias to EmailRegisterRequest for consistency
export type RegisterUserData = EmailRegisterRequest;

export interface UserProfile {
	id: string;
	email: string;
	username?: string;
	display_name?: string;
	role: string;
	tenant_id: string;
	created_at: string;
	updated_at: string;
	preferences?: Record<string, unknown>;
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

	// OAuth2 Flow Methods - DEPRECATED: OAuth2/Kanidm integration removed in PR #133
	// These methods will throw errors as the backend endpoints no longer exist
	// Kept for backward compatibility - will be removed in future cleanup

	/** @deprecated OAuth2 integration removed - use emailLogin instead */
	async initiateOAuth2Authorize(
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		_req: OAuth2AuthorizeReq = {}
	): Promise<ApiResponse<OAuth2AuthorizeResp>> {
		console.warn('OAuth2 integration has been removed. Use email/password authentication instead.');
		return Promise.resolve({
			success: false,
			error: 'OAuth2 integration has been removed. Use email/password authentication instead.'
		} as ApiResponse<OAuth2AuthorizeResp>);
	},

	/** @deprecated OAuth2 integration removed - use emailLogin instead */
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	async handleOAuth2Callback(_req: OAuth2CallbackReq): Promise<ApiResponse<OAuth2CallbackResp>> {
		console.warn('OAuth2 integration has been removed. Use email/password authentication instead.');
		return Promise.resolve({
			success: false,
			error: 'OAuth2 integration has been removed. Use email/password authentication instead.'
		} as ApiResponse<OAuth2CallbackResp>);
	},

	/** @deprecated OAuth2 integration removed - use refreshEmailToken instead */
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	async refreshOAuth2Token(_req: OAuth2RefreshReq): Promise<ApiResponse<OAuth2RefreshResp>> {
		console.warn('OAuth2 integration has been removed. Use refreshEmailToken instead.');
		return Promise.resolve({
			success: false,
			error: 'OAuth2 integration has been removed. Use refreshEmailToken instead.'
		} as ApiResponse<OAuth2RefreshResp>);
	},

	/** @deprecated OAuth2 integration removed - use emailLogin instead */
	async initiateOAuth2Login(): Promise<never> {
		console.warn('OAuth2 integration has been removed. Use email/password authentication instead.');
		throw new Error(
			'OAuth2 integration has been removed. Use email/password authentication instead.'
		);
	},

	/** @deprecated OAuth2 integration removed - use emailLogin instead */
	async handleOAuth2CallbackLegacy(
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		_code: string,
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		_state: string
	): Promise<ApiResponse<OAuth2Tokens>> {
		console.warn('OAuth2 integration has been removed. Use email/password authentication instead.');
		return Promise.resolve({
			success: false,
			error: 'OAuth2 integration has been removed. Use email/password authentication instead.'
		} as ApiResponse<OAuth2Tokens>);
	},

	/** @deprecated OAuth2 integration removed - use refreshEmailToken instead */
	async refreshToken(): Promise<ApiResponse<OAuth2Tokens>> {
		console.warn('OAuth2 integration has been removed. Use refreshEmailToken instead.');
		return Promise.resolve({
			success: false,
			error: 'OAuth2 integration has been removed. Use refreshEmailToken instead.'
		} as ApiResponse<OAuth2Tokens>);
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
	async getPreferences(): Promise<ApiResponse<Record<string, unknown>>> {
		return apiClient.get<Record<string, unknown>>('/auth/preferences');
	},

	async updatePreferences(
		preferences: Record<string, unknown>
	): Promise<ApiResponse<Record<string, unknown>>> {
		return apiClient.put<Record<string, unknown>>('/auth/preferences', preferences);
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

	async getSessionInfo(): Promise<ApiResponse<SessionInfo>> {
		return apiClient.get<SessionInfo>('/auth/session');
	},

	async getActiveSessions(): Promise<ApiResponse<SessionInfo[]>> {
		return apiClient.get<SessionInfo[]>('/auth/sessions');
	},

	async terminateSession(sessionId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`/auth/sessions/${sessionId}`);
	},

	async endAllSessions(): Promise<ApiResponse<void>> {
		return apiClient.delete<void>('/auth/sessions');
	},

	// Email Verification Methods
	async verifyEmail(token: string): Promise<ApiResponse<void>> {
		return apiClient.post<void>('/auth/verify-email', { token });
	},

	async resendVerification(email: string): Promise<ApiResponse<void>> {
		return apiClient.post<void>('/auth/resend-verification', { email });
	},

	// Password Reset Methods
	async forgotPassword(email: string): Promise<ApiResponse<void>> {
		return apiClient.post<void>('/auth/forgot-password', { email });
	},

	async validateResetToken(token: string): Promise<ApiResponse<void>> {
		return apiClient.post<void>('/auth/validate-reset-token', { token });
	},

	async resetPassword(token: string, newPassword: string): Promise<ApiResponse<void>> {
		return apiClient.post<void>('/auth/reset-password', { token, new_password: newPassword });
	},

	// Legacy methods (for backward compatibility)
	async login(credentials: LoginForm): Promise<ApiResponse<AuthResponse>> {
		return apiClient.post<AuthResponse>(
			'/auth/login',
			credentials as unknown as Record<string, unknown>
		);
	},

	async register(userData: RegisterUserData): Promise<ApiResponse<AuthResponse>> {
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
