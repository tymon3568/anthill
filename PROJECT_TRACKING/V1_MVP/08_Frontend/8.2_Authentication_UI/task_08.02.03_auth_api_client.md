# Task: Authentication API Client

**Task ID:** V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.03_auth_api_client.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.2_Authentication_UI
**Priority:** High
**Status:** Todo
**Assignee:**
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
  try {
    const response = await apiClient.post<LoginResponse>('/api/v1/auth/login', credentials);

    // SECURITY: Tokens are stored in httpOnly cookies by the backend
    // Client should NOT store tokens in localStorage (XSS vulnerability)
    // The backend /api/v1/auth/login endpoint sets access_token and refresh_token cookies
    // with httpOnly, secure, and sameSite attributes for maximum security

    return response;
  } catch (error) {
    if (error instanceof AuthApiError) {
      throw error;
    }

    // Map HTTP errors to AuthApiError
    if (error.status === 401) {
      throw new AuthApiError('INVALID_CREDENTIALS', 'Invalid email or password');
    }
    if (error.status === 429) {
      throw new AuthApiError('RATE_LIMITED', 'Too many login attempts. Please try again later.');
    }

    throw new AuthApiError('NETWORK_ERROR', 'Unable to connect to authentication service');
  }
}

export async function register(userData: RegisterRequest): Promise<RegisterResponse> {
  try {
    const response = await apiClient.post<RegisterResponse>('/api/v1/auth/register', userData);
    return response;
  } catch (error) {
    if (error instanceof AuthApiError) {
      throw error;
    }

    // Map HTTP errors to AuthApiError
    if (error.status === 409) {
      throw new AuthApiError('USER_EXISTS', 'An account with this email already exists');
    }
    if (error.status === 400) {
      throw new AuthApiError('VALIDATION_ERROR', 'Please check your input and try again');
    }

    throw new AuthApiError('NETWORK_ERROR', 'Unable to connect to authentication service');
  }
}

export function logout(): void {
  // DEPRECATED: Do not access localStorage for tokens
  // Tokens are managed via httpOnly cookies by the backend
  // Call backend logout endpoint to clear cookies server-side
  // Only user_data (non-sensitive) may be stored in localStorage
}

/**
 * @deprecated Token storage moved to httpOnly cookies
 * This function is kept for backward compatibility but should not be used
 */
export function getStoredToken(): string | null {
  console.warn('getStoredToken() is deprecated. Tokens are in httpOnly cookies.');
  return null;
}

export function isAuthenticated(): boolean {
  const token = getStoredToken();
  return !!token; // In production, also check token expiration
}
```

## Testing Steps:
- [ ] Test login with valid credentials
- [ ] Test login with invalid credentials (wrong password, non-existent email)
- [ ] Test registration with valid data
- [ ] Test registration with existing email
- [ ] Test network error handling
- [ ] Test token storage and retrieval
- [ ] Verify logout clears tokens

## References:
*   User service OpenAPI specification
*   `shared/error/src/lib.rs` - Backend error types
*   `services/user_service/api/handlers/auth.rs` - Auth endpoints
*   Project API patterns and conventions

## Notes / Discussion:
---
*   Follow user service API endpoints and DTOs
*   Handle multi-tenant headers if required
*   Consider security implications of token storage
*   Error messages should be user-friendly
*   API client should be reusable across the application

## AI Agent Log:
---
*   2025-11-12 10:30: Task created by Claude
    - Set up authentication API client structure
    - Included proper error handling and token management
    - Aligned with backend API expectations
    - Ready for implementation</content>
<parameter name="filePath">/home/arch/Project/test/anthill/PROJECT_TRACKING/V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.03_auth_api_client.md
