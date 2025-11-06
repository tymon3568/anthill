import { json, error, type RequestHandler } from '@sveltejs/kit';
import { authApi } from '$lib/api/auth';
import type { UserProfile } from '$lib/api/auth';

// GET /api/v1/users/preferences - Get user preferences
export const GET: RequestHandler = async ({ locals, cookies }) => {
	try {
		// Get the full profile which includes preferences
		const profileResponse = await authApi.getProfile();

		if (!profileResponse.success || !profileResponse.data) {
			throw error(401, 'Unable to retrieve user profile');
		}

		const profile = profileResponse.data;

		// Extract preferences from profile
		const preferences = {
			language: profile.language || 'en',
			timezone: profile.timezone || 'UTC',
			date_format: profile.date_format || 'YYYY-MM-DD',
			time_format: profile.time_format || '24h',
			notification_preferences: profile.notification_preferences || {
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
			profile_visibility: profile.profile_visibility || 'private',
			show_email: profile.show_email || false,
			show_phone: profile.show_phone || false
		};

		return json(preferences);
	} catch (err) {
		console.error('Failed to get user preferences:', err);
		throw error(500, 'Failed to retrieve user preferences');
	}
};

// PUT /api/v1/users/preferences - Update user preferences
export const PUT: RequestHandler = async ({ request, locals, cookies }) => {
	try {
		const preferencesUpdate = await request.json();

		// Validate preferences data
		const allowedFields = [
			'language', 'timezone', 'date_format', 'time_format',
			'notification_preferences', 'profile_visibility', 'show_email', 'show_phone'
		];

		const filteredData: Partial<UserProfile> = {};
		for (const [key, value] of Object.entries(preferencesUpdate)) {
			if (allowedFields.includes(key)) {
				(filteredData as any)[key] = value;
			}
		}

		// Update the profile with preferences
		const updateResponse = await authApi.updateProfile(filteredData);

		if (!updateResponse.success || !updateResponse.data) {
			throw error(400, 'Failed to update preferences');
		}

		const updatedProfile = updateResponse.data;

		// Return only preferences from the updated profile
		const preferences = {
			language: updatedProfile.language || 'en',
			timezone: updatedProfile.timezone || 'UTC',
			date_format: updatedProfile.date_format || 'YYYY-MM-DD',
			time_format: updatedProfile.time_format || '24h',
			notification_preferences: updatedProfile.notification_preferences || {
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
			profile_visibility: updatedProfile.profile_visibility || 'private',
			show_email: updatedProfile.show_email || false,
			show_phone: updatedProfile.show_phone || false
		};

		return json(preferences);
	} catch (err) {
		console.error('Failed to update user preferences:', err);
		if (err instanceof Error && err.message.includes('validation')) {
			throw error(400, err.message);
		}
		throw error(500, 'Failed to update user preferences');
	}
};
