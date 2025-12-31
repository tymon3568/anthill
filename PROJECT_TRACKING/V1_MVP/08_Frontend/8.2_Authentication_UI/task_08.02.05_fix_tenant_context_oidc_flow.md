# Task: Fix Tenant Context and OIDC Authentication Flow

**Task ID:** V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.05_fix_tenant_context_oidc_flow.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.2_Authentication_UI
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-12-31
**Last Updated:** 2025-12-31

## Detailed Description:
Fix the authentication flow to properly handle tenant context and implement Kanidm OIDC integration. Currently, the login fails with "400 Bad Request: Tenant context required" because:

1. Frontend login form only collects Email/Password, but backend requires tenant context
2. Backend CORS only allows `localhost:5173`, not tenant subdomains (`*.localhost:5173`)
3. Frontend lacks Kanidm OIDC "Sign in with Kanidm" button integration

## Issues Identified:

### Issue 1: Tenant Context Missing
- **Location:** `frontend/src/routes/(auth)/login/+page.svelte`
- **Problem:** Login form calls `authStore.emailLogin` without tenant identifier
- **Backend Expectation:** Either `X-Tenant-Id` header or tenant subdomain (e.g., `acme.localhost`)

### Issue 2: CORS Blocking Tenant Subdomains
- **Location:** Backend user_service CORS configuration
- **Problem:** Only `localhost:5173` is allowed, not `*.localhost:5173`
- **Impact:** Cannot use subdomain-based tenant identification

### Issue 3: Missing Kanidm OIDC Integration
- **Location:** `frontend/src/routes/(auth)/login/+page.svelte`
- **Problem:** No "Sign in with Kanidm" button despite Kanidm being configured
- **Kanidm Status:** Running at https://localhost:8300 with test user (alice / Test123!@#)

## Technical Approach:

### Option A: Add Tenant Input Field (Quick Fix)
- Add a "Tenant/Organization" dropdown or input to login form
- Pass tenant ID in `X-Tenant-Id` header with API requests
- Pros: Simple, works immediately
- Cons: Extra step for users

### Option B: Subdomain-Based Tenancy (Recommended)
- Update backend CORS to allow `*.localhost:5173` in development
- Parse tenant from subdomain in frontend hooks
- Set `X-Tenant-Id` header automatically
- Pros: Cleaner UX, standard SaaS pattern
- Cons: Requires CORS and DNS/hosts config

### Option C: Implement Full Kanidm OIDC Flow
- Add "Sign in with Kanidm" button
- Implement OAuth2 authorization code flow with PKCE
- Map Kanidm groups to tenants
- Pros: Single sign-on, enterprise-ready
- Cons: More complex, requires Kanidm client setup

## Specific Sub-tasks:

### Phase 1: Backend CORS Fix
- [ ] 1.1. Update user_service CORS config to allow tenant subdomains
- [ ] 1.2. Add `*.localhost:5173` and `*.localhost:8000` to allowed origins
- [ ] 1.3. Test CORS with subdomain requests

### Phase 2: Frontend Tenant Context
- [ ] 2.1. Create tenant detection hook in `hooks.server.ts` (parse subdomain)
- [ ] 2.2. Add tenant context to `event.locals`
- [ ] 2.3. Pass `X-Tenant-Id` header in API client requests
- [ ] 2.4. Add fallback tenant input field for localhost development

### Phase 3: Kanidm OIDC Integration (Optional)
- [ ] 3.1. Add "Sign in with Kanidm" button to login page
- [ ] 3.2. Implement OAuth2 authorize redirect
- [ ] 3.3. Handle OAuth2 callback and token exchange
- [ ] 3.4. Map Kanidm user to local user + tenant
- [ ] 3.5. Test full OIDC flow with Kanidm

### Phase 4: Testing & Validation
- [ ] 4.1. Test login with tenant subdomain
- [ ] 4.2. Test login with X-Tenant-Id header
- [ ] 4.3. Test protected route access after login
- [ ] 4.4. Test dashboard layout and navigation
- [ ] 4.5. Verify SSR and CSR behavior

## Acceptance Criteria:
- [ ] Login form works with tenant context (subdomain or header)
- [ ] CORS allows tenant subdomains in development
- [ ] Protected routes accessible after successful login
- [ ] Dashboard layout renders correctly
- [ ] Session persists across page refreshes
- [ ] Logout clears session and redirects to login

## Dependencies:
- task_08.02.01_authentication_pages.md (Done)
- task_08.02.03_auth_api_client.md (Done)
- task_08.02.04_session_management.md (Done)

## Related Documents:
- `frontend/src/routes/(auth)/login/+page.svelte` - Login page
- `frontend/src/lib/api/auth.ts` - Auth API client
- `frontend/src/hooks.server.ts` - Server hooks
- `services/user_service/api/src/main.rs` - Backend CORS config
- `infra/docker_compose/docker-compose.yml` - Kanidm service config

## Files to Modify:

### Backend (user_service)
```
services/user_service/api/src/main.rs  # CORS configuration
.env                                    # CORS_ORIGINS variable
```

### Frontend
```
frontend/src/hooks.server.ts           # Tenant detection from subdomain
frontend/src/routes/(auth)/login/+page.svelte  # Add tenant input / OIDC button
frontend/src/lib/api/client.ts         # Add X-Tenant-Id header
frontend/src/lib/stores/auth.svelte.ts # Update login to include tenant
frontend/.env                          # Kanidm OIDC config
```

## Environment Setup for Testing:

### Local DNS (for subdomain testing)
Add to `/etc/hosts`:
```
127.0.0.1 acme.localhost
127.0.0.1 demo.localhost
```

### Kanidm Test Credentials
- URL: https://localhost:8300
- User: alice
- Password: Test123!@#

## Risk Assessment:
- **Medium Risk:** Changes touch auth flow which is security-sensitive
- **Mitigation:** Thorough testing of all auth scenarios
- **Rollback:** Can revert to header-based tenant if subdomain causes issues

## Notes / Discussion:
---
* This task was created after SvelteKit security upgrade (task_08.01.02) revealed pre-existing auth issues
* Recommend implementing Option B (subdomain) + Option A (fallback input) for flexibility
* OIDC integration (Option C) can be done as follow-up if email/password login works first
* Backend already supports tenant context via middleware - just need frontend to send it

## AI Agent Log:
---
* 2025-12-31 22:45: Task created by Claude
  - Extracted from manual testing results of task_08.01.02
  - Auth flow blocked due to tenant context requirement
  - Three main issues identified: tenant input, CORS, OIDC
  - Recommended approach: Fix CORS + subdomain detection first
