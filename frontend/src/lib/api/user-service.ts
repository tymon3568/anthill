/**
 * User Service API Client
 *
 * Comprehensive API client for User Service interactions beyond basic authentication.
 * Provides type-safe methods for:
 * - User Profile Management
 * - Admin User Management
 * - Admin Role Management
 * - Admin Invitation Management
 * - Permission Checking
 *
 * @module user-service
 */

import { apiClient, createPaginationParams } from './client';
import type { ApiResponse } from '$lib/types';
import type {
	User,
	UserProfile,
	PublicProfile,
	Role,
	RoleListResponse,
	Permission,
	PermissionListResponse,
	Invitation,
	InvitationStatus,
	PaginatedUsers,
	PaginatedInvitations,
	ListUsersParams,
	ListInvitationsParams,
	CreateUserRequest,
	UpdateProfileRequest,
	VisibilitySettings,
	CompletenessScore,
	CreateInvitationRequest,
	CreateRoleRequest,
	UpdateRoleRequest,
	ProfileSearchRequest,
	ProfileSearchResult,
	AvatarUploadResponse,
	PermissionCheckResponse,
	UserRolesResponse,
	TenantValidation,
	TenantSettings,
	TenantBilling,
	TenantAnalytics,
	TenantBranding,
	TenantLocalization,
	TenantSecurityPolicy,
	TenantDataRetention,
	TenantIntegrationSettings,
	PaginatedAuditLogs,
	UpdateTenantRequest,
	UpdateBrandingRequest,
	UpdateLocalizationRequest,
	UpdateSecurityPolicyRequest,
	UpdateDataRetentionRequest,
	ListAuditLogsParams,
	TenantExportRequest,
	DeleteTenantRequest,
	// Payment Gateway Types
	PaymentSettings,
	PaymentGateway,
	PaymentGatewayCredentials,
	PaymentWebhookConfig,
	PaymentMethodSettings,
	CurrencyConfig,
	PaymentRegionConfig,
	TransactionFeeConfig,
	SettlementConfig,
	PaymentGatewayHealth,
	PaymentAnalytics,
	PaymentSecuritySettings,
	UpsertPaymentGatewayRequest,
	UpdatePaymentMethodsRequest,
	UpdateCurrenciesRequest,
	UpdateRegionsRequest,
	UpdatePaymentSecurityRequest,
	TestPaymentResult
} from './types/user-service.types';
import { getCurrentTenantSlug } from '$lib/tenant';

// Use relative URL to route through SvelteKit proxy for proper cookie handling
const API_BASE_URL = '/api/v1';

/**
 * Custom API error with status code and error code
 */
export class UserServiceApiError extends Error {
	constructor(
		public status: number,
		public code: string,
		message: string
	) {
		super(message);
		this.name = 'UserServiceApiError';
	}
}

/**
 * User Service API client
 *
 * Provides methods for interacting with User Service endpoints.
 * All methods return ApiResponse wrapper for consistent error handling.
 */
export const userServiceApi = {
	// ============ Profile API ============

	/**
	 * Get current user's profile
	 * @returns User profile data
	 */
	async getProfile(): Promise<ApiResponse<UserProfile>> {
		return apiClient.get<UserProfile>('/users/profile');
	},

	/**
	 * Update current user's profile
	 * @param data - Profile fields to update
	 * @returns Updated user profile
	 */
	async updateProfile(data: UpdateProfileRequest): Promise<ApiResponse<UserProfile>> {
		return apiClient.put<UserProfile>('/users/profile', data as Record<string, unknown>);
	},

	/**
	 * Upload avatar image (max 5MB)
	 * @param file - Image file to upload
	 * @returns Avatar URL
	 */
	async uploadAvatar(file: File): Promise<ApiResponse<AvatarUploadResponse>> {
		const formData = new FormData();
		formData.append('avatar', file);

		const tenantSlug = getCurrentTenantSlug();
		const headers: HeadersInit = {};
		// NOTE: Do NOT set Content-Type for FormData - browser sets it with boundary
		if (tenantSlug) {
			headers['X-Tenant-ID'] = tenantSlug;
		}

		try {
			const response = await fetch(`${API_BASE_URL}/users/profile/avatar`, {
				method: 'POST',
				headers,
				body: formData,
				credentials: 'include' // Use httpOnly cookies for auth
			});

			if (!response.ok) {
				const errorData = await response.json().catch(() => ({
					message: 'Upload failed',
					error: 'Upload failed'
				}));
				return {
					success: false,
					error: errorData.error || errorData.message || `HTTP ${response.status}`
				};
			}

			const data = await response.json();
			return { success: true, data };
		} catch (error) {
			return {
				success: false,
				error: error instanceof Error ? error.message : 'Unknown error'
			};
		}
	},

	/**
	 * Update profile visibility settings
	 * @param settings - Visibility settings
	 */
	async updateVisibility(settings: VisibilitySettings): Promise<ApiResponse<void>> {
		return apiClient.put<void>(
			'/users/profile/visibility',
			settings as unknown as Record<string, unknown>
		);
	},

	/**
	 * Get profile completeness score with recommendations
	 * @returns Completeness score and suggestions
	 */
	async getProfileCompleteness(): Promise<ApiResponse<CompletenessScore>> {
		return apiClient.get<CompletenessScore>('/users/profile/completeness');
	},

	/**
	 * Search for user profiles
	 * @param params - Search parameters
	 * @returns Matching profiles
	 */
	async searchProfiles(params: ProfileSearchRequest): Promise<ApiResponse<ProfileSearchResult>> {
		const searchParams = new URLSearchParams();
		searchParams.set('query', params.query);
		if (params.department) searchParams.set('department', params.department);
		if (params.page) searchParams.set('page', params.page.toString());
		if (params.perPage) searchParams.set('per_page', params.perPage.toString());

		return apiClient.get<ProfileSearchResult>(`/users/profiles/search?${searchParams}`);
	},

	/**
	 * Get public profile of another user
	 * @param userId - Target user ID
	 * @returns Public profile data
	 */
	async getPublicProfile(userId: string): Promise<ApiResponse<PublicProfile>> {
		return apiClient.get<PublicProfile>(`/users/profiles/${userId}`);
	},

	// ============ Admin User API ============

	/**
	 * List users in tenant (admin only)
	 * @param params - Filtering and pagination parameters
	 * @returns Paginated list of users
	 */
	async listUsers(params: ListUsersParams = {}): Promise<ApiResponse<PaginatedUsers>> {
		const searchParams = createPaginationParams(params.page, params.perPage);
		if (params.role) searchParams.set('role', params.role);
		if (params.status) searchParams.set('status', params.status);
		if (params.search) searchParams.set('search', params.search);

		// Backend returns snake_case, frontend expects camelCase
		interface BackendUser {
			id: string;
			email: string;
			full_name: string;
			tenant_id: string;
			role: string;
			roles?: string[];
			status?: string;
			email_verified?: boolean;
			created_at: string;
			last_login_at?: string;
		}
		interface BackendResponse {
			users: BackendUser[];
			total: number;
			page: number;
			page_size: number;
		}

		const result = await apiClient.get<BackendResponse>(`/admin/users?${searchParams}`);

		if (result.success && result.data) {
			// Transform snake_case to camelCase
			const transformedUsers: User[] = result.data.users.map((user) => ({
				id: user.id,
				email: user.email,
				fullName: user.full_name,
				role: user.role,
				roles: user.roles || [],
				status: (user.status as User['status']) || 'active',
				emailVerified: user.email_verified ?? false,
				createdAt: user.created_at,
				lastLoginAt: user.last_login_at
			}));

			const perPage = params.perPage || 10;
			return {
				success: true,
				data: {
					data: transformedUsers,
					total: result.data.total,
					page: result.data.page,
					perPage: perPage,
					totalPages: Math.ceil(result.data.total / perPage)
				}
			};
		}

		return { success: false, error: result.error };
	},

	/**
	 * Get a specific user by ID (admin only)
	 * @param userId - User ID
	 * @returns User data
	 */
	async getUser(userId: string): Promise<ApiResponse<User>> {
		return apiClient.get<User>(`/admin/users/${userId}`);
	},

	/**
	 * Create a new user in tenant (admin only)
	 * @param data - User creation data
	 * @returns Created user
	 */
	async createUser(data: CreateUserRequest): Promise<ApiResponse<User>> {
		// Transform camelCase to snake_case for backend
		const backendData = {
			email: data.email,
			password: data.password,
			full_name: data.fullName,
			role: data.role
		};
		return apiClient.post<User>('/admin/users', backendData as unknown as Record<string, unknown>);
	},

	/**
	 * Suspend a user (admin only)
	 * @param userId - User ID to suspend
	 * @param reason - Optional reason for suspension
	 */
	async suspendUser(userId: string, reason?: string): Promise<ApiResponse<void>> {
		return apiClient.post<void>(`/admin/users/${userId}/suspend`, { reason: reason || null });
	},

	/**
	 * Unsuspend a user (admin only)
	 * @param userId - User ID to unsuspend
	 */
	async unsuspendUser(userId: string): Promise<ApiResponse<void>> {
		return apiClient.post<void>(`/admin/users/${userId}/unsuspend`);
	},

	/**
	 * Delete a user - soft delete (admin only)
	 * @param userId - User ID to delete
	 */
	async deleteUser(userId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`/admin/users/${userId}`);
	},

	/**
	 * Reset user's password (admin only)
	 * @param userId - User ID
	 * @param newPassword - New password to set
	 */
	async resetUserPassword(userId: string, newPassword: string): Promise<ApiResponse<void>> {
		return apiClient.post<void>(`/admin/users/${userId}/reset-password`, {
			new_password: newPassword
		});
	},

	// ============ Admin Role API ============

	/**
	 * List all roles in tenant (admin only)
	 * @returns List of roles
	 */
	async listRoles(): Promise<ApiResponse<Role[]>> {
		const result = await apiClient.get<RoleListResponse>('/admin/roles');
		if (result.success && result.data) {
			return { success: true, data: result.data.roles };
		}
		return { success: false, error: result.error };
	},

	/**
	 * Get a specific role by name (admin only)
	 * @param roleName - Role name
	 * @returns Role data
	 */
	async getRole(roleName: string): Promise<ApiResponse<Role>> {
		return apiClient.get<Role>(`/admin/roles/${roleName}`);
	},

	/**
	 * Create a new role (admin only)
	 * @param data - Role creation data
	 * @returns Created role
	 */
	async createRole(data: CreateRoleRequest): Promise<ApiResponse<Role>> {
		return apiClient.post<Role>('/admin/roles', data as unknown as Record<string, unknown>);
	},

	/**
	 * Update role permissions (admin only)
	 * @param roleName - Role name to update
	 * @param data - Updated role data
	 * @returns Updated role
	 */
	async updateRole(roleName: string, data: UpdateRoleRequest): Promise<ApiResponse<Role>> {
		return apiClient.put<Role>(
			`/admin/roles/${roleName}`,
			data as unknown as Record<string, unknown>
		);
	},

	/**
	 * Delete a role (admin only, cannot delete system roles)
	 * @param roleName - Role name to delete
	 */
	async deleteRole(roleName: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`/admin/roles/${roleName}`);
	},

	/**
	 * Get roles assigned to a user (admin only)
	 * @param userId - User ID
	 * @returns List of role names
	 */
	async getUserRoles(userId: string): Promise<ApiResponse<UserRolesResponse>> {
		return apiClient.get<UserRolesResponse>(`/admin/users/${userId}/roles`);
	},

	/**
	 * Assign a role to a user (admin only)
	 * @param userId - User ID
	 * @param roleName - Role name to assign
	 */
	async assignRole(userId: string, roleName: string): Promise<ApiResponse<void>> {
		return apiClient.post<void>(`/admin/users/${userId}/roles/assign`, { role_name: roleName });
	},

	/**
	 * Remove a role from a user (admin only)
	 * @param userId - User ID
	 * @param roleName - Role name to remove
	 */
	async removeRole(userId: string, roleName: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`/admin/users/${userId}/roles/${roleName}/remove`);
	},

	/**
	 * List all available permissions (admin only)
	 * @returns List of permissions
	 */
	async listPermissions(): Promise<ApiResponse<Permission[]>> {
		const result = await apiClient.get<PermissionListResponse>('/admin/permissions');
		if (result.success && result.data) {
			// Flatten available permissions into Permission[] format
			const permissions: Permission[] = [];
			for (const perm of result.data.permissions) {
				for (const action of perm.actions) {
					permissions.push({ resource: perm.resource, action });
				}
			}
			return { success: true, data: permissions };
		}
		return { success: false, error: result.error };
	},

	// ============ Admin Invitation API ============

	/**
	 * Create a new invitation (admin only)
	 * @param data - Invitation data
	 * @returns Created invitation
	 */
	async createInvitation(data: CreateInvitationRequest): Promise<ApiResponse<Invitation>> {
		return apiClient.post<Invitation>(
			'/admin/users/invite',
			data as unknown as Record<string, unknown>
		);
	},

	/**
	 * List invitations in tenant (admin only)
	 * @param params - Filtering and pagination parameters
	 * @returns Paginated list of invitations
	 */
	async listInvitations(
		params: ListInvitationsParams = {}
	): Promise<ApiResponse<PaginatedInvitations>> {
		const searchParams = createPaginationParams(params.page, params.perPage);
		if (params.status) searchParams.set('status', params.status);

		// Backend returns snake_case with nested objects
		interface BackendInvitedBy {
			user_id: string;
			email: string;
			full_name?: string;
		}
		interface BackendInvitation {
			invitation_id: string;
			email: string;
			role: string;
			status: InvitationStatus;
			invited_by: BackendInvitedBy;
			expires_at: string;
			created_at: string;
			accepted_at?: string;
		}
		interface BackendResponse {
			invitations: BackendInvitation[];
			total: number;
			page: number;
			page_size: number;
		}

		const result = await apiClient.get<BackendResponse>(`/admin/users/invitations?${searchParams}`);

		if (result.success && result.data) {
			// Transform snake_case to camelCase
			const transformedInvitations: Invitation[] = result.data.invitations.map((inv) => ({
				id: inv.invitation_id,
				email: inv.email,
				role: inv.role,
				status: inv.status,
				invitedBy: inv.invited_by.user_id,
				invitedByName: inv.invited_by.full_name || inv.invited_by.email,
				expiresAt: inv.expires_at,
				createdAt: inv.created_at,
				acceptedAt: inv.accepted_at
			}));

			const perPage = params.perPage || 10;
			return {
				success: true,
				data: {
					data: transformedInvitations,
					total: result.data.total,
					page: result.data.page,
					perPage: perPage,
					totalPages: Math.ceil(result.data.total / perPage)
				}
			};
		}

		return { success: false, error: result.error };
	},

	/**
	 * Get a specific invitation by ID (admin only)
	 * @param invitationId - Invitation ID
	 * @returns Invitation data
	 */
	async getInvitation(invitationId: string): Promise<ApiResponse<Invitation>> {
		return apiClient.get<Invitation>(`/admin/users/invitations/${invitationId}`);
	},

	/**
	 * Revoke an invitation (admin only)
	 * @param invitationId - Invitation ID to revoke
	 */
	async revokeInvitation(invitationId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`/admin/users/invitations/${invitationId}`);
	},

	/**
	 * Resend an invitation email (admin only)
	 * @param invitationId - Invitation ID to resend
	 */
	async resendInvitation(invitationId: string): Promise<ApiResponse<void>> {
		return apiClient.post<void>(`/admin/users/invitations/${invitationId}/resend`);
	},

	// ============ Permission Checking API ============

	/**
	 * Check if current user has a specific permission
	 * @param resource - Resource name
	 * @param action - Action name
	 * @returns Whether permission is granted
	 */
	async checkPermission(resource: string, action: string): Promise<ApiResponse<boolean>> {
		const result = await apiClient.get<PermissionCheckResponse>(
			`/users/permissions/check?resource=${encodeURIComponent(resource)}&action=${encodeURIComponent(action)}`
		);
		if (result.success && result.data) {
			return { success: true, data: result.data.allowed };
		}
		return { success: false, error: result.error };
	},

	/**
	 * Get all permissions for current user
	 * @returns List of permissions
	 */
	async getUserPermissions(): Promise<ApiResponse<Permission[]>> {
		return apiClient.get<Permission[]>('/users/permissions');
	},

	/**
	 * Get current user's roles
	 * @returns List of role names
	 */
	async getCurrentUserRoles(): Promise<ApiResponse<string[]>> {
		return apiClient.get<string[]>('/users/roles');
	},

	/**
	 * Validate current tenant access
	 * @returns Tenant validation result
	 */
	async validateTenantAccess(): Promise<ApiResponse<TenantValidation>> {
		return apiClient.get<TenantValidation>('/users/tenant/validate');
	},

	// ============ Tenant Settings API (Owner Only) ============

	/**
	 * Get tenant settings (owner only)
	 * @returns Full tenant settings
	 */
	async getTenantSettings(): Promise<ApiResponse<TenantSettings>> {
		return apiClient.get<TenantSettings>('/tenant/settings');
	},

	/**
	 * Update tenant basic info (owner only)
	 * @param data - Tenant info to update
	 * @returns Updated tenant settings
	 */
	async updateTenant(data: UpdateTenantRequest): Promise<ApiResponse<TenantSettings>> {
		return apiClient.put<TenantSettings>('/tenant', data as Record<string, unknown>);
	},

	/**
	 * Update tenant branding (owner only)
	 * @param data - Branding settings to update
	 */
	async updateBranding(data: UpdateBrandingRequest): Promise<ApiResponse<TenantBranding>> {
		return apiClient.put<TenantBranding>('/tenant/branding', data as Record<string, unknown>);
	},

	/**
	 * Upload tenant logo (owner only)
	 * @param file - Logo image file
	 * @returns Logo URL
	 */
	async uploadLogo(file: File): Promise<ApiResponse<{ logoUrl: string }>> {
		const formData = new FormData();
		formData.append('logo', file);

		const tenantSlug = getCurrentTenantSlug();
		const headers: HeadersInit = {};
		// NOTE: Do NOT set Content-Type for FormData - browser sets it with boundary
		if (tenantSlug) {
			headers['X-Tenant-ID'] = tenantSlug;
		}

		try {
			const response = await fetch(`${API_BASE_URL}/tenant/branding/logo`, {
				method: 'POST',
				headers,
				body: formData,
				credentials: 'include' // Use httpOnly cookies for auth
			});

			if (!response.ok) {
				const errorData = await response.json().catch(() => ({
					message: 'Upload failed',
					error: 'Upload failed'
				}));
				return {
					success: false,
					error: errorData.error || errorData.message || `HTTP ${response.status}`
				};
			}

			const data = await response.json();
			return { success: true, data };
		} catch (error) {
			return {
				success: false,
				error: error instanceof Error ? error.message : 'Unknown error'
			};
		}
	},

	/**
	 * Update tenant localization settings (owner only)
	 * @param data - Localization settings
	 */
	async updateLocalization(
		data: UpdateLocalizationRequest
	): Promise<ApiResponse<TenantLocalization>> {
		return apiClient.put<TenantLocalization>(
			'/tenant/localization',
			data as Record<string, unknown>
		);
	},

	/**
	 * Update tenant security policy (owner only)
	 * @param data - Security policy settings
	 */
	async updateSecurityPolicy(
		data: UpdateSecurityPolicyRequest
	): Promise<ApiResponse<TenantSecurityPolicy>> {
		return apiClient.put<TenantSecurityPolicy>(
			'/tenant/security-policy',
			data as Record<string, unknown>
		);
	},

	/**
	 * Update tenant data retention settings (owner only)
	 * @param data - Data retention settings
	 */
	async updateDataRetention(
		data: UpdateDataRetentionRequest
	): Promise<ApiResponse<TenantDataRetention>> {
		return apiClient.put<TenantDataRetention>(
			'/tenant/data-retention',
			data as Record<string, unknown>
		);
	},

	/**
	 * Get tenant billing information (owner only)
	 * @returns Billing information
	 */
	async getTenantBilling(): Promise<ApiResponse<TenantBilling>> {
		return apiClient.get<TenantBilling>('/tenant/billing');
	},

	/**
	 * Get tenant integration settings (owner only)
	 * @returns Integration settings (webhooks, API keys)
	 */
	async getTenantIntegrations(): Promise<ApiResponse<TenantIntegrationSettings>> {
		return apiClient.get<TenantIntegrationSettings>('/tenant/integrations');
	},

	/**
	 * List audit logs (owner only)
	 * @param params - Filter parameters
	 * @returns Paginated audit logs
	 */
	async listAuditLogs(params: ListAuditLogsParams = {}): Promise<ApiResponse<PaginatedAuditLogs>> {
		const searchParams = createPaginationParams(params.page, params.perPage);
		if (params.userId) searchParams.set('user_id', params.userId);
		if (params.action) searchParams.set('action', params.action);
		if (params.startDate) searchParams.set('start_date', params.startDate);
		if (params.endDate) searchParams.set('end_date', params.endDate);

		return apiClient.get<PaginatedAuditLogs>(`/tenant/audit-logs?${searchParams}`);
	},

	/**
	 * Get tenant analytics (owner only)
	 * @returns Tenant usage analytics
	 */
	async getTenantAnalytics(): Promise<ApiResponse<TenantAnalytics>> {
		return apiClient.get<TenantAnalytics>('/tenant/analytics');
	},

	/**
	 * Export tenant data (owner only)
	 * @param request - Export configuration
	 * @returns Download URL or blob
	 */
	async exportTenantData(
		request: TenantExportRequest
	): Promise<ApiResponse<{ downloadUrl: string }>> {
		return apiClient.post<{ downloadUrl: string }>(
			'/tenant/export',
			request as unknown as Record<string, unknown>
		);
	},

	/**
	 * Delete tenant (owner only) - DANGER ZONE
	 * @param request - Deletion confirmation with tenant name
	 */
	async deleteTenant(request: DeleteTenantRequest): Promise<ApiResponse<void>> {
		return apiClient.post<void>('/tenant/delete', request as unknown as Record<string, unknown>);
	},

	// =====================================
	// Payment Gateway Settings API (Owner Only)
	// =====================================

	/**
	 * Get all payment settings (owner only)
	 * @returns Full payment configuration
	 */
	async getPaymentSettings(): Promise<ApiResponse<PaymentSettings>> {
		return apiClient.get<PaymentSettings>('/tenant/payments');
	},

	/**
	 * Get payment gateway credentials (owner only)
	 * @param gatewayId - Gateway ID
	 * @returns Masked credentials
	 */
	async getGatewayCredentials(gatewayId: string): Promise<ApiResponse<PaymentGatewayCredentials>> {
		return apiClient.get<PaymentGatewayCredentials>(
			`/tenant/payments/gateways/${gatewayId}/credentials`
		);
	},

	/**
	 * Add or update a payment gateway (owner only)
	 * @param gatewayId - Gateway ID (undefined for new)
	 * @param data - Gateway configuration
	 */
	async upsertPaymentGateway(
		gatewayId: string | undefined,
		data: UpsertPaymentGatewayRequest
	): Promise<ApiResponse<PaymentGateway>> {
		if (gatewayId) {
			return apiClient.put<PaymentGateway>(
				`/tenant/payments/gateways/${gatewayId}`,
				data as unknown as Record<string, unknown>
			);
		}
		return apiClient.post<PaymentGateway>(
			'/tenant/payments/gateways',
			data as unknown as Record<string, unknown>
		);
	},

	/**
	 * Delete a payment gateway (owner only)
	 * @param gatewayId - Gateway ID
	 */
	async deletePaymentGateway(gatewayId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`/tenant/payments/gateways/${gatewayId}`);
	},

	/**
	 * Set a gateway as default (owner only)
	 * @param gatewayId - Gateway ID
	 */
	async setDefaultGateway(gatewayId: string): Promise<ApiResponse<void>> {
		return apiClient.post<void>(`/tenant/payments/gateways/${gatewayId}/set-default`, {});
	},

	/**
	 * Test payment gateway connection (owner only)
	 * @param gatewayId - Gateway ID
	 */
	async testPaymentGateway(gatewayId: string): Promise<ApiResponse<TestPaymentResult>> {
		return apiClient.post<TestPaymentResult>(`/tenant/payments/gateways/${gatewayId}/test`, {});
	},

	/**
	 * Get webhook configuration for a gateway (owner only)
	 * @param gatewayId - Gateway ID
	 */
	async getGatewayWebhooks(gatewayId: string): Promise<ApiResponse<PaymentWebhookConfig[]>> {
		return apiClient.get<PaymentWebhookConfig[]>(`/tenant/payments/gateways/${gatewayId}/webhooks`);
	},

	/**
	 * Update payment methods settings (owner only)
	 * @param data - Payment methods configuration
	 */
	async updatePaymentMethods(
		data: UpdatePaymentMethodsRequest
	): Promise<ApiResponse<PaymentMethodSettings[]>> {
		return apiClient.put<PaymentMethodSettings[]>(
			'/tenant/payments/methods',
			data as unknown as Record<string, unknown>
		);
	},

	/**
	 * Update currency configuration (owner only)
	 * @param data - Currency settings
	 */
	async updateCurrencies(data: UpdateCurrenciesRequest): Promise<ApiResponse<CurrencyConfig[]>> {
		return apiClient.put<CurrencyConfig[]>(
			'/tenant/payments/currencies',
			data as unknown as Record<string, unknown>
		);
	},

	/**
	 * Update region configuration (owner only)
	 * @param data - Region settings
	 */
	async updateRegions(data: UpdateRegionsRequest): Promise<ApiResponse<PaymentRegionConfig[]>> {
		return apiClient.put<PaymentRegionConfig[]>(
			'/tenant/payments/regions',
			data as unknown as Record<string, unknown>
		);
	},

	/**
	 * Update payment security settings (owner only)
	 * @param data - Security configuration
	 */
	async updatePaymentSecurity(
		data: UpdatePaymentSecurityRequest
	): Promise<ApiResponse<PaymentSecuritySettings>> {
		return apiClient.put<PaymentSecuritySettings>(
			'/tenant/payments/security',
			data as unknown as Record<string, unknown>
		);
	},

	/**
	 * Get payment gateway health status (owner only)
	 */
	async getPaymentGatewayHealth(): Promise<ApiResponse<PaymentGatewayHealth[]>> {
		return apiClient.get<PaymentGatewayHealth[]>('/tenant/payments/health');
	},

	/**
	 * Get payment analytics (owner only)
	 * @param gatewayId - Optional gateway ID to filter
	 * @param period - Time period
	 */
	async getPaymentAnalytics(
		gatewayId?: string,
		period: 'day' | 'week' | 'month' = 'month'
	): Promise<ApiResponse<PaymentAnalytics>> {
		const params = new URLSearchParams({ period });
		if (gatewayId) params.set('gateway_id', gatewayId);
		return apiClient.get<PaymentAnalytics>(`/tenant/payments/analytics?${params}`);
	},

	/**
	 * Get transaction fees for a gateway (owner only)
	 * @param gatewayId - Gateway ID
	 */
	async getTransactionFees(gatewayId: string): Promise<ApiResponse<TransactionFeeConfig>> {
		return apiClient.get<TransactionFeeConfig>(`/tenant/payments/gateways/${gatewayId}/fees`);
	},

	/**
	 * Get settlement configuration for a gateway (owner only)
	 * @param gatewayId - Gateway ID
	 */
	async getSettlementConfig(gatewayId: string): Promise<ApiResponse<SettlementConfig>> {
		return apiClient.get<SettlementConfig>(`/tenant/payments/gateways/${gatewayId}/settlement`);
	},

	/**
	 * Update settlement configuration (owner only)
	 * @param gatewayId - Gateway ID
	 * @param data - Settlement settings
	 */
	async updateSettlementConfig(
		gatewayId: string,
		data: Partial<SettlementConfig>
	): Promise<ApiResponse<SettlementConfig>> {
		return apiClient.put<SettlementConfig>(
			`/tenant/payments/gateways/${gatewayId}/settlement`,
			data as unknown as Record<string, unknown>
		);
	}
};

export default userServiceApi;
