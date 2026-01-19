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

/**
 * User status in the system
 *
 * Lifecycle:
 * - 'active': User can login and use the system normally
 * - 'suspended': User is temporarily blocked by admin (can be unsuspended)
 * - 'inactive': User has been soft-deleted (deleted_at is set, should not appear in normal lists)
 *
 * Note: Users with 'inactive' status also have deleted_at set, so they are filtered out
 * by the backend list API (WHERE deleted_at IS NULL). If you see 'inactive' users,
 * it may indicate legacy data.
 */
export type UserStatus = 'active' | 'suspended' | 'inactive';

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

/** Role definition - matches backend RoleInfo struct */
export interface Role {
	role_name: string;
	description?: string;
	permissions?: Permission[]; // May be undefined for system roles without explicit policies
	user_count: number;
}

/** Response from list roles endpoint */
export interface RoleListResponse {
	roles: Role[];
	total: number;
}

/** Available permission from backend */
export interface AvailablePermission {
	resource: string;
	actions: string[];
	description: string;
}

/** Response from list permissions endpoint */
export interface PermissionListResponse {
	permissions: AvailablePermission[];
	total: number;
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
	role_name: string;
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

/** User roles response from getUserRoles endpoint */
export interface UserRolesResponse {
	roles: string[];
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

// =====================================
// Payment Gateway Types
// =====================================

/** Payment gateway provider */
export type PaymentProvider =
	| 'stripe'
	| 'paypal'
	| 'square'
	| 'braintree'
	| 'adyen'
	| 'momo'
	| 'vnpay'
	| 'zalopay';

/** Payment gateway status */
export type PaymentGatewayStatus = 'active' | 'inactive' | 'error' | 'pending_setup';

/** Payment method type */
export type PaymentMethodType =
	| 'credit_card'
	| 'debit_card'
	| 'bank_transfer'
	| 'digital_wallet'
	| 'buy_now_pay_later'
	| 'crypto';

/** Payment gateway configuration */
export interface PaymentGateway {
	id: string;
	provider: PaymentProvider;
	name: string;
	status: PaymentGatewayStatus;
	isDefault: boolean;
	isSandbox: boolean;
	supportedMethods: PaymentMethodType[];
	supportedCurrencies: string[];
	supportedRegions: string[];
	createdAt: string;
	updatedAt?: string;
}

/** Payment gateway credentials (masked) */
export interface PaymentGatewayCredentials {
	gatewayId: string;
	provider: PaymentProvider;
	publicKey?: string;
	secretKeyMasked?: string;
	merchantId?: string;
	webhookSecret?: string;
	additionalConfig?: Record<string, unknown>;
	lastVerifiedAt?: string;
}

/** Webhook endpoint configuration */
export interface PaymentWebhookConfig {
	id: string;
	gatewayId: string;
	url: string;
	events: string[];
	isActive: boolean;
	signingSecret?: string;
	lastReceivedAt?: string;
	failureCount: number;
	createdAt: string;
}

/** Payment method settings */
export interface PaymentMethodSettings {
	type: PaymentMethodType;
	enabled: boolean;
	displayName: string;
	minAmount?: number;
	maxAmount?: number;
	currencies?: string[];
	regions?: string[];
}

/** Currency configuration */
export interface CurrencyConfig {
	code: string;
	name: string;
	symbol: string;
	enabled: boolean;
	isDefault: boolean;
	exchangeRate?: number;
	decimalPlaces: number;
}

/** Region configuration for payments */
export interface PaymentRegionConfig {
	code: string;
	name: string;
	enabled: boolean;
	currencies: string[];
	taxRate?: number;
	requiresAddressVerification: boolean;
}

/** Transaction fee configuration */
export interface TransactionFeeConfig {
	gatewayId: string;
	provider: PaymentProvider;
	fixedFee: number;
	percentageFee: number;
	currency: string;
	appliesTo: PaymentMethodType[];
}

/** Settlement configuration */
export interface SettlementConfig {
	gatewayId: string;
	frequency: 'daily' | 'weekly' | 'monthly';
	minimumAmount: number;
	currency: string;
	bankAccountMasked?: string;
	autoSettlement: boolean;
}

/** Payment gateway health status */
export interface PaymentGatewayHealth {
	gatewayId: string;
	provider: PaymentProvider;
	status: 'healthy' | 'degraded' | 'down';
	latencyMs?: number;
	successRate?: number;
	lastCheckedAt: string;
	recentErrors?: Array<{
		timestamp: string;
		errorCode: string;
		message: string;
	}>;
}

/** Payment analytics data */
export interface PaymentAnalytics {
	gatewayId: string;
	period: 'day' | 'week' | 'month';
	totalTransactions: number;
	successfulTransactions: number;
	failedTransactions: number;
	totalVolume: number;
	averageTransactionValue: number;
	currency: string;
	topPaymentMethods: Array<{
		method: PaymentMethodType;
		count: number;
		volume: number;
	}>;
	dailyVolume: Array<{
		date: string;
		volume: number;
		count: number;
	}>;
}

/** Payment security settings */
export interface PaymentSecuritySettings {
	require3DSecure: boolean;
	fraudDetectionEnabled: boolean;
	fraudRiskThreshold: 'low' | 'medium' | 'high';
	velocityChecksEnabled: boolean;
	maxTransactionsPerHour?: number;
	maxAmountPerDay?: number;
	blockedCountries: string[];
	blockedBinRanges: string[];
}

/** Full payment settings */
export interface PaymentSettings {
	gateways: PaymentGateway[];
	paymentMethods: PaymentMethodSettings[];
	currencies: CurrencyConfig[];
	regions: PaymentRegionConfig[];
	security: PaymentSecuritySettings;
}

/** Request to add/update payment gateway */
export interface UpsertPaymentGatewayRequest {
	provider: PaymentProvider;
	name: string;
	isSandbox: boolean;
	isDefault?: boolean;
	publicKey?: string;
	secretKey?: string;
	merchantId?: string;
	webhookSecret?: string;
	additionalConfig?: Record<string, unknown>;
}

/** Request to update payment methods */
export interface UpdatePaymentMethodsRequest {
	methods: PaymentMethodSettings[];
}

/** Request to update currencies */
export interface UpdateCurrenciesRequest {
	currencies: CurrencyConfig[];
}

/** Request to update regions */
export interface UpdateRegionsRequest {
	regions: PaymentRegionConfig[];
}

/** Request to update payment security */
export interface UpdatePaymentSecurityRequest {
	require3DSecure?: boolean;
	fraudDetectionEnabled?: boolean;
	fraudRiskThreshold?: 'low' | 'medium' | 'high';
	velocityChecksEnabled?: boolean;
	maxTransactionsPerHour?: number;
	maxAmountPerDay?: number;
	blockedCountries?: string[];
	blockedBinRanges?: string[];
}

/** Test payment result */
export interface TestPaymentResult {
	success: boolean;
	transactionId?: string;
	errorCode?: string;
	errorMessage?: string;
	latencyMs: number;
}
