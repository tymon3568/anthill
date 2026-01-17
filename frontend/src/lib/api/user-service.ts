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
	Permission,
	Invitation,
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
	TenantValidation
} from './types/user-service.types';
import { tokenManager } from '$lib/auth/token-manager';
import { PUBLIC_API_BASE_URL } from '$env/static/public';

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

		const token = tokenManager.getAccessToken();
		const headers: HeadersInit = {};
		if (token) {
			headers['Authorization'] = `Bearer ${token}`;
		}

		try {
			const response = await fetch(`${PUBLIC_API_BASE_URL}/users/profile/avatar`, {
				method: 'POST',
				headers,
				body: formData
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

		return apiClient.get<PaginatedUsers>(`/users?${searchParams}`);
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
		return apiClient.post<User>('/admin/users', data as unknown as Record<string, unknown>);
	},

	/**
	 * Suspend a user (admin only)
	 * @param userId - User ID to suspend
	 */
	async suspendUser(userId: string): Promise<ApiResponse<void>> {
		return apiClient.post<void>(`/admin/users/${userId}/suspend`);
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
		return apiClient.get<Role[]>('/admin/roles');
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
	async getUserRoles(userId: string): Promise<ApiResponse<string[]>> {
		return apiClient.get<string[]>(`/admin/users/${userId}/roles`);
	},

	/**
	 * Assign a role to a user (admin only)
	 * @param userId - User ID
	 * @param roleName - Role name to assign
	 */
	async assignRole(userId: string, roleName: string): Promise<ApiResponse<void>> {
		return apiClient.post<void>(`/admin/users/${userId}/roles/assign`, { role: roleName });
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
		return apiClient.get<Permission[]>('/admin/permissions');
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

		return apiClient.get<PaginatedInvitations>(`/admin/users/invitations?${searchParams}`);
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
	}
};

export default userServiceApi;
