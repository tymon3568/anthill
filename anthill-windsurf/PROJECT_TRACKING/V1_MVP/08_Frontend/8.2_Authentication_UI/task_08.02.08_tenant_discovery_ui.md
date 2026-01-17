# Task: Implement Tenant Discovery & Check Availability UI

**Task ID:** V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.08_tenant_discovery_ui.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.2_Authentication_UI
**Priority:** Low
**Status:** NeedsReview
**Assignee:** Claude 
**Created Date:** 2025-01-26
**Last Updated:** 2025-01-27

## Detailed Description:
Enhance the registration flow by adding a "Tenant Availability Check" step or a standalone "Find My Tenant" page. This helps users understand if they are creating a new tenant or joining an existing one *before* they submit the full registration form.

## Specific Sub-tasks:
- [x] 1. Add "Check Tenant URL" Async Validator to Registration Form
    - [x] As user types `tenant_name`, convert to slug and check availability.
    - [x] Visual feedback: "Acme Corp (acme-corp) is available - You will be the Owner" vs "acme-corp exists - You will request to join".
- [ ] 2. (Optional) "Find Workspace" Page
    - [ ] Enter email to see which tenants you belong to (requires backend support).

## Acceptance Criteria:
- [x] Registration form provides real-time or on-blur feedback about tenant status.
- [x] Users clearly understand if they are creating vs joining a workspace.

## Dependencies:
- V1_MVP/08_Frontend/8.2_Authentication_UI/task_08.02.01_authentication_pages.md

## Notes / Discussion:
---
* This is a UX improvement on top of `task_08.02.01`. The backend handles the logic, but UI clarity prevents confusion.

## AI Agent Log:
---
### 2025-01-27 - Implementation Complete

**Changes Made:**

1. **Backend - New Public Endpoint** (`/api/v1/auth/check-tenant-slug`):
   - Added `CheckTenantSlugQuery` and `CheckTenantSlugResp` DTOs to `auth_dto.rs`
   - Added `check_tenant_slug` handler in `handlers.rs`
   - Registered route in `main.rs` as a public endpoint (no auth required)
   - Endpoint normalizes slug and checks if tenant exists via `find_by_slug`

2. **Frontend - API Client** (`auth.ts`):
   - Added `checkTenantSlug(slug)` method to `authApi`
   - Returns availability status and existing tenant name if not available

3. **Frontend - Registration Form** (`+page.svelte`):
   - Added tenant availability state tracking (`tenantCheckStatus`, `existingTenantName`)
   - Implemented debounced async check (400ms delay) as user types
   - Added visual feedback with icons:
     - **Available**: Green border, Building2 icon, "You will be the Owner"
     - **Exists**: Amber border, UserPlus icon, "You will request to join"
     - **Checking**: Loader spinner
     - **Error**: Gray alert icon with fallback message
   - Added `$derived` for real-time slug display

**Files Modified:**
- `services/user_service/core/src/domains/auth/dto/auth_dto.rs`
- `services/user_service/api/src/handlers.rs`
- `services/user_service/api/src/main.rs`
- `frontend/src/lib/api/auth.ts`
- `frontend/src/routes/(auth)/register/+page.svelte`
