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

// ============ Tenant Settings Types ============

/** Tenant subscription plan */
export type TenantPlan = 'free' | 'starter' | 'professional' | 'enterprise';

/** Tenant status */
export type TenantStatus = 'active' | 'suspended' | 'pending_deletion';

/** Tenant information */
export interface Tenant {
	tenantId: string;
	name: string;
	slug: string;
	ownerUserId: string;
	plan: TenantPlan;
	status: TenantStatus;
	createdAt: string;
	updatedAt?: string;
}

/** Tenant branding settings */
export interface TenantBranding {
	logoUrl?: string;
	faviconUrl?: string;
	primaryColor?: string;
	secondaryColor?: string;
	accentColor?: string;
}

/** Tenant localization settings */
export interface TenantLocalization {
	defaultTimezone: string;
	defaultCurrency: string;
	defaultLanguage: string;
	dateFormat: string;
	timeFormat: '12h' | '24h';
}

/** Tenant security policy */
export interface TenantSecurityPolicy {
	passwordMinLength: number;
	passwordRequireUppercase: boolean;
	passwordRequireLowercase: boolean;
	passwordRequireNumbers: boolean;
	passwordRequireSpecialChars: boolean;
	sessionTimeoutMinutes: number;
	maxLoginAttempts: number;
	lockoutDurationMinutes: number;
	mfaRequired: boolean;
}

/** Tenant data retention settings */
export interface TenantDataRetention {
	auditLogRetentionDays: number;
	deletedUserRetentionDays: number;
	sessionHistoryRetentionDays: number;
	backupEnabled: boolean;
	backupFrequency: 'daily' | 'weekly' | 'monthly';
}

/** Tenant billing information */
export interface TenantBilling {
	plan: TenantPlan;
	billingEmail?: string;
	billingAddress?: string;
	paymentMethod?: string;
	nextBillingDate?: string;
	currentPeriodEnd?: string;
	usageStats?: {
		usersCount: number;
		usersLimit: number;
		storageUsedMb: number;
		storageLimitMb: number;
		apiCallsCount: number;
		apiCallsLimit: number;
	};
}

/** Tenant integration settings (webhooks, API keys) */
export interface TenantIntegrationSettings {
	webhookEndpoints: WebhookEndpoint[];
	apiKeys: ApiKey[];
}

/** Webhook endpoint */
export interface WebhookEndpoint {
	id: string;
	url: string;
	events: string[];
	active: boolean;
	secret?: string;
	createdAt: string;
}

/** API key */
export interface ApiKey {
	id: string;
	name: string;
	keyPrefix: string;
	scopes: string[];
	lastUsedAt?: string;
	expiresAt?: string;
	createdAt: string;
}

/** Audit log entry */
export interface AuditLogEntry {
	id: string;
	userId: string;
	userEmail: string;
	action: string;
	resource: string;
	resourceId?: string;
	details?: Record<string, unknown>;
	ipAddress?: string;
	userAgent?: string;
	createdAt: string;
}

/** Paginated audit logs */
export type PaginatedAuditLogs = PaginatedResponse<AuditLogEntry>;

/** Tenant usage analytics */
export interface TenantAnalytics {
	activeUsersLast30Days: number;
	totalUsers: number;
	totalOrders: number;
	totalProducts: number;
	storageUsedMb: number;
	apiCallsLast30Days: number;
	loginCountLast30Days: number;
	topActiveUsers: Array<{
		userId: string;
		email: string;
		actionsCount: number;
	}>;
}

/** Full tenant settings */
export interface TenantSettings {
	tenant: Tenant;
	branding: TenantBranding;
	localization: TenantLocalization;
	securityPolicy: TenantSecurityPolicy;
	dataRetention: TenantDataRetention;
}

/** Request to update tenant info */
export interface UpdateTenantRequest {
	name?: string;
	contactEmail?: string;
	contactPhone?: string;
	address?: string;
}

/** Request to update tenant branding */
export interface UpdateBrandingRequest {
	logoUrl?: string;
	faviconUrl?: string;
	primaryColor?: string;
	secondaryColor?: string;
	accentColor?: string;
}

/** Request to update localization */
export interface UpdateLocalizationRequest {
	defaultTimezone?: string;
	defaultCurrency?: string;
	defaultLanguage?: string;
	dateFormat?: string;
	timeFormat?: '12h' | '24h';
}

/** Request to update security policy */
export interface UpdateSecurityPolicyRequest {
	passwordMinLength?: number;
	passwordRequireUppercase?: boolean;
	passwordRequireLowercase?: boolean;
	passwordRequireNumbers?: boolean;
	passwordRequireSpecialChars?: boolean;
	sessionTimeoutMinutes?: number;
	maxLoginAttempts?: number;
	lockoutDurationMinutes?: number;
	mfaRequired?: boolean;
}

/** Request to update data retention */
export interface UpdateDataRetentionRequest {
	auditLogRetentionDays?: number;
	deletedUserRetentionDays?: number;
	sessionHistoryRetentionDays?: number;
	backupEnabled?: boolean;
	backupFrequency?: 'daily' | 'weekly' | 'monthly';
}

/** Audit log filter params */
export interface ListAuditLogsParams {
	page?: number;
	perPage?: number;
	userId?: string;
	action?: string;
	startDate?: string;
	endDate?: string;
}

/** Tenant export request */
export interface TenantExportRequest {
	format: 'json' | 'csv';
	includeUsers?: boolean;
	includeAuditLogs?: boolean;
	includeSettings?: boolean;
}

/** Delete tenant confirmation */
export interface DeleteTenantRequest {
	confirmTenantName: string;
	reason?: string;
}
