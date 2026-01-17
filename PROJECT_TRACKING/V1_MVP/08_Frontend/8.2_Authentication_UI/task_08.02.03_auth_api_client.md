# Task: Authentication API Client

**Task ID:** V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.03_auth_api_client.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.2_Authentication_UI
**Priority:** High
**Status:** NeedsReview
**Assignee:** User
**Created Date:** 2025-11-12
**Last Updated:** 2025-11-12

## Detailed Description:
Create a centralized authentication API client for handling email/password login and registration. The client should integrate with the user service API, handle errors appropriately, and provide a clean interface for the authentication pages.

## Acceptance Criteria:
- [ ] Login API client function with proper error handling
- [ ] Registration API client function with proper error handling
- [ ] Typed request/response DTOs matching backend API
- [ ] Proper HTTP status code handling
- [ ] Network error handling and retry logic
- [ ] Integration with shared API infrastructure
- [ ] Session token handling and storage
- [ ] Clear error messages for different failure scenarios
- [ ] Code compiles without errors: `bun run check`
- [ ] API client is well-tested and documented

## Specific Sub-tasks:
- [ ] 1. Create authentication API types
    - [ ] 1.1. Define login request/response interfaces
    - [ ] 1.2. Define registration request/response interfaces
    - [ ] 1.3. Define error response types
    - [ ] 1.4. Create TypeScript types file (`src/lib/api/auth/types.ts`)

- [ ] 2. Implement authentication API client
    - [ ] 2.1. Create auth client module (`src/lib/api/auth/client.ts`)
    - [ ] 2.2. Implement login function with fetch API
    - [ ] 2.3. Implement register function with fetch API
    - [ ] 2.4. Add proper error handling and mapping

- [ ] 3. Integrate with shared API infrastructure
    - [ ] 3.1. Use shared fetch utilities for consistency
    - [ ] 3.2. Include tenant headers if required
    - [ ] 3.3. Handle authentication-specific headers
    - [ ] 3.4. Integrate with error handling utilities

- [ ] 4. Implement session management
    - [ ] 4.1. Store access tokens securely (localStorage/sessionStorage)
    - [ ] 4.2. Handle token refresh if applicable
    - [ ] 4.3. Provide logout functionality
    - [ ] 4.4. Check authentication status

## Dependencies:
*   Task: `task_08.02.04_api_infrastructure_core_setup.md` (Status: Done)
*   Task: `task_08.02.02_form_validation.md` (Status: Done)

## Files to Create/Modify:
*   `src/lib/api/auth/types.ts` - TypeScript interfaces for auth API
*   `src/lib/api/auth/client.ts` - Authentication API client implementation
*   `src/lib/api/auth/index.ts` - Auth API exports
*   `src/lib/stores/auth.ts` - Authentication state store (Svelte 5 runes)

## Code Examples:
```typescript
// src/lib/api/auth/types.ts
export interface LoginRequest {
  email: string;
  password: string;
}

export interface LoginResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
  user: {
    id: string;
    email: string;
    tenant_id: string;
  };
}

export interface RegisterRequest {
  email: string;
  password: string;
}

export interface RegisterResponse {
  user: {
    id: string;
    email: string;
    tenant_id: string;
  };
  message: string;
}

export interface AuthError {
  code: string;
  message: string;
  details?: Record<string, any>;
}
```

```typescript
// src/lib/api/auth/client.ts
import { apiClient } from '$lib/api/client';
import type { LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, AuthError } from './types';

export class AuthApiError extends Error {
  constructor(public code: string, message: string, public details?: any) {
    super(message);
    this.name = 'AuthApiError';
  }
}

export async function login(credentials: LoginRequest): Promise<LoginResponse> {
  // Call apiClient which returns ApiResponse<T> (does not throw)
  const response = await apiClient.post<LoginResponse>('/api/v1/auth/login', credentials);

  // Check if request was successful
  if (!response.success) {
    // Map known error messages to AuthApiError codes
    const errorMsg = response.error || 'Unknown error';

    // Check for specific error patterns
    if (errorMsg.includes('401') || errorMsg.toLowerCase().includes('invalid credentials')) {
      throw new AuthApiError('INVALID_CREDENTIALS', 'Invalid email or password');
    }
    if (errorMsg.includes('429') || errorMsg.toLowerCase().includes('rate limit')) {
      throw new AuthApiError('RATE_LIMITED', 'Too many login attempts. Please try again later.');
    }
    if (errorMsg.toLowerCase().includes('network') || errorMsg.toLowerCase().includes('fetch')) {
      throw new AuthApiError('NETWORK_ERROR', 'Unable to connect to authentication service');
    }

    // Generic error fallback
    throw new AuthApiError('UNKNOWN_ERROR', errorMsg);
  }

  // SECURITY: Tokens are stored in httpOnly cookies by the backend
  // Client should NOT store tokens in localStorage (XSS vulnerability)
  // The backend /api/v1/auth/login endpoint sets access_token and refresh_token cookies
  // with httpOnly, secure, and sameSite attributes for maximum security

  return response.data!; // TypeScript knows data exists when success=true
}

export async function register(userData: RegisterRequest): Promise<RegisterResponse> {
  // Call apiClient which returns ApiResponse<T> (does not throw)
  const response = await apiClient.post<RegisterResponse>('/api/v1/auth/register', userData);

  // Check if request was successful
  if (!response.success) {
    // Map known error messages to AuthApiError codes
    const errorMsg = response.error || 'Unknown error';

    // Check for specific error patterns
    if (errorMsg.includes('409') || errorMsg.toLowerCase().includes('already exists')) {
      throw new AuthApiError('USER_EXISTS', 'An account with this email already exists');
    }
    if (errorMsg.includes('400') || errorMsg.toLowerCase().includes('validation')) {
      throw new AuthApiError('VALIDATION_ERROR', 'Please check your input and try again');
    }
    if (errorMsg.toLowerCase().includes('network') || errorMsg.toLowerCase().includes('fetch')) {
      throw new AuthApiError('NETWORK_ERROR', 'Unable to connect to authentication service');
    }

    // Generic error fallback
    throw new AuthApiError('UNKNOWN_ERROR', errorMsg);
  }

  return response.data!; // TypeScript knows data exists when success=true
}

/**
 * Logout user by clearing httpOnly cookies server-side
 * 
 * This function calls the backend logout endpoint which clears all httpOnly cookies
 * (access_token, refresh_token) and the user_data cookie.
 * 
 * @throws {AuthApiError} If the logout request fails
 */
export async function logout(): Promise<void> {
  try {
    const response = await fetch('/api/v1/auth/logout', {
      method: 'POST',
      credentials: 'include', // Send httpOnly cookies to server
      headers: {
        'Content-Type': 'application/json'
      }
    });

    if (!response.ok) {
      const errorText = await response.text().catch(() => 'Unknown error');
      console.error('Logout failed:', response.status, errorText);
      throw new AuthApiError(
        'LOGOUT_FAILED',
        `Failed to logout: ${response.status} ${response.statusText}`
      );
    }

    // Success - cookies cleared server-side
    console.log('Logout successful');
  } catch (error) {
    // Network errors or fetch failures
    if (error instanceof AuthApiError) {
      throw error;
    }
    
    console.error('Logout network error:', error);
    throw new AuthApiError('NETWORK_ERROR', 'Unable to connect to logout service');
  }
}

/**
 * @deprecated Token storage moved to httpOnly cookies
 * This function is kept for backward compatibility but should not be used
 */
export function getStoredToken(): string | null {
  console.warn('getStoredToken() is deprecated. Tokens are in httpOnly cookies.');
  return null;
}

/**
 * Check authentication status
 * 
 * IMPORTANT: Authentication is determined SERVER-SIDE via httpOnly cookies.
 * 
 * Client-side authentication check approaches:
 * 
 * 1. RECOMMENDED - Check for user_data cookie (non-httpOnly, readable):
 *    The backend sets a 'user_data' cookie on successful login containing
 *    non-sensitive user info. Frontend can read this to determine auth state.
 * 
 *    export function isAuthenticated(): boolean {
 *      return document.cookie.includes('user_data=');
 *    }
 * 
 * 2. ALTERNATIVE - Call backend validation endpoint:
 *    Make a request to /api/v1/auth/me which validates the httpOnly cookies
 *    server-side and returns user info or 401.
 * 
 *    export async function checkAuth(): Promise<boolean> {
 *      try {
 *        const response = await fetch('/api/v1/auth/me', {
 *          credentials: 'include' // Send httpOnly cookies
 *        });
 *        return response.ok;
 *      } catch {
 *        return false;
 *      }
 *    }
 * 
 * 3. Server-side route protection (SvelteKit):
 *    Use hooks.server.ts to validate httpOnly cookies and redirect
 *    unauthenticated requests. Frontend just handles the redirect.
 * 
 * For this project, we use approach #1 (user_data cookie) for client-side checks
 * and approach #3 (server hooks) for actual route protection.
 */
export function isAuthenticated(): boolean {
  // Check if user_data cookie exists (set by backend on login)
  if (typeof document !== 'undefined') {
    return document.cookie.includes('user_data=');
  }
  return false;
}
```

## Testing Steps:
- [ ] Test login with valid credentials
- [ ] Test login with invalid credentials (wrong password, non-existent email)
- [ ] Test registration with valid data
- [ ] Test registration with existing email
- [ ] Test network error handling
- [ ] Verify httpOnly cookies are set after successful login (access_token, refresh_token)
- [ ] Verify user_data cookie (non-httpOnly) is readable client-side
- [ ] Verify logout clears all auth cookies
- [ ] Test isAuthenticated() returns true when user_data cookie exists
- [ ] Test isAuthenticated() returns false after logout
- [ ] Verify protected routes redirect to login when not authenticated

## References:
*   User service OpenAPI specification
*   `shared/error/src/lib.rs` - Backend error types
*   `services/user_service/api/handlers/auth.rs` - Auth endpoints
*   `frontend/src/hooks.server.ts` - Server-side authentication hooks
*   Project API patterns and conventions

## Notes / Discussion:
---
*   Follow user service API endpoints and DTOs
*   Handle multi-tenant headers if required
*   **SECURITY: Tokens stored in httpOnly cookies (access_token, refresh_token)**
*   **Client-side auth check uses user_data cookie (non-httpOnly, readable)**
*   **Server-side route protection via hooks.server.ts validates httpOnly cookies**
*   Error messages should be user-friendly
*   API client should be reusable across the application

## AI Agent Log:
---
*   2025-11-12 10:30: Task created by Claude
    - Set up authentication API client structure
    - Included proper error handling and token management
    - Aligned with backend API expectations
    - Ready for implementation
*   2026-01-17: Implementation verified and tests updated by Claude
    - Auth API client already exists in `src/lib/api/auth.ts` with all required methods
    - Email login/register, token refresh, logout, profile, permissions, sessions all implemented
    - Updated tests in `src/lib/api/auth.test.ts` for email/password auth (18 tests pass)
    - Fixed deprecated OAuth2 type definitions to include user/tenant fields
    - All acceptance criteria met: typed DTOs, error handling, session management</content>
<parameter name="filePath">/home/arch/Project/test/anthill/PROJECT_TRACKING/V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.03_auth_api_client.md
