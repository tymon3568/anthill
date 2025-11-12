import { json, error } from '@sveltejs/kit';
import { validateAndParseToken } from '$lib/auth/jwt';
import { createAuthError, AuthErrorCode } from '$lib/auth/errors';
import type { RequestHandler } from './$types';
import type { Cookies } from '@sveltejs/kit';

interface UserProfile {
	id: string;
	email: string;
	username: string;
	display_name?: string;
	tenant_id: string;
	roles: string[];
	permissions: string[];
	created_at: string;
	updated_at: string;
}

// Mock user profile data - in production this would come from database
const mockUserProfile: UserProfile = {
	id: 'user-123',
	email: 'user@example.com',
	username: 'johndoe',
	display_name: 'John Doe',
	tenant_id: 'tenant-456',
	roles: ['user', 'admin'],
	permissions: ['read:products', 'write:products', 'read:orders'],
	created_at: '2025-01-01T00:00:00Z',
	updated_at: '2025-01-01T00:00:00Z'
};

// GET /api/v1/users/profile - Get current user profile (matches backend API spec)
export const GET: RequestHandler = async ({ cookies }) => {
	try {
		// Get access token from cookie
		const accessToken = cookies.get('access_token');
		if (!accessToken) {
			throw error(401, JSON.stringify(createAuthError(AuthErrorCode.NO_SESSION)));
		}

		// Validate token and extract user info
		const userInfo = await validateAndParseToken(accessToken);
		if (!userInfo) {
			throw error(401, JSON.stringify(createAuthError(AuthErrorCode.INVALID_TOKEN)));
		}

		// In production, fetch user profile from database using userInfo.userId
		// For now, return mock data with user info from token
		const profile: UserProfile = {
			...mockUserProfile,
			id: userInfo.userId,
			email: userInfo.email,
			username: userInfo.name || userInfo.email.split('@')[0],
			tenant_id: userInfo.tenantId || 'default-tenant',
			roles: userInfo.groups || [],
			permissions: [] // Would be calculated based on roles
		};

		return json({
			success: true,
			data: profile
		});
	} catch (err) {
		console.error('Get profile error:', err);

		// If it's already a SvelteKit error, re-throw it to preserve status code
		if (err && typeof err === 'object' && 'status' in err) {
			throw err;
		}

		const authError =
			err instanceof Error && 'code' in err
				? (err as any)
				: createAuthError(AuthErrorCode.PROFILE_FETCH_FAILED);
		throw error(authError.statusCode || 500, JSON.stringify(authError));
	}
};

// PUT /api/v1/auth/profile - Update user profile
export const PUT: RequestHandler = async ({ request, cookies }) => {
	try {
		// Get access token from cookie
		const accessToken = cookies.get('access_token');
		if (!accessToken) {
			throw error(401, JSON.stringify(createAuthError(AuthErrorCode.NO_SESSION)));
		}

		// Validate token
		const userInfo = await validateAndParseToken(accessToken);
		if (!userInfo) {
			throw error(401, JSON.stringify(createAuthError(AuthErrorCode.INVALID_TOKEN)));
		}

		// Parse request body
		const updateData = await request.json();

		// Validate update data (basic validation)
		const allowedFields = ['display_name', 'username'];
		const filteredData: Record<string, any> = {};

		for (const field of allowedFields) {
			if (updateData[field] !== undefined) {
				filteredData[field] = updateData[field];
			}
		}

		// In production, update user profile in database
		// For now, return updated mock data
		const updatedProfile: UserProfile = {
			...mockUserProfile,
			...filteredData,
			id: userInfo.userId,
			email: userInfo.email,
			updated_at: new Date().toISOString()
		};

		return json({
			success: true,
			data: updatedProfile
		});
	} catch (err) {
		console.error('Update profile error:', err);

		// If it's already a SvelteKit error, re-throw it to preserve status code
		if (err && typeof err === 'object' && 'status' in err) {
			throw err;
		}

		const authError =
			err instanceof Error && 'code' in err
				? (err as any)
				: createAuthError(AuthErrorCode.PROFILE_UPDATE_FAILED);
		throw error(authError.statusCode || 500, JSON.stringify(authError));
	}
};
