# Task: Integrate Backend Authentication & Authorization

**Task ID:** V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.02_integrate_backend_auth.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.2_Authentication_UI
**Priority:** Critical
**Status:** Not Started
**Assignee:** tymon3568
**Created Date:** 2025-11-05
**Last Updated:** 2025-11-05

## Detailed Description:
Implement production-ready authentication and authorization integration with Kanidm OAuth2 and Casbin RBAC. This includes OAuth2 flow handling, JWT token management, route protection, and multi-tenant permission checking.

## Specific Sub-tasks:
- [ ] 1. Implement OAuth2 Authorization Code Flow with Kanidm
- [ ] 2. Create OAuth2 callback handler for token exchange
- [ ] 3. Implement JWT token validation and parsing
- [ ] 4. Add automatic token refresh before expiry
- [ ] 5. Create route guards for protected pages
- [ ] 6. Implement Casbin permission checking
- [ ] 7. Add multi-tenant context extraction from JWT
- [ ] 8. Create auth middleware for SvelteKit
- [ ] 9. Implement secure token storage (httpOnly cookies)
- [ ] 10. Add logout functionality with token cleanup
- [ ] 11. Create error handling for auth failures
- [ ] 12. Implement loading states during auth checks
- [ ] 13. Add unit tests for OAuth2 flow
- [ ] 14. Add unit tests for permission checking
- [ ] 15. Add E2E tests for complete auth flow
- [ ] 16. Update documentation for production deployment

## Acceptance Criteria:
- [ ] OAuth2 flow working end-to-end with Kanidm
- [ ] JWT tokens properly validated and parsed
- [ ] Automatic token refresh working
- [ ] Protected routes properly guarded
- [ ] Casbin permissions enforced correctly
- [ ] Multi-tenant context extracted from JWT groups
- [ ] Secure token storage implemented
- [ ] Logout functionality complete with cleanup
- [ ] Error handling for network/auth failures
- [ ] Loading states during auth operations
- [ ] All auth flows tested with unit tests
- [ ] E2E tests covering complete user journeys
- [ ] Production deployment documentation updated

## Technical Implementation Details:

### OAuth2 Flow Implementation:
```typescript
// OAuth2 endpoints to implement
GET  /api/v1/auth/oauth/authorize  // Initiate OAuth2 flow
POST /api/v1/auth/oauth/callback   // Handle OAuth2 callback
POST /api/v1/auth/oauth/refresh    // Refresh access token
POST /api/v1/auth/logout          // Logout with token cleanup
```

### JWT Token Structure:
```typescript
interface KanidmJWT {
  sub: string;           // Kanidm user UUID
  email: string;         // User email
  groups: string[];      // Kanidm groups (tenant mappings)
  exp: number;           // Expiry timestamp
  iat: number;           // Issued at timestamp
}
```

### Route Protection:
```typescript
// SvelteKit route guards
// +layout.ts - Root layout protection
// +page.ts - Page-level protection
// Custom guards for specific permissions
```

### Casbin Integration:
```typescript
// Permission checking with tenant context
interface PermissionCheck {
  subject: string;    // User ID
  tenant: string;     // Tenant ID
  resource: string;   // Resource being accessed
  action: string;     // Action (read, write, delete, etc.)
}
```

### Token Management:
```typescript
// Secure token storage and refresh
interface TokenManager {
  storeTokens(accessToken: string, refreshToken: string): void;
  getAccessToken(): string | null;
  refreshTokens(): Promise<boolean>;
  clearTokens(): void;
}
```

## Dependencies:
- V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.01_create_login_registration_pages.md
- Backend Kanidm OAuth2 service running
- Backend Casbin RBAC service running
- Shared auth library (`shared/auth`)

## Related Documents:
- `ARCHITECTURE.md` - System architecture and auth design
- `shared/auth/src/lib.rs` - Backend auth implementation
- `frontend/src/lib/stores/auth.svelte.ts` - Current auth store
- `frontend/src/lib/hooks/useAuth.ts` - Current auth hooks

## Implementation Notes:

### Security Considerations:
- Never store tokens in localStorage (XSS vulnerable)
- Use httpOnly cookies for token storage
- Implement PKCE for OAuth2 security
- Validate JWT signatures on backend
- Implement CSRF protection

### Multi-tenant Handling:
- Extract tenant_id from JWT groups claim
- Map Kanidm groups to Anthill tenants
- Include tenant context in all API calls
- Enforce tenant isolation in permissions

### Error Scenarios:
- Network failures during OAuth2 flow
- Token expiry during user session
- Invalid/revoked tokens
- Kanidm server unavailable
- Permission denied scenarios

### Testing Strategy:
- Mock Kanidm OAuth2 responses for unit tests
- Test token refresh logic
- Test permission checking with various user roles
- E2E tests for complete login->protected page->logout flow
- Test error handling scenarios

## Estimated Effort:
- **Development**: 3-4 days
- **Testing**: 2 days
- **Documentation**: 1 day
- **Total**: ~1 week

## Risk Assessment:
- **High**: OAuth2 flow complexity
- **Medium**: Token refresh timing issues
- **Medium**: Multi-tenant permission edge cases
- **Low**: UI integration with existing components

## Success Metrics:
- All OAuth2 flows working in development environment
- JWT validation passing for all test users
- Route protection working for all protected pages
- Casbin permissions correctly enforced
- E2E tests passing for auth user journeys
- No security vulnerabilities in token handling</content>
<parameter name="filePath">/home/arch/Project/test/anthill/PROJECT_TRACKING/V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.02_integrate_backend_auth.md
