/**
 * API Client Exports
 *
 * Central export point for all service API clients.
 */

// Base client
export { apiClient, createPaginationParams } from './client';

// Authentication API
export { authApi } from './auth';
export type {
	EmailLoginRequest,
	EmailRegisterRequest,
	EmailAuthResponse,
	EmailUserInfo,
	UserProfile as AuthUserProfile,
	SessionInfo
} from './auth';

// User Service API
export { userServiceApi, UserServiceApiError } from './user-service';
export type * from './types/user-service.types';

// Inventory API
export { inventoryApi } from './inventory';

// Dashboard API
export { dashboardApi } from './dashboard';

// Products API
export { productsApi } from './products';

// Orders API
export { ordersApi } from './orders';

// Integrations API
export { integrationsApi } from './integrations';
