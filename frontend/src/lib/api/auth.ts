import { apiClient } from './client';
import type { ApiResponse, LoginForm } from '$lib/types';
import { getCurrentTenantSlug } from '$lib/tenant';

/**
 * Auth API client that uses local SvelteKit proxy endpoints
 * This ensures cookies are set on the same domain as the frontend
 */
class AuthApiClient {
	private tenantSlug: string | null = null;

	setTenantSlug(slug: string | null): void {
		this.tenantSlug = slug;
	}

	getTenantSlug(): string | null {
		return this.tenantSlug;
	}

	private async request<T>(endpoint: string, options: RequestInit = {}): Promise<ApiResponse<T>> {
		// Use relative URL to hit SvelteKit proxy endpoints
		const url = `/api/v1${endpoint}`;

		const headers: Record<string, string> = {
			'Content-Type': 'application/json',
			...(options.headers as Record<string, string>)
		};

		// Add X-Tenant-ID header if tenant context is available
		const tenantSlug = this.tenantSlug ?? getCurrentTenantSlug();
		if (tenantSlug) {
			headers['X-Tenant-ID'] = tenantSlug;
		}

		const { headers: _optHeaders, ...restOptions } = options;
		const config: RequestInit = {
			...restOptions,
			headers,
			credentials: 'include'
		};

		try {
			const response = await fetch(url, config);

			if (!response.ok) {
				const errorData = await response.json().catch(() => ({
					message: 'Network error',
					error: 'Network error'
				}));

				return {
					success: false,
					error: errorData.error || errorData.message || `HTTP ${response.status}`
				};
			}

			if (response.status === 204 || response.headers.get('content-length') === '0') {
				return { success: true };
			}

			const contentType = response.headers.get('content-type') ?? '';
			let data: T;
			if (contentType.includes('application/json')) {
				data = await response.json();
			} else {
				data = (await response.text()) as unknown as T;
			}

			return {
				success: true,
				data
			};
		} catch (error) {
			return {
				success: false,
				error: error instanceof Error ? error.message : 'Unknown error'
			};
		}
	}

	async post<T>(endpoint: string, data?: Record<string, unknown>): Promise<ApiResponse<T>> {
		return this.request<T>(endpoint, {
			method: 'POST',
			body: data ? JSON.stringify(data) : undefined
		});
	}
}

// Singleton instance for auth API calls via proxy
const authApiClient = new AuthApiClient();

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

// OAuth2 types - DEPRECATED: OAuth2 integration removed in PR #133
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
		user_id?: string; // Legacy field
		email?: string;
		preferred_username?: string;
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
	// Email/Password Authentication Methods - use proxy for cookie handling
	async emailLogin(credentials: EmailLoginRequest): Promise<ApiResponse<EmailAuthResponse>> {
		// Use authApiClient which routes through SvelteKit proxy
		return authApiClient.post<EmailAuthResponse>(
			'/auth/login',
			credentials as unknown as Record<string, unknown>
		);
	},

	async emailRegister(userData: EmailRegisterRequest): Promise<ApiResponse<EmailAuthResponse>> {
		// Use authApiClient which routes through SvelteKit proxy
		return authApiClient.post<EmailAuthResponse>(
			'/auth/register',
			userData as unknown as Record<string, unknown>
		);
	},

	async refreshEmailToken(_refreshToken?: string): Promise<ApiResponse<EmailAuthResponse>> {
		// Use authApiClient - refresh token is read from httpOnly cookie by proxy
		return authApiClient.post<EmailAuthResponse>('/auth/refresh', {});
	},

	async emailLogout(_refreshToken?: string): Promise<ApiResponse<void>> {
		// Use authApiClient - logout clears cookies via proxy
		return authApiClient.post<void>('/auth/logout', {});
	},

	/**
	 * Set tenant slug for auth API client
	 * This is used to set X-Tenant-ID header for login requests
	 */
	setTenantSlug(slug: string | null): void {
		authApiClient.setTenantSlug(slug);
	},

	// Tenant Discovery Methods

	/**
	 * Check if a tenant slug is available
	 * @param slug - The tenant slug to check (will be normalized by backend)
	 * @returns Availability status and existing tenant name if not available
	 */
	async checkTenantSlug(
		slug: string
	): Promise<
		ApiResponse<{ slug: string; available: boolean; existing_tenant_name: string | null }>
	> {
		return apiClient.get<{
			slug: string;
			available: boolean;
			existing_tenant_name: string | null;
		}>(`/auth/check-tenant-slug?slug=${encodeURIComponent(slug)}`);
	},

	// OAuth2 Flow Methods - DEPRECATED: OAuth2 integration removed in PR #133
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
		// Use authApiClient to clear cookies via proxy
		return authApiClient.post<void>(`/auth/logout${params}`);
	},

	// User Profile Methods
	async getProfile(): Promise<ApiResponse<UserProfile>> {
		return apiClient.get<UserProfile>('/users/profile');
	},

	async updateProfile(profileData: Partial<UserProfile>): Promise<ApiResponse<UserProfile>> {
		return apiClient.put<UserProfile>(
			'/users/profile',
			profileData as unknown as Record<string, unknown>
		);
	},

	// User Preferences Methods
	async getPreferences(): Promise<ApiResponse<Record<string, unknown>>> {
		return apiClient.get<Record<string, unknown>>('/users/preferences');
	},

	async updatePreferences(
		preferences: Record<string, unknown>
	): Promise<ApiResponse<Record<string, unknown>>> {
		return apiClient.put<Record<string, unknown>>('/users/preferences', preferences);
	},

	// Permission Methods
	async checkPermission(resource: string, action: string): Promise<ApiResponse<boolean>> {
		return apiClient.get<boolean>(
			`/users/permissions/check?resource=${encodeURIComponent(resource)}&action=${encodeURIComponent(action)}`
		);
	},

	async getUserPermissions(): Promise<ApiResponse<string[]>> {
		return apiClient.get<string[]>('/users/permissions');
	},

	// Role Methods
	async getUserRoles(): Promise<ApiResponse<string[]>> {
		return apiClient.get<string[]>('/users/roles');
	},

	// Session Methods
	async validateSession(): Promise<ApiResponse<boolean>> {
		return apiClient.get<boolean>('/users/session/validate');
	},

	async getSessionInfo(): Promise<ApiResponse<SessionInfo>> {
		return apiClient.get<SessionInfo>('/users/session');
	},

	async getActiveSessions(): Promise<ApiResponse<SessionInfo[]>> {
		return apiClient.get<SessionInfo[]>('/users/sessions');
	},

	async terminateSession(sessionId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`/users/sessions/${sessionId}`);
	},

	async endAllSessions(): Promise<ApiResponse<void>> {
		return apiClient.delete<void>('/users/sessions');
	},

	// Email Verification Methods
	async verifyEmail(token: string): Promise<ApiResponse<void>> {
		return apiClient.post<void>('/auth/verify-email', { token });
	},

	async resendVerification(email: string, tenantId?: string): Promise<ApiResponse<void>> {
		const headers: Record<string, string> = {};
		if (tenantId) {
			headers['X-Tenant-ID'] = tenantId;
		}
		return apiClient.post<void>('/auth/resend-verification', { email }, { headers });
	},

	// Password Reset Methods
	async forgotPassword(email: string): Promise<ApiResponse<void>> {
		return apiClient.post<void>('/auth/forgot-password', { email });
	},

	async validateResetToken(token: string): Promise<ApiResponse<void>> {
		return apiClient.post<void>('/auth/validate-reset-token', { token });
	},

	async resetPassword(
		token: string,
		newPassword: string,
		confirmPassword: string
	): Promise<ApiResponse<void>> {
		return apiClient.post<void>('/auth/reset-password', {
			token,
			new_password: newPassword,
			confirm_password: confirmPassword
		});
	},

	// Legacy methods (for backward compatibility) - use proxy for cookie handling
	async login(credentials: LoginForm): Promise<ApiResponse<AuthResponse>> {
		return authApiClient.post<AuthResponse>(
			'/auth/login',
			credentials as unknown as Record<string, unknown>
		);
	},

	async register(userData: RegisterUserData): Promise<ApiResponse<AuthResponse>> {
		return authApiClient.post<AuthResponse>(
			'/auth/register',
			userData as unknown as Record<string, unknown>
		);
	},

	async refreshTokenLegacy(_refreshToken?: string): Promise<ApiResponse<AuthResponse>> {
		// Use proxy - refresh token is read from httpOnly cookie
		return authApiClient.post<AuthResponse>('/auth/refresh', {});
	},

	async logoutLegacy(refreshToken?: { refresh_token: string }): Promise<ApiResponse<void>> {
		const data = refreshToken ? refreshToken : {};
		// Use authApiClient to clear cookies via proxy
		return authApiClient.post('/auth/logout', data);
	}
};
