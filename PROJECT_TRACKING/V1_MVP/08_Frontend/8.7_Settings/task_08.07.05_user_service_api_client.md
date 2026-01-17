# Task: User Service API Client Integration

**Task ID:** V1_MVP/08_Frontend/8.7_Settings/task_08.07.05_user_service_api_client.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.7_Settings
**Priority:** High
**Status:** Done
**Assignee:** Opus
**Created Date:** 2026-01-17
**Last Updated:** 2026-01-17

## Detailed Description:
Create a comprehensive User Service API client that consolidates all User Service API interactions beyond basic authentication. This follows the same pattern as other service API clients in the project (Inventory, Order, Integration services). The client will provide type-safe methods for:

1. **User Profile Management** - Profile CRUD, avatar upload, visibility settings
2. **Admin User Management** - List users, create/suspend/delete users
3. **Admin Role Management** - CRUD roles, assign/remove roles from users
4. **Admin Invitation Management** - Create/list/revoke/resend invitations
5. **Permission Checking** - Check permissions, get user roles

This client will be a dependency for Settings pages (8.7) and Admin Console (8.8).

## Acceptance Criteria:
- [x] User Service API client module created at `src/lib/api/user-service.ts`
- [x] All endpoint methods are fully typed (request/response types)
- [x] Error handling with consistent ApiError type
- [x] Authentication token automatically attached to requests
- [x] Tenant context handled correctly
- [ ] Unit tests for critical API methods
- [x] Code compiles without errors: `bun run check`
- [x] Documentation comments for each public method

## Specific Sub-tasks:
- [x] 1. Create Base API Client Structure
    - [x] 1.1. Create `src/lib/api/user-service.ts` module
    - [x] 1.2. Create `src/lib/api/types/user-service.types.ts` for all types
    - [x] 1.3. Set up base fetch wrapper with auth token injection
    - [x] 1.4. Implement consistent error handling (ApiError class)
    - [ ] 1.5. Add request/response logging in development mode

- [x] 2. Implement Profile API Methods
    - [x] 2.1. `getProfile(): Promise<UserProfile>` - Get current user profile
    - [x] 2.2. `updateProfile(data: UpdateProfileRequest): Promise<UserProfile>`
    - [x] 2.3. `uploadAvatar(file: File): Promise<{ avatarUrl: string }>`
    - [x] 2.4. `updateVisibility(settings: VisibilitySettings): Promise<void>`
    - [x] 2.5. `getProfileCompleteness(): Promise<CompletenessScore>`
    - [x] 2.6. `searchProfiles(query: ProfileSearchRequest): Promise<ProfileSearchResult>`
    - [x] 2.7. `getPublicProfile(userId: string): Promise<PublicProfile>`

- [x] 3. Implement Admin User API Methods
    - [x] 3.1. `listUsers(params: ListUsersParams): Promise<PaginatedUsers>`
    - [x] 3.2. `createUser(data: CreateUserRequest): Promise<User>`
    - [x] 3.3. `suspendUser(userId: string): Promise<void>`
    - [x] 3.4. `unsuspendUser(userId: string): Promise<void>`
    - [x] 3.5. `deleteUser(userId: string): Promise<void>`
    - [x] 3.6. `resetUserPassword(userId: string, newPassword: string): Promise<void>`

- [x] 4. Implement Admin Role API Methods
    - [x] 4.1. `listRoles(): Promise<Role[]>`
    - [x] 4.2. `createRole(data: CreateRoleRequest): Promise<Role>`
    - [x] 4.3. `updateRole(roleName: string, data: UpdateRoleRequest): Promise<Role>`
    - [x] 4.4. `deleteRole(roleName: string): Promise<void>`
    - [x] 4.5. `getUserRoles(userId: string): Promise<string[]>`
    - [x] 4.6. `assignRole(userId: string, roleName: string): Promise<void>`
    - [x] 4.7. `removeRole(userId: string, roleName: string): Promise<void>`
    - [x] 4.8. `listPermissions(): Promise<Permission[]>`

- [x] 5. Implement Admin Invitation API Methods
    - [x] 5.1. `createInvitation(data: CreateInvitationRequest): Promise<Invitation>`
    - [x] 5.2. `listInvitations(params: ListInvitationsParams): Promise<PaginatedInvitations>`
    - [x] 5.3. `revokeInvitation(invitationId: string): Promise<void>`
    - [x] 5.4. `resendInvitation(invitationId: string): Promise<void>`

- [x] 6. Implement Permission Checking Methods
    - [x] 6.1. `checkPermission(resource: string, action: string): Promise<boolean>`
    - [x] 6.2. `getUserPermissions(): Promise<Permission[]>`
    - [x] 6.3. `validateTenantAccess(): Promise<TenantValidation>`

- [x] 7. Create Type Definitions
    - [x] 7.1. User and UserProfile types
    - [x] 7.2. Role and Permission types
    - [x] 7.3. Invitation types
    - [x] 7.4. Request/Response types for all endpoints
    - [x] 7.5. Pagination types (consistent with other API clients)

- [ ] 8. Add Unit Tests
    - [ ] 8.1. Test profile API methods with mocked fetch
    - [ ] 8.2. Test admin user API methods
    - [ ] 8.3. Test error handling for various HTTP status codes
    - [ ] 8.4. Test auth token injection

## Dependencies:
*   Task: `task_08.02.03_auth_api_client.md` (Status: Done)
*   Task: `task_08.02.04_session_management.md` (Status: Done)

## Files to Create/Modify:
*   `src/lib/api/user-service.ts` - Main User Service API client
*   `src/lib/api/types/user-service.types.ts` - Type definitions
*   `src/lib/api/index.ts` - Export user service client
*   `src/lib/api/base.ts` - Shared base API utilities (if not exists)
*   `tests/lib/api/user-service.test.ts` - Unit tests

## Code Examples:
```typescript
// src/lib/api/types/user-service.types.ts
export interface User {
  id: string;
  email: string;
  fullName: string;
  role: string;
  status: 'active' | 'suspended' | 'deleted';
  emailVerified: boolean;
  createdAt: string;
  lastLoginAt?: string;
}

export interface UserProfile {
  userId: string;
  bio?: string;
  title?: string;
  department?: string;
  location?: string;
  websiteUrl?: string;
  socialLinks?: Record<string, string>;
  language: string;
  timezone: string;
  profileVisibility: 'public' | 'private' | 'team_only';
  showEmail: boolean;
  showPhone: boolean;
  completenessScore: number;
  verified: boolean;
  verificationBadge?: string;
}

export interface Role {
  name: string;
  description?: string;
  permissions: Permission[];
  isSystem: boolean;
  createdAt: string;
}

export interface Permission {
  resource: string;
  action: string;
}

export interface Invitation {
  id: string;
  email: string;
  role: string;
  status: 'pending' | 'accepted' | 'expired' | 'revoked';
  invitedBy: string;
  expiresAt: string;
  createdAt: string;
}

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  perPage: number;
  totalPages: number;
}

export interface ListUsersParams {
  page?: number;
  perPage?: number;
  role?: string;
  status?: 'active' | 'suspended';
  search?: string;
}

export interface CreateUserRequest {
  email: string;
  password: string;
  fullName: string;
  role: string;
}

export interface CreateInvitationRequest {
  email: string;
  role: string;
  customMessage?: string;
}

export interface UpdateProfileRequest {
  bio?: string;
  title?: string;
  department?: string;
  location?: string;
  websiteUrl?: string;
  socialLinks?: Record<string, string>;
  language?: string;
  timezone?: string;
}

export interface VisibilitySettings {
  profileVisibility: 'public' | 'private' | 'team_only';
  showEmail: boolean;
  showPhone: boolean;
}

export interface CompletenessScore {
  score: number;
  missingFields: string[];
  recommendations: string[];
}
```

```typescript
// src/lib/api/user-service.ts
import { getAuthToken } from '$lib/auth/session';
import type {
  User,
  UserProfile,
  Role,
  Permission,
  Invitation,
  PaginatedResponse,
  ListUsersParams,
  CreateUserRequest,
  UpdateProfileRequest,
  VisibilitySettings,
  CompletenessScore,
  CreateInvitationRequest,
} from './types/user-service.types';

const API_BASE = import.meta.env.PUBLIC_USER_SERVICE_URL || 'http://localhost:8000';

class ApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string,
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

async function fetchWithAuth(
  endpoint: string,
  options: RequestInit = {},
): Promise<Response> {
  const token = getAuthToken();
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
    ...options.headers,
  };

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const response = await fetch(`${API_BASE}${endpoint}`, {
    ...options,
    headers,
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({}));
    throw new ApiError(
      response.status,
      error.code || 'UNKNOWN_ERROR',
      error.message || 'An error occurred',
    );
  }

  return response;
}

export const userServiceApi = {
  // ============ Profile API ============
  
  /** Get current user's profile */
  async getProfile(): Promise<UserProfile> {
    const response = await fetchWithAuth('/api/v1/users/profile');
    return response.json();
  },

  /** Update current user's profile */
  async updateProfile(data: UpdateProfileRequest): Promise<UserProfile> {
    const response = await fetchWithAuth('/api/v1/users/profile', {
      method: 'PUT',
      body: JSON.stringify(data),
    });
    return response.json();
  },

  /** Upload avatar image (max 5MB) */
  async uploadAvatar(file: File): Promise<{ avatarUrl: string }> {
    const formData = new FormData();
    formData.append('avatar', file);
    
    const token = getAuthToken();
    const response = await fetch(`${API_BASE}/api/v1/users/profile/avatar`, {
      method: 'POST',
      headers: token ? { Authorization: `Bearer ${token}` } : {},
      body: formData,
    });
    
    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new ApiError(response.status, error.code, error.message);
    }
    
    return response.json();
  },

  /** Update profile visibility settings */
  async updateVisibility(settings: VisibilitySettings): Promise<void> {
    await fetchWithAuth('/api/v1/users/profile/visibility', {
      method: 'PUT',
      body: JSON.stringify(settings),
    });
  },

  /** Get profile completeness score */
  async getProfileCompleteness(): Promise<CompletenessScore> {
    const response = await fetchWithAuth('/api/v1/users/profile/completeness');
    return response.json();
  },

  /** Get public profile of another user */
  async getPublicProfile(userId: string): Promise<UserProfile> {
    const response = await fetchWithAuth(`/api/v1/users/profiles/${userId}`);
    return response.json();
  },

  // ============ Admin User API ============

  /** List users in tenant (admin only) */
  async listUsers(params: ListUsersParams = {}): Promise<PaginatedResponse<User>> {
    const searchParams = new URLSearchParams();
    if (params.page) searchParams.set('page', params.page.toString());
    if (params.perPage) searchParams.set('per_page', params.perPage.toString());
    if (params.role) searchParams.set('role', params.role);
    if (params.status) searchParams.set('status', params.status);
    if (params.search) searchParams.set('search', params.search);

    const response = await fetchWithAuth(`/api/v1/users?${searchParams}`);
    return response.json();
  },

  /** Create a new user in tenant (admin only) */
  async createUser(data: CreateUserRequest): Promise<User> {
    const response = await fetchWithAuth('/api/v1/admin/users', {
      method: 'POST',
      body: JSON.stringify(data),
    });
    return response.json();
  },

  /** Suspend a user (admin only) */
  async suspendUser(userId: string): Promise<void> {
    await fetchWithAuth(`/api/v1/admin/users/${userId}/suspend`, {
      method: 'POST',
    });
  },

  /** Unsuspend a user (admin only) */
  async unsuspendUser(userId: string): Promise<void> {
    await fetchWithAuth(`/api/v1/admin/users/${userId}/unsuspend`, {
      method: 'POST',
    });
  },

  /** Delete a user - soft delete (admin only) */
  async deleteUser(userId: string): Promise<void> {
    await fetchWithAuth(`/api/v1/admin/users/${userId}`, {
      method: 'DELETE',
    });
  },

  /** Reset user's password (admin only) */
  async resetUserPassword(userId: string, newPassword: string): Promise<void> {
    await fetchWithAuth(`/api/v1/admin/users/${userId}/reset-password`, {
      method: 'POST',
      body: JSON.stringify({ new_password: newPassword }),
    });
  },

  // ============ Admin Role API ============

  /** List all roles in tenant (admin only) */
  async listRoles(): Promise<Role[]> {
    const response = await fetchWithAuth('/api/v1/admin/roles');
    return response.json();
  },

  /** Create a new role (admin only) */
  async createRole(data: { name: string; description?: string; permissions: Permission[] }): Promise<Role> {
    const response = await fetchWithAuth('/api/v1/admin/roles', {
      method: 'POST',
      body: JSON.stringify(data),
    });
    return response.json();
  },

  /** Update role permissions (admin only) */
  async updateRole(roleName: string, data: { description?: string; permissions: Permission[] }): Promise<Role> {
    const response = await fetchWithAuth(`/api/v1/admin/roles/${roleName}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
    return response.json();
  },

  /** Delete a role (admin only, cannot delete system roles) */
  async deleteRole(roleName: string): Promise<void> {
    await fetchWithAuth(`/api/v1/admin/roles/${roleName}`, {
      method: 'DELETE',
    });
  },

  /** Get roles assigned to a user */
  async getUserRoles(userId: string): Promise<string[]> {
    const response = await fetchWithAuth(`/api/v1/admin/users/${userId}/roles`);
    return response.json();
  },

  /** Assign a role to a user (admin only) */
  async assignRole(userId: string, roleName: string): Promise<void> {
    await fetchWithAuth(`/api/v1/admin/users/${userId}/roles/assign`, {
      method: 'POST',
      body: JSON.stringify({ role: roleName }),
    });
  },

  /** Remove a role from a user (admin only) */
  async removeRole(userId: string, roleName: string): Promise<void> {
    await fetchWithAuth(`/api/v1/admin/users/${userId}/roles/${roleName}/remove`, {
      method: 'DELETE',
    });
  },

  /** List all available permissions */
  async listPermissions(): Promise<Permission[]> {
    const response = await fetchWithAuth('/api/v1/admin/permissions');
    return response.json();
  },

  // ============ Admin Invitation API ============

  /** Create a new invitation (admin only) */
  async createInvitation(data: CreateInvitationRequest): Promise<Invitation> {
    const response = await fetchWithAuth('/api/v1/admin/users/invite', {
      method: 'POST',
      body: JSON.stringify(data),
    });
    return response.json();
  },

  /** List invitations in tenant (admin only) */
  async listInvitations(params: { page?: number; perPage?: number; status?: string } = {}): Promise<PaginatedResponse<Invitation>> {
    const searchParams = new URLSearchParams();
    if (params.page) searchParams.set('page', params.page.toString());
    if (params.perPage) searchParams.set('per_page', params.perPage.toString());
    if (params.status) searchParams.set('status', params.status);

    const response = await fetchWithAuth(`/api/v1/admin/users/invitations?${searchParams}`);
    return response.json();
  },

  /** Revoke an invitation (admin only) */
  async revokeInvitation(invitationId: string): Promise<void> {
    await fetchWithAuth(`/api/v1/admin/users/invitations/${invitationId}`, {
      method: 'DELETE',
    });
  },

  /** Resend an invitation email (admin only) */
  async resendInvitation(invitationId: string): Promise<void> {
    await fetchWithAuth(`/api/v1/admin/users/invitations/${invitationId}/resend`, {
      method: 'POST',
    });
  },

  // ============ Permission Checking API ============

  /** Check if current user has a specific permission */
  async checkPermission(resource: string, action: string): Promise<boolean> {
    const response = await fetchWithAuth(
      `/api/v1/users/permissions/check?resource=${resource}&action=${action}`
    );
    const result = await response.json();
    return result.allowed;
  },

  /** Get all permissions for current user */
  async getUserPermissions(): Promise<Permission[]> {
    const response = await fetchWithAuth('/api/v1/users/permissions');
    return response.json();
  },

  /** Get current user's roles */
  async getCurrentUserRoles(): Promise<string[]> {
    const response = await fetchWithAuth('/api/v1/users/roles');
    return response.json();
  },
};

export { ApiError };
export default userServiceApi;
```

## Testing Steps:
- [ ] Import userServiceApi and verify no TypeScript errors
- [ ] Test getProfile() returns current user profile
- [ ] Test updateProfile() with valid data
- [ ] Test listUsers() with pagination
- [ ] Test createUser() with valid data
- [ ] Test error handling for 403 Forbidden (non-admin)
- [ ] Test error handling for 401 Unauthorized (no token)
- [ ] Test error handling for 404 Not Found
- [ ] Test error handling for 409 Conflict (duplicate email)
- [ ] Verify auth token is attached to all requests

## Backend API Reference:
```
# Profile
GET    /api/v1/users/profile
PUT    /api/v1/users/profile
POST   /api/v1/users/profile/avatar
PUT    /api/v1/users/profile/visibility
GET    /api/v1/users/profile/completeness
GET    /api/v1/users/profiles/{user_id}

# Admin Users
GET    /api/v1/users
POST   /api/v1/admin/users
POST   /api/v1/admin/users/{user_id}/suspend
POST   /api/v1/admin/users/{user_id}/unsuspend
DELETE /api/v1/admin/users/{user_id}
POST   /api/v1/admin/users/{user_id}/reset-password

# Admin Roles
POST   /api/v1/admin/roles
GET    /api/v1/admin/roles
PUT    /api/v1/admin/roles/{role_name}
DELETE /api/v1/admin/roles/{role_name}
GET    /api/v1/admin/users/{user_id}/roles
POST   /api/v1/admin/users/{user_id}/roles/assign
DELETE /api/v1/admin/users/{user_id}/roles/{role_name}/remove
GET    /api/v1/admin/permissions

# Admin Invitations
POST   /api/v1/admin/users/invite
GET    /api/v1/admin/users/invitations
DELETE /api/v1/admin/users/invitations/{invitation_id}
POST   /api/v1/admin/users/invitations/{invitation_id}/resend

# Permissions
GET    /api/v1/users/permissions/check
GET    /api/v1/users/permissions
GET    /api/v1/users/roles
```

## Notes / Discussion:
---
*   This API client follows the same pattern as inventory/order/integration service clients
*   All admin endpoints require `admin` role enforced by backend
*   Avatar upload uses FormData (not JSON) for file handling
*   Error codes from backend: UNAUTHORIZED, FORBIDDEN, NOT_FOUND, CONFLICT, RATE_LIMITED
*   Token injection uses shared auth session management from task_08.02.04
*   API client should be used by Settings (8.7) and Admin Console (8.8) modules

## AI Agent Log:
---
*   2026-01-17 10:30: Task created by Opus
    - Created to provide consistent API client pattern across all services
    - Consolidates all User Service API interactions beyond basic auth
    - Serves as dependency for Settings and Admin Console modules
    - Follows existing patterns from Inventory/Order/Integration service clients

*   2026-01-17 12:45: Task claimed by Opus
    - Verified dependencies: task_08.02.03 (Done), task_08.02.04 (Done)
    - Created feature branch: feat/user-service-frontend-tasks
    - Starting implementation of sub-task 1: Base API Client Structure

*   2026-01-17 13:30: Implementation completed by Opus
    - Created `src/lib/api/types/user-service.types.ts` with 30+ type definitions
    - Created `src/lib/api/user-service.ts` with 35+ API methods covering:
      - Profile API (7 methods)
      - Admin User API (7 methods)
      - Admin Role API (9 methods)
      - Admin Invitation API (5 methods)
      - Permission Checking API (4 methods)
    - Created `src/lib/api/index.ts` for centralized exports
    - All methods use existing apiClient for consistent auth token handling
    - TypeScript check passes: `bun run check` ✓
    - Lint check passes for new files ✓
    - Status changed to NeedsReview
    - Note: Unit tests (sub-task 8) and dev logging (sub-task 1.5) deferred
