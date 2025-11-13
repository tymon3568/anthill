# Task: Session Management and Routing

**Task ID:** V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.04_session_management.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.2_Authentication_UI
**Priority:** High
**Status:** Todo
**Assignee:**
**Created Date:** 2025-11-12
**Last Updated:** 2025-11-12

## Detailed Description:
Implement session management, protected routing, and logout functionality. Create an authentication store using Svelte 5 runes to manage user state across the application, handle route protection, and provide seamless login/logout flows.

## Acceptance Criteria:
- [ ] Authentication store using Svelte 5 runes ($state)
- [ ] Login form integration with API client
- [ ] Registration form integration with API client
- [ ] Protected route guards for authenticated pages
- [ ] Automatic redirect after successful login/registration
- [ ] Logout functionality with proper cleanup
- [ ] Session persistence across page reloads
- [ ] Loading states during authentication checks
- [ ] Proper error handling and user feedback
- [ ] Code compiles without errors: `bun run check`
- [ ] Authentication flow works end-to-end

## Specific Sub-tasks:
- [ ] 1. Create authentication store
    - [ ] 1.1. Implement auth store with Svelte 5 runes (`src/lib/stores/auth.ts`)
    - [ ] 1.2. Manage user state (authenticated/unauthenticated)
    - [ ] 1.3. Handle token storage and retrieval
    - [ ] 1.4. Provide login/logout methods

- [ ] 2. Integrate forms with authentication
    - [ ] 2.1. Update login page to use auth store and API client
    - [ ] 2.2. Update registration page to use auth store and API client
    - [ ] 2.3. Handle form submission and error display
    - [ ] 2.4. Implement success redirects

- [ ] 3. Implement route protection
    - [ ] 3.1. Create route guard component or hook
    - [ ] 3.2. Protect authenticated routes (dashboard, etc.)
    - [ ] 3.3. Redirect unauthenticated users to login
    - [ ] 3.4. Handle loading states during auth checks

- [ ] 4. Implement logout functionality
    - [ ] 4.1. Add logout button to protected pages
    - [ ] 4.2. Clear authentication state and tokens
    - [ ] 4.3. Redirect to login page after logout
    - [ ] 4.4. Handle logout from multiple tabs/windows

## Dependencies:
*   Task: `task_08.02.01_authentication_pages.md` (Status: Done)
*   Task: `task_08.02.02_form_validation.md` (Status: Done)
*   Task: `task_08.02.03_auth_api_client.md` (Status: Done)

## Files to Create/Modify:
*   `src/lib/stores/auth.ts` - Authentication state management
*   `src/routes/login/+page.svelte` - Integrate with auth store
*   `src/routes/register/+page.svelte` - Integrate with auth store
*   `src/lib/components/AuthGuard.svelte` - Route protection component
*   `src/hooks.client.ts` - Client-side authentication checks
*   `src/app.html` - Update for authentication state

## Code Examples:
```typescript
// src/lib/stores/auth.ts
import { login, register, logout as apiLogout, getStoredToken, isAuthenticated } from '$lib/api/auth/client';
import type { LoginRequest, RegisterRequest } from '$lib/api/auth/types';

interface User {
  id: string;
  email: string;
  tenant_id: string;
}

interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
}

class AuthStore {
  state = $state<AuthState>({
    user: null,
    isAuthenticated: false,
    isLoading: false,
    error: null
  });

  constructor() {
    // Check for existing session on initialization
    this.checkAuthStatus();
  }

  async checkAuthStatus() {
    this.state.isLoading = true;
    try {
      if (isAuthenticated()) {
        // In a real app, you might validate the token with the server
        // For now, assume stored token is valid
        this.state.isAuthenticated = true;
        // You could decode JWT to get user info, but for simplicity:
        this.state.user = { id: 'temp', email: 'temp@example.com', tenant_id: 'temp' };
      }
    } catch (error) {
      this.logout();
    } finally {
      this.state.isLoading = false;
    }
  }

  async login(credentials: LoginRequest) {
    this.state.isLoading = true;
    this.state.error = null;

    try {
      const response = await login(credentials);
      this.state.user = response.user;
      this.state.isAuthenticated = true;

      // Redirect will be handled by the component
    } catch (error) {
      this.state.error = error.message;
      throw error;
    } finally {
      this.state.isLoading = false;
    }
  }

  async register(userData: RegisterRequest) {
    this.state.isLoading = true;
    this.state.error = null;

    try {
      const response = await register(userData);
      this.state.user = response.user;
      this.state.isAuthenticated = true;

      // Redirect will be handled by the component
    } catch (error) {
      this.state.error = error.message;
      throw error;
    } finally {
      this.state.isLoading = false;
    }
  }

  logout() {
    apiLogout();
    this.state.user = null;
    this.state.isAuthenticated = false;
    this.state.error = null;
  }
}

export const authStore = new AuthStore();
```

```svelte
<!-- src/lib/components/AuthGuard.svelte -->
<script>
  import { authStore } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  let { children } = $props();

  onMount(() => {
    if (!authStore.state.isAuthenticated && !authStore.state.isLoading) {
      goto('/login');
    }
  });

  $effect(() => {
    if (!authStore.state.isAuthenticated && !authStore.state.isLoading) {
      goto('/login');
    }
  });
</script>

{#if authStore.state.isLoading}
  <div class="flex items-center justify-center min-h-screen">
    <div class="text-gray-600">Loading...</div>
  </div>
{:else if authStore.state.isAuthenticated}
  {@render children()}
{/if}
```

```svelte
<!-- Integration in login page -->
<script>
  import { authStore } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import { loginSchema, type LoginData } from '$lib/validation/auth';
  import * as v from 'valibot';

  let formData = $state<LoginData>({ email: '', password: '' });
  let errors = $state<Record<string, string>>({});

  // Redirect if already authenticated
  $effect(() => {
    if (authStore.state.isAuthenticated) {
      goto('/dashboard');
    }
  });

  function validateForm(): boolean {
    try {
      v.parse(loginSchema, formData);
      errors = {};
      return true;
    } catch (error) {
      if (error instanceof v.ValiError) {
        errors = {};
        for (const issue of error.issues) {
          const field = issue.path?.[0]?.key as string;
          if (field) {
            errors[field] = issue.message;
          }
        }
      }
      return false;
    }
  }

  async function handleSubmit() {
    if (!validateForm()) return;

    try {
      await authStore.login(formData);
      goto('/dashboard');
    } catch (error) {
      // Error is handled by authStore
    }
  }
</script>
```

## Testing Steps:
- [ ] Test complete login flow (form → API → redirect)
- [ ] Test complete registration flow
- [ ] Test logout functionality
- [ ] Test protected route access control
- [ ] Test session persistence across page reloads
- [ ] Test authentication state management
- [ ] Verify error handling in all scenarios

## References:
*   SvelteKit routing documentation
*   `frontend/.svelte-instructions.md` - Svelte 5 runes guidelines
*   Project state management patterns

## Notes / Discussion:
---
*   Use Svelte 5 runes for all state management
*   Consider security implications of client-side session storage
*   Handle edge cases like expired tokens
*   Ensure smooth user experience with loading states

## AI Agent Log:
---
*   2025-11-12 10:45: Task created by Claude
    - Set up comprehensive session management system
    - Included route protection and auth store
    - Integrated with previous tasks
    - Ready for implementation</content>
<parameter name="filePath">/home/arch/Project/test/anthill/PROJECT_TRACKING/V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.04_session_management.md
