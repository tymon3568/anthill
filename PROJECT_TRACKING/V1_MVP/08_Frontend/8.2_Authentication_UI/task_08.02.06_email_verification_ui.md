# Task: Email Verification Flow UI

**Task ID:** V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.06_email_verification_ui.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.2_Authentication_UI
**Priority:** High
**Status:** Done
**Assignee:** Opus
**Created Date:** 2026-01-17
**Last Updated:** 2026-01-17

## Detailed Description:
Implement the frontend UI for email verification flow. This includes the verification page that handles the token from email links, a success/failure page after verification, and a "resend verification email" functionality. The backend already supports these endpoints:
- `POST /api/v1/auth/verify-email` - Verify email with token
- `POST /api/v1/auth/resend-verification` - Resend verification email (rate limited)

## Acceptance Criteria:
- [x] Verification page accessible at `/verify-email?token=...`
- [x] Page automatically submits token on load and shows verification status
- [x] Success state: Shows confirmation message with link to login
- [x] Error states: Invalid token, expired token, already verified
- [x] Resend verification button available on login/register pages for unverified users
- [x] Rate limiting feedback displayed when resend limit reached
- [x] Responsive design following Frappe UI guidelines
- [x] Svelte 5 runes used throughout ($state, $derived, $effect)
- [x] Code compiles without errors: `bun run check`
- [x] Accessibility requirements met (ARIA attributes, keyboard navigation)

## Specific Sub-tasks:
- [ ] 1. Create Email Verification Page (`src/routes/(auth)/verify-email/+page.svelte`)
    - [ ] 1.1. Set up route with query parameter handling for token
    - [ ] 1.2. Create +page.ts to extract token from URL
    - [ ] 1.3. Implement verification submission on page load
    - [ ] 1.4. Display loading state during verification
    - [ ] 1.5. Show success message with "Go to Login" button
    - [ ] 1.6. Handle error states with appropriate messages

- [ ] 2. Create Verification Status Components
    - [ ] 2.1. Create VerificationSuccess.svelte component
    - [ ] 2.2. Create VerificationError.svelte component (with error type differentiation)
    - [ ] 2.3. Create VerificationLoading.svelte component

- [ ] 3. Implement Resend Verification Functionality
    - [ ] 3.1. Add "Resend Verification Email" button to login page (for unverified users)
    - [ ] 3.2. Create ResendVerificationModal.svelte component
    - [ ] 3.3. Implement API call to `POST /api/v1/auth/resend-verification`
    - [ ] 3.4. Handle rate limiting errors (show remaining time if available)
    - [ ] 3.5. Show success toast after resending

- [ ] 4. Update Auth API Client
    - [ ] 4.1. Add `verifyEmail(token: string)` method
    - [ ] 4.2. Add `resendVerification(email: string)` method
    - [ ] 4.3. Define response types for verification endpoints

- [ ] 5. Add Registration Flow Integration
    - [ ] 5.1. After successful registration, show "Check your email" message
    - [ ] 5.2. Add link to resend verification on registration success page

## Dependencies:
*   Task: `task_08.02.01_authentication_pages.md` (Status: Done)
*   Task: `task_08.02.03_auth_api_client.md` (Status: Done)

## Files to Create/Modify:
*   `src/routes/(auth)/verify-email/+page.svelte` - Email verification page
*   `src/routes/(auth)/verify-email/+page.ts` - Server load function for token extraction
*   `src/lib/components/auth/VerificationSuccess.svelte` - Success state component
*   `src/lib/components/auth/VerificationError.svelte` - Error state component
*   `src/lib/components/auth/VerificationLoading.svelte` - Loading state component
*   `src/lib/components/auth/ResendVerificationModal.svelte` - Resend modal
*   `src/lib/api/auth.ts` - Add verification endpoints
*   `src/routes/(auth)/register/+page.svelte` - Update with verification message

## Code Examples:
```svelte
<!-- src/routes/(auth)/verify-email/+page.svelte -->
<script lang="ts">
  import { page } from '$app/stores';
  import { authApi } from '$lib/api/auth';
  import VerificationSuccess from '$lib/components/auth/VerificationSuccess.svelte';
  import VerificationError from '$lib/components/auth/VerificationError.svelte';
  import VerificationLoading from '$lib/components/auth/VerificationLoading.svelte';

  let status = $state<'loading' | 'success' | 'error'>('loading');
  let errorType = $state<'invalid' | 'expired' | 'already_verified' | 'unknown'>('unknown');
  let errorMessage = $state('');

  $effect(() => {
    const token = $page.url.searchParams.get('token');
    if (token) {
      verifyEmail(token);
    } else {
      status = 'error';
      errorType = 'invalid';
      errorMessage = 'No verification token provided';
    }
  });

  async function verifyEmail(token: string) {
    try {
      await authApi.verifyEmail(token);
      status = 'success';
    } catch (error) {
      status = 'error';
      // Handle different error types based on API response
      if (error.code === 'TOKEN_EXPIRED') {
        errorType = 'expired';
        errorMessage = 'Verification link has expired. Please request a new one.';
      } else if (error.code === 'ALREADY_VERIFIED') {
        errorType = 'already_verified';
        errorMessage = 'Your email is already verified. You can log in.';
      } else {
        errorType = 'invalid';
        errorMessage = 'Invalid verification link.';
      }
    }
  }
</script>

{#if status === 'loading'}
  <VerificationLoading />
{:else if status === 'success'}
  <VerificationSuccess />
{:else}
  <VerificationError {errorType} {errorMessage} />
{/if}
```

```typescript
// src/lib/api/auth.ts - additions
export interface VerifyEmailRequest {
  token: string;
}

export interface ResendVerificationRequest {
  email: string;
}

export const authApi = {
  // ... existing methods
  
  async verifyEmail(token: string): Promise<void> {
    const response = await fetch(`${API_BASE}/api/v1/auth/verify-email`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token }),
    });
    if (!response.ok) {
      throw await parseApiError(response);
    }
  },

  async resendVerification(email: string): Promise<void> {
    const response = await fetch(`${API_BASE}/api/v1/auth/resend-verification`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email }),
    });
    if (!response.ok) {
      throw await parseApiError(response);
    }
  },
};
```

## Testing Steps:
- [ ] Navigate to `/verify-email` without token - should show error
- [ ] Navigate to `/verify-email?token=invalid` - should show invalid token error
- [ ] Navigate to `/verify-email?token=valid` (from test email) - should show success
- [ ] Test resend verification from login page
- [ ] Test rate limiting by clicking resend multiple times
- [ ] Verify responsive design on mobile
- [ ] Test keyboard navigation and screen reader accessibility

## Backend API Reference:
```
POST /api/v1/auth/verify-email
Body: { "token": "string" }
Response: 200 OK | 400 Bad Request | 404 Not Found

POST /api/v1/auth/resend-verification
Body: { "email": "string" }
Response: 200 OK | 429 Too Many Requests
```

## Notes / Discussion:
---
*   Token is passed via URL query parameter from email link
*   Backend uses SHA-256 hashed tokens stored in database
*   Rate limiting: 3 resend requests per hour per email
*   Verification tokens expire after 24 hours
*   After verification, user should be redirected to login page

## AI Agent Log:
---
*   2026-01-17 10:00: Task created by Opus
    - Created to address gap in frontend coverage for email verification flow
    - Backend endpoints already implemented in User Service
    - Follows existing auth UI patterns from task_08.02.01
*   2026-01-17 19:00: Task completed by Opus
    - Added verifyEmail and resendVerification methods to auth API client
    - Created email verification page at /verify-email with:
      - Automatic token verification on page load
      - Loading, success, and error states
      - Error handling for invalid/expired/already verified tokens
    - Updated registration page with 'Check your email' success state
    - Added resend verification functionality with rate limiting feedback
    - PR #165 merged successfully
