# Task: Password Reset Flow UI

**Task ID:** V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.07_password_reset_flow_ui.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.2_Authentication_UI
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2026-01-17
**Last Updated:** 2026-01-17

## Detailed Description:
Implement the complete password reset flow UI including:
1. "Forgot Password" page where users enter their email
2. Password reset page where users set a new password using the token from email
3. Integration with backend endpoints for the full reset flow

The backend already supports these endpoints:
- `POST /api/v1/auth/forgot-password` - Initiate password reset (timing-safe)
- `POST /api/v1/auth/validate-reset-token` - Validate token before showing form
- `POST /api/v1/auth/reset-password` - Complete password reset

## Acceptance Criteria:
- [ ] Forgot password page accessible at `/forgot-password`
- [ ] User can enter email and request password reset
- [ ] Success message shown regardless of email existence (security: timing-safe)
- [ ] Reset password page accessible at `/reset-password?token=...`
- [ ] Token validated before showing password form
- [ ] Password form with new password and confirm password fields
- [ ] Password strength indicator displayed
- [ ] Success state redirects to login with success message
- [ ] Error states: Invalid token, expired token, password mismatch
- [ ] Rate limiting feedback displayed
- [ ] Responsive design following Frappe UI guidelines
- [ ] Svelte 5 runes used throughout ($state, $derived, $effect)
- [ ] Code compiles without errors: `bun run check`
- [ ] Accessibility requirements met

## Specific Sub-tasks:
- [ ] 1. Create Forgot Password Page (`src/routes/(auth)/forgot-password/+page.svelte`)
    - [ ] 1.1. Set up page with Card component
    - [ ] 1.2. Add email input field
    - [ ] 1.3. Add submit button with loading state
    - [ ] 1.4. Implement API call to `POST /api/v1/auth/forgot-password`
    - [ ] 1.5. Show success message after submission (regardless of email existence)
    - [ ] 1.6. Add "Back to Login" link
    - [ ] 1.7. Handle rate limiting errors

- [ ] 2. Create Reset Password Page (`src/routes/(auth)/reset-password/+page.svelte`)
    - [ ] 2.1. Set up route with query parameter handling for token
    - [ ] 2.2. Create +page.ts to extract and validate token on load
    - [ ] 2.3. Show loading state during token validation
    - [ ] 2.4. Show error if token is invalid/expired
    - [ ] 2.5. Display password reset form if token is valid

- [ ] 3. Implement Password Reset Form
    - [ ] 3.1. Add new password field with visibility toggle
    - [ ] 3.2. Add confirm password field
    - [ ] 3.3. Implement password strength indicator (using zxcvbn)
    - [ ] 3.4. Real-time password match validation
    - [ ] 3.5. Submit button with loading state
    - [ ] 3.6. API call to `POST /api/v1/auth/reset-password`
    - [ ] 3.7. Success state with redirect to login

- [ ] 4. Create Reusable Password Components
    - [ ] 4.1. PasswordStrengthIndicator.svelte component
    - [ ] 4.2. PasswordInput.svelte with visibility toggle
    - [ ] 4.3. Update registration page to use shared components

- [ ] 5. Update Auth API Client
    - [ ] 5.1. Add `forgotPassword(email: string)` method
    - [ ] 5.2. Add `validateResetToken(token: string)` method
    - [ ] 5.3. Add `resetPassword(token: string, newPassword: string)` method
    - [ ] 5.4. Define response types for password reset endpoints

- [ ] 6. Update Login Page
    - [ ] 6.1. Add "Forgot Password?" link to login page
    - [ ] 6.2. Handle success message after password reset (via URL params)

## Dependencies:
*   Task: `task_08.02.01_authentication_pages.md` (Status: Done)
*   Task: `task_08.02.02_form_validation.md` (Status: Done)
*   Task: `task_08.02.03_auth_api_client.md` (Status: Done)

## Files to Create/Modify:
*   `src/routes/(auth)/forgot-password/+page.svelte` - Forgot password page
*   `src/routes/(auth)/reset-password/+page.svelte` - Reset password page
*   `src/routes/(auth)/reset-password/+page.ts` - Token validation on load
*   `src/lib/components/auth/PasswordStrengthIndicator.svelte` - Password strength UI
*   `src/lib/components/auth/PasswordInput.svelte` - Reusable password input
*   `src/lib/api/auth.ts` - Add password reset endpoints
*   `src/routes/(auth)/login/+page.svelte` - Add forgot password link

## Code Examples:
```svelte
<!-- src/routes/(auth)/forgot-password/+page.svelte -->
<script lang="ts">
  import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { authApi } from '$lib/api/auth';
  import { Mail, ArrowLeft, CheckCircle } from 'lucide-svelte';

  let email = $state('');
  let isLoading = $state(false);
  let isSubmitted = $state(false);
  let error = $state('');

  async function handleSubmit(e: Event) {
    e.preventDefault();
    isLoading = true;
    error = '';

    try {
      await authApi.forgotPassword(email);
      isSubmitted = true;
    } catch (err) {
      if (err.code === 'RATE_LIMITED') {
        error = 'Too many requests. Please try again later.';
      } else {
        // Always show success for security (timing-safe)
        isSubmitted = true;
      }
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="min-h-screen flex items-center justify-center p-4">
  <Card class="w-full max-w-md">
    <CardHeader class="text-center">
      <CardTitle>Reset Password</CardTitle>
      <CardDescription>
        Enter your email address and we'll send you a link to reset your password.
      </CardDescription>
    </CardHeader>
    <CardContent>
      {#if isSubmitted}
        <div class="text-center space-y-4">
          <CheckCircle class="w-12 h-12 text-green-500 mx-auto" />
          <p class="text-muted-foreground">
            If an account exists with that email, you'll receive a password reset link shortly.
          </p>
          <Button variant="outline" href="/login" class="w-full">
            <ArrowLeft class="w-4 h-4 mr-2" />
            Back to Login
          </Button>
        </div>
      {:else}
        <form onsubmit={handleSubmit} class="space-y-4">
          <div class="space-y-2">
            <Label for="email">Email</Label>
            <div class="relative">
              <Mail class="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
              <Input
                id="email"
                type="email"
                placeholder="you@example.com"
                bind:value={email}
                class="pl-10"
                required
              />
            </div>
          </div>

          {#if error}
            <p class="text-sm text-red-500">{error}</p>
          {/if}

          <Button type="submit" class="w-full" disabled={isLoading}>
            {#if isLoading}
              Sending...
            {:else}
              Send Reset Link
            {/if}
          </Button>

          <Button variant="ghost" href="/login" class="w-full">
            <ArrowLeft class="w-4 h-4 mr-2" />
            Back to Login
          </Button>
        </form>
      {/if}
    </CardContent>
  </Card>
</div>
```

```svelte
<!-- src/routes/(auth)/reset-password/+page.svelte -->
<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { authApi } from '$lib/api/auth';
  import PasswordInput from '$lib/components/auth/PasswordInput.svelte';
  import PasswordStrengthIndicator from '$lib/components/auth/PasswordStrengthIndicator.svelte';

  let newPassword = $state('');
  let confirmPassword = $state('');
  let isLoading = $state(false);
  let isValidating = $state(true);
  let tokenValid = $state(false);
  let error = $state('');
  let tokenError = $state('');

  const token = $derived($page.url.searchParams.get('token') || '');
  const passwordsMatch = $derived(newPassword === confirmPassword);

  $effect(() => {
    if (token) {
      validateToken();
    } else {
      isValidating = false;
      tokenError = 'No reset token provided';
    }
  });

  async function validateToken() {
    try {
      await authApi.validateResetToken(token);
      tokenValid = true;
    } catch (err) {
      tokenError = err.code === 'TOKEN_EXPIRED' 
        ? 'This reset link has expired. Please request a new one.'
        : 'Invalid reset link.';
    } finally {
      isValidating = false;
    }
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!passwordsMatch) {
      error = 'Passwords do not match';
      return;
    }

    isLoading = true;
    error = '';

    try {
      await authApi.resetPassword(token, newPassword);
      goto('/login?reset=success');
    } catch (err) {
      error = err.message || 'Failed to reset password';
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="min-h-screen flex items-center justify-center p-4">
  {#if isValidating}
    <Card class="w-full max-w-md">
      <CardContent class="py-8 text-center">
        <p>Validating reset link...</p>
      </CardContent>
    </Card>
  {:else if tokenError}
    <Card class="w-full max-w-md">
      <CardContent class="py-8 text-center space-y-4">
        <p class="text-red-500">{tokenError}</p>
        <Button href="/forgot-password">Request New Reset Link</Button>
      </CardContent>
    </Card>
  {:else}
    <Card class="w-full max-w-md">
      <CardHeader>
        <CardTitle>Set New Password</CardTitle>
      </CardHeader>
      <CardContent>
        <form onsubmit={handleSubmit} class="space-y-4">
          <PasswordInput
            label="New Password"
            bind:value={newPassword}
            required
          />
          <PasswordStrengthIndicator password={newPassword} />
          
          <PasswordInput
            label="Confirm Password"
            bind:value={confirmPassword}
            required
          />
          
          {#if confirmPassword && !passwordsMatch}
            <p class="text-sm text-red-500">Passwords do not match</p>
          {/if}

          {#if error}
            <p class="text-sm text-red-500">{error}</p>
          {/if}

          <Button 
            type="submit" 
            class="w-full" 
            disabled={isLoading || !passwordsMatch || !newPassword}
          >
            {isLoading ? 'Resetting...' : 'Reset Password'}
          </Button>
        </form>
      </CardContent>
    </Card>
  {/if}
</div>
```

```typescript
// src/lib/api/auth.ts - additions
export const authApi = {
  // ... existing methods

  async forgotPassword(email: string): Promise<void> {
    const response = await fetch(`${API_BASE}/api/v1/auth/forgot-password`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email }),
    });
    if (!response.ok) {
      throw await parseApiError(response);
    }
  },

  async validateResetToken(token: string): Promise<void> {
    const response = await fetch(`${API_BASE}/api/v1/auth/validate-reset-token`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token }),
    });
    if (!response.ok) {
      throw await parseApiError(response);
    }
  },

  async resetPassword(token: string, newPassword: string): Promise<void> {
    const response = await fetch(`${API_BASE}/api/v1/auth/reset-password`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token, new_password: newPassword }),
    });
    if (!response.ok) {
      throw await parseApiError(response);
    }
  },
};
```

## Testing Steps:
- [ ] Navigate to `/forgot-password` - should show email form
- [ ] Submit with valid email - should show success message
- [ ] Submit with invalid email format - should show validation error
- [ ] Test rate limiting by submitting multiple times quickly
- [ ] Navigate to `/reset-password` without token - should show error
- [ ] Navigate to `/reset-password?token=invalid` - should show invalid token error
- [ ] Navigate to `/reset-password?token=valid` - should show password form
- [ ] Test password strength indicator with various passwords
- [ ] Test password mismatch validation
- [ ] Submit new password - should redirect to login with success message
- [ ] Verify responsive design on mobile
- [ ] Test keyboard navigation

## Backend API Reference:
```
POST /api/v1/auth/forgot-password
Body: { "email": "string" }
Response: 200 OK (always, for timing-safety) | 429 Too Many Requests

POST /api/v1/auth/validate-reset-token
Body: { "token": "string" }
Response: 200 OK | 400 Bad Request | 404 Not Found

POST /api/v1/auth/reset-password
Body: { "token": "string", "new_password": "string" }
Response: 200 OK | 400 Bad Request | 404 Not Found
```

## Notes / Discussion:
---
*   Forgot password response is timing-safe: always returns 200 regardless of email existence
*   Token is passed via URL query parameter from email link
*   Backend uses SHA-256 hashed tokens stored in database
*   Rate limiting: 3 requests per hour per email
*   Password reset tokens expire after 1 hour
*   Password must meet strength requirements (min 8 chars, zxcvbn validation)
*   After reset, all user sessions are invalidated (force logout)

## AI Agent Log:
---
*   2026-01-17 10:15: Task created by Opus
    - Created to address gap in frontend coverage for password reset flow
    - Backend endpoints already implemented in User Service
    - Follows existing auth UI patterns from task_08.02.01
    - Includes security considerations (timing-safe responses)
