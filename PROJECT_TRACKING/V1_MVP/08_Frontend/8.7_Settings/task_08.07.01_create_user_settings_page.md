# Task: Create User Settings and Profile Management Page

**Task ID:** V1_MVP/08_Frontend/8.7_Settings/task_08.07.01_create_user_settings_page.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.7_Settings
**Priority:** Medium
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-01-21
**Last Updated:** 2026-01-17

## Detailed Description:
Create comprehensive user settings page with profile management, preferences, notification settings, and account configuration options. This task covers the "My Profile" section, ensuring users can manage their own data as per `PROFILE_API.md`.

## Specific Sub-tasks:
- [ ] 1. Create User Profile Settings Form
    - [ ] Basic Info: Name, Phone, Title, Department, Location, Bio.
    - [ ] Social Links: Dynamic list or specific fields (LinkedIn, GitHub, etc.).
    - [ ] Avatar: Upload/View (placeholder until S3 is ready).
    - [ ] API: `GET/PUT /api/v1/users/profile`
- [ ] 2. Implement Profile Visibility & Privacy Settings
    - [ ] Visibility Toggle: Public / Team Only / Private.
    - [ ] Field Visibility: Show Email, Show Phone.
    - [ ] API: `PUT /api/v1/users/profile/visibility`
- [ ] 3. Implement Profile Completeness Widget
    - [ ] Fetch score and missing fields from `GET /api/v1/users/profile/completeness`.
    - [ ] Display progress bar and suggestions.
- [ ] 4. Create Notification Preferences Interface
    - [ ] Toggles for Email, Push, SMS.
    - [ ] Granular types: Order Updates, Inventory Alerts, etc.
- [ ] 5. Implement Account Security (Password/2FA) - *Optional/Later if not in MVP scope*
- [ ] 6. Implement Theme/Display Preferences (Language, Timezone, Date Format).

## Acceptance Criteria:
- [ ] User can view and update their extended profile information.
- [ ] Profile completeness score is visible and encourages data entry.
- [ ] Visibility settings are adjustable and persist.
- [ ] Notification preferences are saved correctly.
- [ ] UI provides feedback on save success/failure.
- [ ] Mobile-responsive layout.

## Dependencies:
- V1_MVP/08_Frontend/8.1_Project_Setup/task_08.01.01_setup_sveltekit_project.md (Status: Done)
- V1_MVP/08_Frontend/8.7_Settings/task_08.07.05_user_service_api_client.md (Status: NeedsReview)

## Related Documents:
- `services/user_service/PROFILE_API.md`
- `frontend/src/routes/settings/profile/+page.svelte` (suggested path)

## Notes / Discussion:
---
* Settings should provide comprehensive user control
* Implement proper validation and confirmation for sensitive settings
* Consider GDPR compliance for data settings
* Add proper feedback and success/error messages
* Optimize for both desktop and mobile user experience

## AI Agent Log:
---
*   2026-01-17 14:00: Status reset to Todo by Opus
    - Reviewed existing implementation at `frontend/src/routes/(protected)/settings/+page.svelte`
    - Current implementation is basic placeholder with no User Service API integration
    - Sub-tasks not completed: no profile API calls, no visibility settings, no completeness widget
    - Dependency task_08.07.05 (User Service API Client) is now NeedsReview
    - Task ready to be claimed once dependency is Done