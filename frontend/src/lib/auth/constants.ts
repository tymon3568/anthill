/**
 * Auth-related constants
 * Centralized to avoid magic strings and ensure consistency
 */

// Cookie names
export const COOKIE_SESSION_INVALIDATED = 'session_invalidated';
export const COOKIE_USER_DATA = 'user_data';
export const COOKIE_ACCESS_TOKEN = 'access_token';
export const COOKIE_REFRESH_TOKEN = 'refresh_token';

// localStorage keys
export const STORAGE_USER_DATA = 'user_data';
export const STORAGE_TENANT_SLUG = 'anthill_tenant_slug';

// Error codes that indicate permanent session invalidation
export const PERMANENT_ERROR_CODES = [
	'USER_NOT_FOUND',
	'SESSION_REVOKED',
	'TENANT_NOT_FOUND'
] as const;
