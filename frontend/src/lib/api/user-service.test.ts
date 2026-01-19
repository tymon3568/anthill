import { describe, it, expect, vi, beforeEach } from 'vitest';
import { userServiceApi, UserServiceApiError } from './user-service';
import { apiClient } from './client';
import type {
	UserProfile,
	User,
	Role,
	RoleListResponse,
	Permission,
	PermissionListResponse,
	Invitation,
	PaginatedUsers,
	PaginatedInvitations,
	CompletenessScore,
	PublicProfile,
	ProfileSearchResult,
	PermissionCheckResponse,
	TenantValidation,
	TenantSettings
} from './types/user-service.types';

// Mock the apiClient
vi.mock('./client', () => ({
	apiClient: {
		get: vi.fn(),
		post: vi.fn(),
		put: vi.fn(),
		delete: vi.fn()
	},
	createPaginationParams: vi.fn((page = 1, perPage = 10) => {
		const params = new URLSearchParams();
		params.set('page', page.toString());
		params.set('limit', perPage.toString());
		return params;
	})
}));

// Mock tenant module
vi.mock('$lib/tenant', () => ({
	getCurrentTenantSlug: vi.fn(() => 'test-tenant')
}));

// Mock environment variables - no longer needed as we use relative URLs
// The module now uses '/api/v1' as base URL for proxy routing

// Mock global fetch for upload methods
const mockFetch = vi.fn();
global.fetch = mockFetch;

describe('User Service API Client', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockFetch.mockReset();
	});

	// ============ Profile API Tests ============
	describe('Profile API', () => {
		const mockProfile: UserProfile = {
			userId: 'user-123',
			email: 'test@example.com',
			fullName: 'Test User',
			bio: 'A test user',
			title: 'Developer',
			department: 'Engineering',
			location: 'Remote',
			avatarUrl: 'https://example.com/avatar.png',
			language: 'en',
			timezone: 'UTC',
			profileVisibility: 'public',
			showEmail: true,
			showPhone: false,
			completenessScore: 85,
			verified: true
		};

		describe('getProfile', () => {
			it('should get user profile successfully', async () => {
				const mockResponse = { success: true, data: mockProfile };
				vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

				const result = await userServiceApi.getProfile();

				expect(apiClient.get).toHaveBeenCalledWith('/users/profile');
				expect(result).toEqual(mockResponse);
			});

			it('should handle error when getting profile', async () => {
				const mockResponse = { success: false, error: 'Unauthorized' };
				vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

				const result = await userServiceApi.getProfile();

				expect(result.success).toBe(false);
				expect(result.error).toBe('Unauthorized');
			});
		});

		describe('updateProfile', () => {
			it('should update user profile successfully', async () => {
				const updateData = { fullName: 'Updated Name', bio: 'Updated bio' };
				const mockResponse = { success: true, data: { ...mockProfile, ...updateData } };
				vi.mocked(apiClient.put).mockResolvedValue(mockResponse);

				const result = await userServiceApi.updateProfile(updateData);

				expect(apiClient.put).toHaveBeenCalledWith('/users/profile', updateData);
				expect(result.success).toBe(true);
				expect(result.data?.fullName).toBe('Updated Name');
			});
		});

		describe('uploadAvatar', () => {
			it('should upload avatar successfully', async () => {
				const file = new File(['test'], 'avatar.png', { type: 'image/png' });
				const mockResponse = { avatarUrl: 'https://example.com/new-avatar.png' };

				mockFetch.mockResolvedValue({
					ok: true,
					status: 200,
					json: async () => mockResponse
				});

				const result = await userServiceApi.uploadAvatar(file);

				expect(mockFetch).toHaveBeenCalledWith(
					'/api/v1/users/profile/avatar',
					expect.objectContaining({
						method: 'POST',
						credentials: 'include'
					})
				);
				expect(result.success).toBe(true);
				expect(result.data?.avatarUrl).toBe('https://example.com/new-avatar.png');
			});

			it('should handle upload error', async () => {
				const file = new File(['test'], 'avatar.png', { type: 'image/png' });

				mockFetch.mockResolvedValue({
					ok: false,
					status: 413,
					json: async () => ({ error: 'File too large' })
				});

				const result = await userServiceApi.uploadAvatar(file);

				expect(result.success).toBe(false);
				expect(result.error).toBe('File too large');
			});

			it('should handle network error during upload', async () => {
				const file = new File(['test'], 'avatar.png', { type: 'image/png' });

				mockFetch.mockRejectedValue(new Error('Network error'));

				const result = await userServiceApi.uploadAvatar(file);

				expect(result.success).toBe(false);
				expect(result.error).toBe('Network error');
			});
		});

		describe('updateVisibility', () => {
			it('should update visibility settings', async () => {
				const settings = {
					profileVisibility: 'private' as const,
					showEmail: false,
					showPhone: false
				};
				vi.mocked(apiClient.put).mockResolvedValue({ success: true });

				const result = await userServiceApi.updateVisibility(settings);

				expect(apiClient.put).toHaveBeenCalledWith('/users/profile/visibility', settings);
				expect(result.success).toBe(true);
			});
		});

		describe('getProfileCompleteness', () => {
			it('should get profile completeness score', async () => {
				const mockScore: CompletenessScore = {
					score: 75,
					missingFields: ['bio', 'location'],
					recommendations: ['Add a bio', 'Set your location']
				};
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: mockScore });

				const result = await userServiceApi.getProfileCompleteness();

				expect(apiClient.get).toHaveBeenCalledWith('/users/profile/completeness');
				expect(result.success).toBe(true);
				expect(result.data?.score).toBe(75);
			});
		});

		describe('searchProfiles', () => {
			it('should search profiles with query', async () => {
				const mockResult: ProfileSearchResult = {
					profiles: [{ userId: 'user-1', fullName: 'John Doe', verified: true }],
					total: 1,
					page: 1,
					perPage: 10
				};
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: mockResult });

				const result = await userServiceApi.searchProfiles({ query: 'john', page: 1 });

				expect(apiClient.get).toHaveBeenCalledWith(
					expect.stringContaining('/users/profiles/search?')
				);
				expect(apiClient.get).toHaveBeenCalledWith(expect.stringContaining('query=john'));
				expect(result.success).toBe(true);
			});
		});

		describe('getPublicProfile', () => {
			it('should get public profile by user ID', async () => {
				const mockPublicProfile: PublicProfile = {
					userId: 'user-123',
					fullName: 'John Doe',
					title: 'Developer',
					verified: true
				};
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: mockPublicProfile });

				const result = await userServiceApi.getPublicProfile('user-123');

				expect(apiClient.get).toHaveBeenCalledWith('/users/profiles/user-123');
				expect(result.success).toBe(true);
			});
		});
	});

	// ============ Admin User API Tests ============
	describe('Admin User API', () => {
		const mockUser: User = {
			id: 'user-123',
			email: 'user@example.com',
			fullName: 'Test User',
			role: 'user',
			status: 'active',
			emailVerified: true,
			createdAt: '2024-01-01T00:00:00Z'
		};

		describe('listUsers', () => {
			it('should list users with default params', async () => {
				const mockResponse: PaginatedUsers = {
					data: [mockUser],
					total: 1,
					page: 1,
					perPage: 10,
					totalPages: 1
				};
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: mockResponse });

				const result = await userServiceApi.listUsers();

				expect(apiClient.get).toHaveBeenCalledWith(expect.stringContaining('/admin/users?'));
				expect(result.success).toBe(true);
				expect(result.data?.data).toHaveLength(1);
			});

			it('should list users with filters', async () => {
				const mockResponse: PaginatedUsers = {
					data: [mockUser],
					total: 1,
					page: 1,
					perPage: 10,
					totalPages: 1
				};
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: mockResponse });

				await userServiceApi.listUsers({ role: 'admin', status: 'active', search: 'test' });

				expect(apiClient.get).toHaveBeenCalledWith(expect.stringContaining('role=admin'));
				expect(apiClient.get).toHaveBeenCalledWith(expect.stringContaining('status=active'));
				expect(apiClient.get).toHaveBeenCalledWith(expect.stringContaining('search=test'));
			});
		});

		describe('getUser', () => {
			it('should get user by ID', async () => {
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: mockUser });

				const result = await userServiceApi.getUser('user-123');

				expect(apiClient.get).toHaveBeenCalledWith('/admin/users/user-123');
				expect(result.success).toBe(true);
				expect(result.data?.id).toBe('user-123');
			});

			it('should handle 404 for non-existent user', async () => {
				vi.mocked(apiClient.get).mockResolvedValue({ success: false, error: 'User not found' });

				const result = await userServiceApi.getUser('non-existent');

				expect(result.success).toBe(false);
				expect(result.error).toBe('User not found');
			});
		});

		describe('createUser', () => {
			it('should create user successfully', async () => {
				const createData = {
					email: 'new@example.com',
					password: 'SecurePass123!',
					fullName: 'New User',
					role: 'user'
				};
				vi.mocked(apiClient.post).mockResolvedValue({
					success: true,
					data: { ...mockUser, ...createData }
				});

				const result = await userServiceApi.createUser(createData);

				expect(apiClient.post).toHaveBeenCalledWith('/admin/users', createData);
				expect(result.success).toBe(true);
			});

			it('should handle 409 conflict for duplicate email', async () => {
				const createData = {
					email: 'existing@example.com',
					password: 'SecurePass123!',
					fullName: 'New User',
					role: 'user'
				};
				vi.mocked(apiClient.post).mockResolvedValue({
					success: false,
					error: 'Email already exists'
				});

				const result = await userServiceApi.createUser(createData);

				expect(result.success).toBe(false);
				expect(result.error).toBe('Email already exists');
			});
		});

		describe('suspendUser', () => {
			it('should suspend user successfully', async () => {
				vi.mocked(apiClient.post).mockResolvedValue({ success: true });

				const result = await userServiceApi.suspendUser('user-123');

				expect(apiClient.post).toHaveBeenCalledWith('/admin/users/user-123/suspend');
				expect(result.success).toBe(true);
			});
		});

		describe('unsuspendUser', () => {
			it('should unsuspend user successfully', async () => {
				vi.mocked(apiClient.post).mockResolvedValue({ success: true });

				const result = await userServiceApi.unsuspendUser('user-123');

				expect(apiClient.post).toHaveBeenCalledWith('/admin/users/user-123/unsuspend');
				expect(result.success).toBe(true);
			});
		});

		describe('deleteUser', () => {
			it('should delete user successfully', async () => {
				vi.mocked(apiClient.delete).mockResolvedValue({ success: true });

				const result = await userServiceApi.deleteUser('user-123');

				expect(apiClient.delete).toHaveBeenCalledWith('/admin/users/user-123');
				expect(result.success).toBe(true);
			});
		});

		describe('resetUserPassword', () => {
			it('should reset user password', async () => {
				vi.mocked(apiClient.post).mockResolvedValue({ success: true });

				const result = await userServiceApi.resetUserPassword('user-123', 'NewPassword123!');

				expect(apiClient.post).toHaveBeenCalledWith('/admin/users/user-123/reset-password', {
					new_password: 'NewPassword123!'
				});
				expect(result.success).toBe(true);
			});
		});
	});

	// ============ Admin Role API Tests ============
	describe('Admin Role API', () => {
		const mockRole: Role = {
			role_name: 'editor',
			description: 'Can edit content',
			permissions: [
				{ resource: 'posts', action: 'read' },
				{ resource: 'posts', action: 'write' }
			],
			user_count: 5
		};

		describe('listRoles', () => {
			it('should list roles and extract roles array from response', async () => {
				const mockResponse: RoleListResponse = {
					roles: [mockRole],
					total: 1
				};
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: mockResponse });

				const result = await userServiceApi.listRoles();

				expect(apiClient.get).toHaveBeenCalledWith('/admin/roles');
				expect(result.success).toBe(true);
				expect(result.data).toHaveLength(1);
				expect(result.data?.[0].role_name).toBe('editor');
			});

			it('should handle error when listing roles', async () => {
				vi.mocked(apiClient.get).mockResolvedValue({ success: false, error: 'Forbidden' });

				const result = await userServiceApi.listRoles();

				expect(result.success).toBe(false);
				expect(result.error).toBe('Forbidden');
			});
		});

		describe('getRole', () => {
			it('should get role by name', async () => {
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: mockRole });

				const result = await userServiceApi.getRole('editor');

				expect(apiClient.get).toHaveBeenCalledWith('/admin/roles/editor');
				expect(result.success).toBe(true);
			});
		});

		describe('createRole', () => {
			it('should create role successfully', async () => {
				const createData = {
					role_name: 'reviewer',
					description: 'Can review content',
					permissions: [{ resource: 'posts', action: 'read' }]
				};
				vi.mocked(apiClient.post).mockResolvedValue({
					success: true,
					data: { ...mockRole, ...createData }
				});

				const result = await userServiceApi.createRole(createData);

				expect(apiClient.post).toHaveBeenCalledWith('/admin/roles', createData);
				expect(result.success).toBe(true);
			});
		});

		describe('updateRole', () => {
			it('should update role permissions', async () => {
				const updateData = {
					description: 'Updated description',
					permissions: [{ resource: 'posts', action: 'delete' }]
				};
				vi.mocked(apiClient.put).mockResolvedValue({
					success: true,
					data: { ...mockRole, ...updateData }
				});

				const result = await userServiceApi.updateRole('editor', updateData);

				expect(apiClient.put).toHaveBeenCalledWith('/admin/roles/editor', updateData);
				expect(result.success).toBe(true);
			});
		});

		describe('deleteRole', () => {
			it('should delete role successfully', async () => {
				vi.mocked(apiClient.delete).mockResolvedValue({ success: true });

				const result = await userServiceApi.deleteRole('custom-role');

				expect(apiClient.delete).toHaveBeenCalledWith('/admin/roles/custom-role');
				expect(result.success).toBe(true);
			});

			it('should handle 409 conflict when deleting role with users', async () => {
				vi.mocked(apiClient.delete).mockResolvedValue({
					success: false,
					error: 'Cannot delete role with assigned users'
				});

				const result = await userServiceApi.deleteRole('editor');

				expect(result.success).toBe(false);
				expect(result.error).toContain('Cannot delete');
			});
		});

		describe('getUserRoles', () => {
			it('should get roles for a user', async () => {
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: ['user', 'editor'] });

				const result = await userServiceApi.getUserRoles('user-123');

				expect(apiClient.get).toHaveBeenCalledWith('/admin/users/user-123/roles');
				expect(result.success).toBe(true);
				expect(result.data).toContain('editor');
			});
		});

		describe('assignRole', () => {
			it('should assign role to user', async () => {
				vi.mocked(apiClient.post).mockResolvedValue({ success: true });

				const result = await userServiceApi.assignRole('user-123', 'admin');

				expect(apiClient.post).toHaveBeenCalledWith('/admin/users/user-123/roles/assign', {
					role: 'admin'
				});
				expect(result.success).toBe(true);
			});
		});

		describe('removeRole', () => {
			it('should remove role from user', async () => {
				vi.mocked(apiClient.delete).mockResolvedValue({ success: true });

				const result = await userServiceApi.removeRole('user-123', 'editor');

				expect(apiClient.delete).toHaveBeenCalledWith('/admin/users/user-123/roles/editor/remove');
				expect(result.success).toBe(true);
			});
		});

		describe('listPermissions', () => {
			it('should list and flatten permissions', async () => {
				const mockResponse: PermissionListResponse = {
					permissions: [
						{
							resource: 'users',
							actions: ['read', 'write', 'delete'],
							description: 'User management'
						},
						{ resource: 'posts', actions: ['read', 'write'], description: 'Post management' }
					],
					total: 2
				};
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: mockResponse });

				const result = await userServiceApi.listPermissions();

				expect(apiClient.get).toHaveBeenCalledWith('/admin/permissions');
				expect(result.success).toBe(true);
				// Should flatten: users(3 actions) + posts(2 actions) = 5 permissions
				expect(result.data).toHaveLength(5);
				expect(
					result.data?.find((p) => p.resource === 'users' && p.action === 'delete')
				).toBeDefined();
			});

			it('should handle error when listing permissions', async () => {
				vi.mocked(apiClient.get).mockResolvedValue({ success: false, error: 'Forbidden' });

				const result = await userServiceApi.listPermissions();

				expect(result.success).toBe(false);
			});
		});
	});

	// ============ Admin Invitation API Tests ============
	describe('Admin Invitation API', () => {
		const mockInvitation: Invitation = {
			id: 'inv-123',
			email: 'invited@example.com',
			role: 'user',
			status: 'pending',
			invitedBy: 'admin-123',
			invitedByName: 'Admin User',
			expiresAt: '2024-01-10T00:00:00Z',
			createdAt: '2024-01-08T00:00:00Z'
		};

		describe('createInvitation', () => {
			it('should create invitation successfully', async () => {
				const createData = {
					email: 'newuser@example.com',
					role: 'user',
					customMessage: 'Welcome to the team!'
				};
				vi.mocked(apiClient.post).mockResolvedValue({ success: true, data: mockInvitation });

				const result = await userServiceApi.createInvitation(createData);

				expect(apiClient.post).toHaveBeenCalledWith('/admin/users/invite', createData);
				expect(result.success).toBe(true);
			});

			it('should handle 409 conflict for duplicate invitation', async () => {
				const createData = { email: 'existing@example.com', role: 'user' };
				vi.mocked(apiClient.post).mockResolvedValue({
					success: false,
					error: 'Invitation already exists for this email'
				});

				const result = await userServiceApi.createInvitation(createData);

				expect(result.success).toBe(false);
				expect(result.error).toContain('already exists');
			});

			it('should handle 429 rate limit', async () => {
				const createData = { email: 'test@example.com', role: 'user' };
				vi.mocked(apiClient.post).mockResolvedValue({
					success: false,
					error: 'Rate limit exceeded'
				});

				const result = await userServiceApi.createInvitation(createData);

				expect(result.success).toBe(false);
				expect(result.error).toContain('Rate limit');
			});
		});

		describe('listInvitations', () => {
			it('should list invitations with default params', async () => {
				const mockResponse: PaginatedInvitations = {
					data: [mockInvitation],
					total: 1,
					page: 1,
					perPage: 10,
					totalPages: 1
				};
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: mockResponse });

				const result = await userServiceApi.listInvitations();

				expect(apiClient.get).toHaveBeenCalledWith(
					expect.stringContaining('/admin/users/invitations?')
				);
				expect(result.success).toBe(true);
			});

			it('should list invitations with status filter', async () => {
				const mockResponse: PaginatedInvitations = {
					data: [mockInvitation],
					total: 1,
					page: 1,
					perPage: 10,
					totalPages: 1
				};
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: mockResponse });

				await userServiceApi.listInvitations({ status: 'pending' });

				expect(apiClient.get).toHaveBeenCalledWith(expect.stringContaining('status=pending'));
			});
		});

		describe('getInvitation', () => {
			it('should get invitation by ID', async () => {
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: mockInvitation });

				const result = await userServiceApi.getInvitation('inv-123');

				expect(apiClient.get).toHaveBeenCalledWith('/admin/users/invitations/inv-123');
				expect(result.success).toBe(true);
			});
		});

		describe('revokeInvitation', () => {
			it('should revoke invitation successfully', async () => {
				vi.mocked(apiClient.delete).mockResolvedValue({ success: true });

				const result = await userServiceApi.revokeInvitation('inv-123');

				expect(apiClient.delete).toHaveBeenCalledWith('/admin/users/invitations/inv-123');
				expect(result.success).toBe(true);
			});
		});

		describe('resendInvitation', () => {
			it('should resend invitation successfully', async () => {
				vi.mocked(apiClient.post).mockResolvedValue({ success: true });

				const result = await userServiceApi.resendInvitation('inv-123');

				expect(apiClient.post).toHaveBeenCalledWith('/admin/users/invitations/inv-123/resend');
				expect(result.success).toBe(true);
			});

			it('should handle 429 rate limit on resend', async () => {
				vi.mocked(apiClient.post).mockResolvedValue({
					success: false,
					error: 'Too many resend attempts'
				});

				const result = await userServiceApi.resendInvitation('inv-123');

				expect(result.success).toBe(false);
			});
		});
	});

	// ============ Permission Checking API Tests ============
	describe('Permission Checking API', () => {
		describe('checkPermission', () => {
			it('should check permission and return boolean', async () => {
				const mockResponse: PermissionCheckResponse = { allowed: true };
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: mockResponse });

				const result = await userServiceApi.checkPermission('products', 'write');

				expect(apiClient.get).toHaveBeenCalledWith(
					'/users/permissions/check?resource=products&action=write'
				);
				expect(result.success).toBe(true);
				expect(result.data).toBe(true);
			});

			it('should return false when permission denied', async () => {
				const mockResponse: PermissionCheckResponse = { allowed: false };
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: mockResponse });

				const result = await userServiceApi.checkPermission('admin', 'delete');

				expect(result.success).toBe(true);
				expect(result.data).toBe(false);
			});

			it('should handle error in permission check', async () => {
				vi.mocked(apiClient.get).mockResolvedValue({ success: false, error: 'Unauthorized' });

				const result = await userServiceApi.checkPermission('admin', 'read');

				expect(result.success).toBe(false);
			});
		});

		describe('getUserPermissions', () => {
			it('should get current user permissions', async () => {
				const mockPermissions: Permission[] = [
					{ resource: 'products', action: 'read' },
					{ resource: 'products', action: 'write' }
				];
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: mockPermissions });

				const result = await userServiceApi.getUserPermissions();

				expect(apiClient.get).toHaveBeenCalledWith('/users/permissions');
				expect(result.success).toBe(true);
				expect(result.data).toHaveLength(2);
			});
		});

		describe('getCurrentUserRoles', () => {
			it('should get current user roles', async () => {
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: ['user', 'editor'] });

				const result = await userServiceApi.getCurrentUserRoles();

				expect(apiClient.get).toHaveBeenCalledWith('/users/roles');
				expect(result.success).toBe(true);
				expect(result.data).toContain('editor');
			});
		});

		describe('validateTenantAccess', () => {
			it('should validate tenant access', async () => {
				const mockValidation: TenantValidation = {
					valid: true,
					tenantId: 'tenant-123',
					tenantName: 'Test Tenant',
					userRole: 'admin'
				};
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: mockValidation });

				const result = await userServiceApi.validateTenantAccess();

				expect(apiClient.get).toHaveBeenCalledWith('/users/tenant/validate');
				expect(result.success).toBe(true);
				expect(result.data?.valid).toBe(true);
			});
		});
	});

	// ============ Tenant Settings API Tests ============
	describe('Tenant Settings API', () => {
		describe('getTenantSettings', () => {
			it('should get tenant settings', async () => {
				const mockSettings: TenantSettings = {
					tenant: {
						tenantId: 'tenant-123',
						name: 'Test Tenant',
						slug: 'test-tenant',
						ownerUserId: 'user-123',
						plan: 'professional',
						status: 'active',
						createdAt: '2024-01-01T00:00:00Z'
					},
					branding: {
						primaryColor: '#3B82F6'
					},
					localization: {
						defaultTimezone: 'UTC',
						defaultCurrency: 'USD',
						defaultLanguage: 'en',
						dateFormat: 'YYYY-MM-DD',
						timeFormat: '24h'
					},
					securityPolicy: {
						passwordMinLength: 8,
						passwordRequireUppercase: true,
						passwordRequireLowercase: true,
						passwordRequireNumbers: true,
						passwordRequireSpecialChars: false,
						sessionTimeoutMinutes: 60,
						maxLoginAttempts: 5,
						lockoutDurationMinutes: 15,
						mfaRequired: false
					},
					dataRetention: {
						auditLogRetentionDays: 90,
						deletedUserRetentionDays: 30,
						sessionHistoryRetentionDays: 30,
						backupEnabled: true,
						backupFrequency: 'daily'
					}
				};
				vi.mocked(apiClient.get).mockResolvedValue({ success: true, data: mockSettings });

				const result = await userServiceApi.getTenantSettings();

				expect(apiClient.get).toHaveBeenCalledWith('/tenant/settings');
				expect(result.success).toBe(true);
				expect(result.data?.tenant.name).toBe('Test Tenant');
			});
		});

		describe('updateTenant', () => {
			it('should update tenant info', async () => {
				const updateData = { name: 'Updated Tenant Name' };
				vi.mocked(apiClient.put).mockResolvedValue({ success: true, data: {} });

				const result = await userServiceApi.updateTenant(updateData);

				expect(apiClient.put).toHaveBeenCalledWith('/tenant', updateData);
				expect(result.success).toBe(true);
			});
		});

		describe('updateBranding', () => {
			it('should update branding settings', async () => {
				const updateData = { primaryColor: '#FF0000', secondaryColor: '#00FF00' };
				vi.mocked(apiClient.put).mockResolvedValue({ success: true, data: updateData });

				const result = await userServiceApi.updateBranding(updateData);

				expect(apiClient.put).toHaveBeenCalledWith('/tenant/branding', updateData);
				expect(result.success).toBe(true);
			});
		});

		describe('uploadLogo', () => {
			it('should upload tenant logo', async () => {
				const file = new File(['test'], 'logo.png', { type: 'image/png' });
				mockFetch.mockResolvedValue({
					ok: true,
					status: 200,
					json: async () => ({ logoUrl: 'https://example.com/logo.png' })
				});

				const result = await userServiceApi.uploadLogo(file);

				expect(mockFetch).toHaveBeenCalledWith(
					'/api/v1/tenant/branding/logo',
					expect.objectContaining({ method: 'POST' })
				);
				expect(result.success).toBe(true);
			});
		});

		describe('updateSecurityPolicy', () => {
			it('should update security policy', async () => {
				const updateData = { passwordMinLength: 12, mfaRequired: true };
				vi.mocked(apiClient.put).mockResolvedValue({ success: true, data: updateData });

				const result = await userServiceApi.updateSecurityPolicy(updateData);

				expect(apiClient.put).toHaveBeenCalledWith('/tenant/security-policy', updateData);
				expect(result.success).toBe(true);
			});
		});

		describe('getTenantBilling', () => {
			it('should get tenant billing info', async () => {
				vi.mocked(apiClient.get).mockResolvedValue({
					success: true,
					data: { plan: 'professional', billingEmail: 'billing@example.com' }
				});

				const result = await userServiceApi.getTenantBilling();

				expect(apiClient.get).toHaveBeenCalledWith('/tenant/billing');
				expect(result.success).toBe(true);
			});
		});

		describe('listAuditLogs', () => {
			it('should list audit logs with filters', async () => {
				vi.mocked(apiClient.get).mockResolvedValue({
					success: true,
					data: { data: [], total: 0, page: 1, perPage: 10, totalPages: 0 }
				});

				await userServiceApi.listAuditLogs({ userId: 'user-123', action: 'login' });

				expect(apiClient.get).toHaveBeenCalledWith(expect.stringContaining('/tenant/audit-logs?'));
				expect(apiClient.get).toHaveBeenCalledWith(expect.stringContaining('user_id=user-123'));
				expect(apiClient.get).toHaveBeenCalledWith(expect.stringContaining('action=login'));
			});
		});

		describe('getTenantAnalytics', () => {
			it('should get tenant analytics', async () => {
				vi.mocked(apiClient.get).mockResolvedValue({
					success: true,
					data: { activeUsersLast30Days: 50, totalUsers: 100 }
				});

				const result = await userServiceApi.getTenantAnalytics();

				expect(apiClient.get).toHaveBeenCalledWith('/tenant/analytics');
				expect(result.success).toBe(true);
			});
		});

		describe('exportTenantData', () => {
			it('should export tenant data', async () => {
				vi.mocked(apiClient.post).mockResolvedValue({
					success: true,
					data: { downloadUrl: 'https://example.com/export.zip' }
				});

				const result = await userServiceApi.exportTenantData({
					format: 'json',
					includeUsers: true,
					includeAuditLogs: true
				});

				expect(apiClient.post).toHaveBeenCalledWith('/tenant/export', expect.any(Object));
				expect(result.success).toBe(true);
			});
		});

		describe('deleteTenant', () => {
			it('should delete tenant with confirmation', async () => {
				vi.mocked(apiClient.post).mockResolvedValue({ success: true });

				const result = await userServiceApi.deleteTenant({
					confirmTenantName: 'Test Tenant',
					reason: 'No longer needed'
				});

				expect(apiClient.post).toHaveBeenCalledWith(
					'/tenant/delete',
					expect.objectContaining({
						confirmTenantName: 'Test Tenant'
					})
				);
				expect(result.success).toBe(true);
			});
		});
	});

	// ============ Payment Gateway API Tests ============
	describe('Payment Gateway API', () => {
		describe('getPaymentSettings', () => {
			it('should get payment settings', async () => {
				vi.mocked(apiClient.get).mockResolvedValue({
					success: true,
					data: { gateways: [], paymentMethods: [], currencies: [], regions: [], security: {} }
				});

				const result = await userServiceApi.getPaymentSettings();

				expect(apiClient.get).toHaveBeenCalledWith('/tenant/payments');
				expect(result.success).toBe(true);
			});
		});

		describe('upsertPaymentGateway', () => {
			it('should create new payment gateway', async () => {
				const gatewayData = {
					provider: 'stripe' as const,
					name: 'Stripe',
					isSandbox: true,
					publicKey: 'pk_test_xxx'
				};
				vi.mocked(apiClient.post).mockResolvedValue({
					success: true,
					data: { id: 'gw-123', ...gatewayData }
				});

				const result = await userServiceApi.upsertPaymentGateway(undefined, gatewayData);

				expect(apiClient.post).toHaveBeenCalledWith('/tenant/payments/gateways', gatewayData);
				expect(result.success).toBe(true);
			});

			it('should update existing payment gateway', async () => {
				const gatewayData = {
					provider: 'stripe' as const,
					name: 'Stripe Updated',
					isSandbox: false
				};
				vi.mocked(apiClient.put).mockResolvedValue({
					success: true,
					data: { id: 'gw-123', ...gatewayData }
				});

				const result = await userServiceApi.upsertPaymentGateway('gw-123', gatewayData);

				expect(apiClient.put).toHaveBeenCalledWith('/tenant/payments/gateways/gw-123', gatewayData);
				expect(result.success).toBe(true);
			});
		});

		describe('testPaymentGateway', () => {
			it('should test payment gateway connection', async () => {
				vi.mocked(apiClient.post).mockResolvedValue({
					success: true,
					data: { success: true, latencyMs: 150 }
				});

				const result = await userServiceApi.testPaymentGateway('gw-123');

				expect(apiClient.post).toHaveBeenCalledWith('/tenant/payments/gateways/gw-123/test', {});
				expect(result.success).toBe(true);
			});
		});

		describe('getPaymentGatewayHealth', () => {
			it('should get gateway health status', async () => {
				vi.mocked(apiClient.get).mockResolvedValue({
					success: true,
					data: [{ gatewayId: 'gw-123', status: 'healthy', latencyMs: 100 }]
				});

				const result = await userServiceApi.getPaymentGatewayHealth();

				expect(apiClient.get).toHaveBeenCalledWith('/tenant/payments/health');
				expect(result.success).toBe(true);
			});
		});

		describe('getPaymentAnalytics', () => {
			it('should get payment analytics with default period', async () => {
				vi.mocked(apiClient.get).mockResolvedValue({
					success: true,
					data: { totalTransactions: 100, totalVolume: 10000 }
				});

				const result = await userServiceApi.getPaymentAnalytics();

				expect(apiClient.get).toHaveBeenCalledWith('/tenant/payments/analytics?period=month');
				expect(result.success).toBe(true);
			});

			it('should get payment analytics with gateway filter', async () => {
				vi.mocked(apiClient.get).mockResolvedValue({
					success: true,
					data: { totalTransactions: 50, totalVolume: 5000 }
				});

				await userServiceApi.getPaymentAnalytics('gw-123', 'week');

				expect(apiClient.get).toHaveBeenCalledWith(expect.stringContaining('gateway_id=gw-123'));
				expect(apiClient.get).toHaveBeenCalledWith(expect.stringContaining('period=week'));
			});
		});
	});

	// ============ Error Handling Tests ============
	describe('Error Handling', () => {
		it('should handle 403 Forbidden errors', async () => {
			vi.mocked(apiClient.get).mockResolvedValue({
				success: false,
				error: 'Forbidden: You do not have permission to access this resource'
			});

			const result = await userServiceApi.listUsers();

			expect(result.success).toBe(false);
			expect(result.error).toContain('Forbidden');
		});

		it('should handle 404 Not Found errors', async () => {
			vi.mocked(apiClient.get).mockResolvedValue({
				success: false,
				error: 'User not found'
			});

			const result = await userServiceApi.getUser('non-existent');

			expect(result.success).toBe(false);
			expect(result.error).toContain('not found');
		});

		it('should handle 500 Internal Server errors', async () => {
			vi.mocked(apiClient.get).mockResolvedValue({
				success: false,
				error: 'Internal server error'
			});

			const result = await userServiceApi.getProfile();

			expect(result.success).toBe(false);
			expect(result.error).toContain('Internal server');
		});

		it('should handle network errors', async () => {
			vi.mocked(apiClient.get).mockRejectedValue(new Error('Network error'));

			await expect(userServiceApi.getProfile()).rejects.toThrow('Network error');
		});
	});
});

describe('UserServiceApiError', () => {
	it('should create error with status, code, and message', () => {
		const error = new UserServiceApiError(404, 'NOT_FOUND', 'User not found');

		expect(error.status).toBe(404);
		expect(error.code).toBe('NOT_FOUND');
		expect(error.message).toBe('User not found');
		expect(error.name).toBe('UserServiceApiError');
	});
});
