# Task: Implement Email Verification UI

**Task ID:** V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.07_email_verification_ui.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.2_Authentication_UI
**Priority:** Medium
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-01-26
**Last Updated:** 2025-01-26

## Detailed Description:
Create the UI for the email verification flow. This includes a "Verify Email" landing page (that consumes the token) and a "Resend Verification Email" feature for users who haven't verified yet.

## Specific Sub-tasks:
- [ ] 1. Create Verification Landing Page (`src/routes/verify-email/+page.svelte`)
    - [ ] Extract `token` from URL.
    - [ ] Auto-submit verification request on mount (or show button).
    - [ ] API integration: `POST /api/v1/auth/verify-email`.
    - [ ] Success: "Email verified! Redirecting to dashboard...".
    - [ ] Error: "Verification failed or expired." + Link to Resend.
- [ ] 2. Create "Unverified User" Banner/Guard
    - [ ] Detect if logged-in user is unverified (`user.email_verified === false`).
    - [ ] Show global banner: "Please verify your email."
- [ ] 3. Create Resend Verification Component
    - [ ] Button/Link to request new token.
    - [ ] API integration: `POST /api/v1/auth/resend-verification`.

## Acceptance Criteria:
- [ ] Users clicking the email link are verified and redirected.
- [ ] Unverified users are reminded to verify.
- [ ] Users can request a new verification email.

## Dependencies:
- V1_MVP/03_User_Service/3.6_Self_Auth_Enhancements/task_03.06.01_email_verification_flow.md (Backend)

## Related Documents:
- `services/user_service/api/src/auth_handlers.rs`

## Notes / Discussion:
---
* The backend might restrict certain actions until email is verified. The frontend should reflect this state.

## AI Agent Log:
---
