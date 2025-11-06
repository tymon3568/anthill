import { json, error } from '@sveltejs/kit';
import { validateAndParseToken } from '$lib/auth/jwt';
import { createAuthError, AuthErrorCode } from '$lib/auth/errors';
import type { RequestHandler } from './$types';
import type { Cookies } from '@sveltejs/kit';

interface UserProfile {
	id: string;
	email: string;
	full_name?: string;
	tenant_id: string;
	role: string;
	avatar_url?: string;
	phone?: string;
	bio?: string;
	title?: string;
	department?: string;
	location?: string;
	website_url?: string;
	social_links?: Record<string, string>;
	language?: string;
	timezone?: string;
	date_format?: string;
	time_format?: string;
	notification_preferences?: {
		email_notifications?: boolean;
		push_notifications?: boolean;
		sms_notifications?: boolean;
		notification_types?: Record<string, boolean>;
	};
	profile_visibility?: string;
	show_email?: boolean;
	show_phone?: boolean;
	completeness_score?: number;
	verified?: boolean;
	verification_badge?: string;
	custom_fields?: Record<string, any>;
	created_at: string;
	updated_at: string;
}

// Mock user profile data - in production this would come from database
const mockUserProfile: UserProfile = {
	id: 'user-123',
	email: 'user@example.com',
	full_name: 'John Doe',
	tenant_id: 'tenant-456',
	role: 'user',
	avatar_url: 'https://example.com/avatar.jpg',
	phone: '+1234567890',
	bio: 'Software engineer passionate about building great products',
	title: 'Senior Developer',
	department: 'Engineering',
	location: 'San Francisco, CA',
	website_url: 'https://johndoe.com',
	social_links: {
		linkedin: 'https://linkedin.com/in/johndoe',
		github: 'https://github.com/johndoe'
	},
	language: 'en',
	timezone: 'America/Los_Angeles',
	date_format: 'YYYY-MM-DD',
	time_format: '24h',
	notification_preferences: {
		email_notifications: true,
		push_notifications: false,
		sms_notifications: false,
		notification_types: {
			order_updates: true,
			inventory_alerts: true,
			system_announcements: true,
			security_alerts: true,
			marketing_emails: false
		}
	},
	profile_visibility: 'private',
	show_email: false,
	show_phone: false,
	completeness_score: 85,
	verified: false,
	verification_badge: undefined,
	custom_fields: {},
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
			full_name: userInfo.name,
			tenant_id: userInfo.tenantId || 'default-tenant',
			// Keep other mock data for now
		};

		return json(profile);

	} catch (err) {
		console.error('Get profile error:', err);

		const authError = err instanceof Error && 'code' in err
			? err as any
			: createAuthError(AuthErrorCode.PROFILE_FETCH_FAILED);
		throw error(authError.statusCode || 500, JSON.stringify(authError));
	}
};

// PUT /api/v1/users/profile - Update user profile (matches backend API spec)
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
		const allowedFields = [
			'full_name', 'phone', 'bio', 'title', 'department', 'location',
			'website_url', 'social_links', 'language', 'timezone', 'date_format', 'time_format',
			'notification_preferences', 'profile_visibility', 'show_email', 'show_phone', 'custom_fields'
		];
		const filteredData: Partial<UserProfile> = {};

		for (const field of allowedFields) {
			if (updateData[field] !== undefined) {
				(filteredData as any)[field] = updateData[field];
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

		return json(updatedProfile);

	} catch (err) {
		console.error('Update profile error:', err);

		const authError = err instanceof Error && 'code' in err
			? err as any
			: createAuthError(AuthErrorCode.PROFILE_UPDATE_FAILED);
		throw error(authError.statusCode || 500, JSON.stringify(authError));
	}
};
