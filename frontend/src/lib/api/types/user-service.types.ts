/**
 * User Service API Type Definitions
 *
 * Type definitions for all User Service API interactions including:
 * - User Profile Management
 * - Admin User Management
 * - Admin Role Management
 * - Admin Invitation Management
 * - Permission Checking
 */

// ============ User Types ============

/** User status in the system */
export type UserStatus = 'active' | 'suspended' | 'deleted';

/** Profile visibility options */
export type ProfileVisibility = 'public' | 'private' | 'team_only';

/** Invitation status */
export type InvitationStatus = 'pending' | 'accepted' | 'expired' | 'revoked';

/** Base user information */
export interface User {
	id: string;
	email: string;
	fullName: string;
	role: string;
	status: UserStatus;
	emailVerified: boolean;
	createdAt: string;
	lastLoginAt?: string;
}

/** Extended user profile information */
export interface UserProfile {
	userId: string;
	email: string;
	fullName: string;
	bio?: string;
	title?: string;
	department?: string;
	location?: string;
	avatarUrl?: string;
	websiteUrl?: string;
	socialLinks?: Record<string, string>;
	language: string;
	timezone: string;
	profileVisibility: ProfileVisibility;
	showEmail: boolean;
	showPhone: boolean;
	completenessScore: number;
	verified: boolean;
	verificationBadge?: string;
}

/** Public profile visible to other users */
export interface PublicProfile {
	userId: string;
	fullName: string;
	bio?: string;
	title?: string;
	department?: string;
	location?: string;
	avatarUrl?: string;
	verified: boolean;
	verificationBadge?: string;
}

// ============ Role & Permission Types ============

/** Permission definition */
export interface Permission {
	resource: string;
	action: string;
}

/** Role definition */
export interface Role {
	name: string;
	description?: string;
	permissions: Permission[];
	isSystem: boolean;
	createdAt: string;
}

// ============ Invitation Types ============

/** User invitation */
export interface Invitation {
	id: string;
	email: string;
	role: string;
	status: InvitationStatus;
	invitedBy: string;
	invitedByName?: string;
	expiresAt: string;
	createdAt: string;
	acceptedAt?: string;
}

// ============ Request Types ============

/** Parameters for listing users */
export interface ListUsersParams {
	page?: number;
	perPage?: number;
	role?: string;
	status?: UserStatus;
	search?: string;
}

/** Parameters for listing invitations */
export interface ListInvitationsParams {
	page?: number;
	perPage?: number;
	status?: InvitationStatus;
}

/** Request to create a new user */
export interface CreateUserRequest {
	email: string;
	password: string;
	fullName: string;
	role: string;
}

/** Request to update user profile */
export interface UpdateProfileRequest {
	fullName?: string;
	bio?: string;
	title?: string;
	department?: string;
	location?: string;
	websiteUrl?: string;
	socialLinks?: Record<string, string>;
	language?: string;
	timezone?: string;
}

/** Request to update profile visibility */
export interface VisibilitySettings {
	profileVisibility: ProfileVisibility;
	showEmail: boolean;
	showPhone: boolean;
}

/** Request to create a new invitation */
export interface CreateInvitationRequest {
	email: string;
	role: string;
	customMessage?: string;
}

/** Request to create a new role */
export interface CreateRoleRequest {
	name: string;
	description?: string;
	permissions: Permission[];
}

/** Request to update a role */
export interface UpdateRoleRequest {
	description?: string;
	permissions: Permission[];
}

/** Request for profile search */
export interface ProfileSearchRequest {
	query: string;
	department?: string;
	page?: number;
	perPage?: number;
}

// ============ Response Types ============

/** Generic paginated response */
export interface PaginatedResponse<T> {
	data: T[];
	total: number;
	page: number;
	perPage: number;
	totalPages: number;
}

/** Paginated users response */
export type PaginatedUsers = PaginatedResponse<User>;

/** Paginated invitations response */
export type PaginatedInvitations = PaginatedResponse<Invitation>;

/** Profile search result */
export interface ProfileSearchResult {
	profiles: PublicProfile[];
	total: number;
	page: number;
	perPage: number;
}

/** Profile completeness score */
export interface CompletenessScore {
	score: number;
	missingFields: string[];
	recommendations: string[];
}

/** Avatar upload response */
export interface AvatarUploadResponse {
	avatarUrl: string;
}

/** Permission check response */
export interface PermissionCheckResponse {
	allowed: boolean;
}

/** Tenant validation response */
export interface TenantValidation {
	valid: boolean;
	tenantId: string;
	tenantName: string;
	userRole: string;
}
