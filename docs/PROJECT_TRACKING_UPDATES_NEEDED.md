# PROJECT_TRACKING Updates Needed for Kanidm Migration

**Created**: 2025-11-03  
**Purpose**: Track all task files in PROJECT_TRACKING that need updates due to Kanidm migration

---

## Summary

Với việc chuyển từ custom JWT auth sang Kanidm, các tasks sau cần được review và update:

- **Total tasks affected**: ~30+ files
- **Categories**:
  - Authentication implementation tasks (DEPRECATED/COMPLETE)
  - Testing tasks (NEEDS MAJOR UPDATES)
  - Frontend auth UI tasks (NEEDS UPDATES for OAuth2)
  - Integration tasks (marketplace OAuth remains same)
  - Monitoring/Security tasks (minor updates)

---

## 🔴 HIGH PRIORITY - Authentication Tasks (Mark as DEPRECATED/ARCHIVED)

These tasks are no longer relevant as Kanidm handles authentication:

### 03_User_Service/3.1_Core_Authentication/
All tasks in this folder are **DEPRECATED** - Kanidm handles this now.

**Files to mark as ARCHIVED**:
```
✗ task_03.01.01_implement_user_registration.md
  → Kanidm UI handles registration
  
✗ task_03.01.02_implement_tenant_resolution_on_login.md
  → Tenant mapping from Kanidm groups
  
✗ task_03.01.03_implement_jwt_token_generation.md
  → Kanidm generates JWT tokens
  
✗ task_03.01.04_implement_refresh_token_logic.md
  → Kanidm handles token refresh
  
✗ task_03.01.05_implement_session_management.md
  → Kanidm manages sessions
  
✗ task_03.01.06_implement_password_hashing.md
  → Kanidm handles password storage
```

**Action**: Add deprecation notice to each file header.

---

## 🟡 MEDIUM PRIORITY - Testing Tasks (NEEDS MAJOR UPDATES)

### 03_User_Service/3.4_Testing/

#### task_03.04.02_implement_integration_testing.md
**Status**: NEEDS COMPLETE REWRITE

**Current**: Tests JWT generation, password hashing, session management  
**New**: Test OAuth2 flow, Kanidm token validation, group mapping

**Changes needed**:
- Replace "Registration/login flow tests" → "OAuth2 callback handling tests"
- Replace "JWT token lifecycle tests" → "Kanidm JWT validation tests"
- Replace "Password change flow tests" → "Kanidm password reset redirect tests"
- Add "Kanidm group to tenant mapping tests"
- Add "OAuth2 PKCE flow tests"

**Files to DELETE**:
```
✗ services/user_service/api/tests/auth_flow_tests.rs
  → Replace with oauth2_flow_tests.rs

✗ services/user_service/api/tests/jwt_session_security_tests.rs
  → Replace with kanidm_token_validation_tests.rs
```

---

## 🟢 LOW PRIORITY - Frontend Tasks (NEEDS UPDATES)

### 08_Frontend/8.2_Authentication_UI/

#### task_08.02.01_create_login_registration_pages.md
**Status**: NEEDS MODERATE UPDATES

**Current**: Traditional login/registration forms  
**New**: OAuth2 authorization flow

**Changes needed**:
```diff
- Login page with email/password fields
+ OAuth2 "Login with Kanidm" button
+ OAuth2 callback handler page

- Registration form with validation
+ Redirect to Kanidm registration UI

- Password strength indicator
+ (Removed - Kanidm handles this)

- "Forgot Password" link
+ Redirect to Kanidm password reset

- Session management and token storage
+ Store Kanidm access_token (same as before)
```

**Files to create**:
```
+ frontend/src/routes/oauth/callback/+page.svelte
+ frontend/src/lib/auth/kanidm-oauth.ts
```

**Files to update**:
```
~ frontend/src/routes/login/+page.svelte → Simplified to OAuth button
~ frontend/src/routes/register/+page.svelte → Redirect page
~ frontend/src/lib/auth/auth-store.ts → Update to use Kanidm tokens
```

---

## ⚪ NO CHANGES NEEDED

These tasks reference "authorization" (Casbin) which remains unchanged:

### 03_User_Service/3.2_Casbin_Authorization/
**Status**: ✅ NO CHANGES - Casbin stays the same

**Files**:
- task_03.02.xx_* (all Casbin tasks)
  - Only minor update: Auth extractors now use Kanidm JWT claims
  - Core Casbin logic unchanged

### 03_User_Service/3.3_User_Management/
**Status**: ⚠️ MINOR CHANGES

**Changes needed**:
- Update user profile endpoints to sync with Kanidm
- Add `kanidm_user_id` field to user entities
- Update tenant management to map Kanidm groups

---

## 🔵 EXTERNAL OAUTH (NO CHANGES)

These tasks deal with marketplace OAuth (Shopee, Lazada, etc.) - unaffected:

### 06_Integration_Service/6.1_Adapter_Pattern/
**Status**: ✅ NO CHANGES

**Files**:
- task_06.01.02_implement_shopee_adapter.md (Shopee OAuth)
- task_06.01.03_implement_lazada_adapter.md (Lazada OAuth)
- task_06.01.01_create_marketplace_adapter_trait.md

These use marketplace-specific OAuth, not our auth system.

---

## 📋 NEW TASKS TO CREATE

Need to create new tasks for Kanidm integration:

### New folder: 03_User_Service/3.1_Kanidm_Integration/

```
task_03.01.01_setup_kanidm_server.md
  - Deploy Kanidm via docker-compose
  - Configure OAuth2 client
  - Setup groups for multi-tenancy

task_03.01.02_create_kanidm_client_library.md
  - Create shared/kanidm_client crate
  - Implement OAuth2 flow (Authorization Code + PKCE)
  - Implement JWT token validation

task_03.01.03_implement_oauth2_endpoints.md
  - GET /api/v1/auth/oauth/authorize
  - POST /api/v1/auth/oauth/callback
  - POST /api/v1/auth/oauth/refresh

task_03.01.04_implement_group_tenant_mapping.md
  - Create kanidm_tenant_groups table migration
  - Implement group→tenant mapping logic
  - Sync users from Kanidm to PostgreSQL

task_03.01.05_update_auth_extractors.md
  - Modify AuthUser extractor for Kanidm JWT
  - Update RequireAdmin to use Kanidm groups
  - Update Casbin subject from Kanidm claims

task_03.01.06_user_migration_script.md
  - Script to migrate existing users to Kanidm
  - Email notification for password reset
  - Verify migration success
```

---

## 🔧 TASKS_OVERVIEW.md Updates

### Current mentions of authentication:

**Line 49**: Phase 3 title  
```diff
- ### [🔄] Phase 3: User Service (Auth & Tenancy) - `In Progress 95%`
+ ### [✅] Phase 3: User Service (Kanidm Integration & Tenancy) - `Completed`
```

**Line 54-55**: Core Authentication  
```diff
- - [✅] 3.1 Core Authentication - `Completed`
-       → [View folder](./V1_MVP/03_User_Service/3.1_Core_Authentication/)
+ - [⚠️] 3.1 Core Authentication - `DEPRECATED - Replaced by Kanidm`
+       → [View folder](./V1_MVP/03_User_Service/3.1_Core_Authentication/)
+ - [🔄] 3.1 Kanidm Integration - `In Progress`
+       → [View folder](./V1_MVP/03_User_Service/3.1_Kanidm_Integration/)
```

**Line 140-141**: Frontend Auth UI  
```diff
- - [⏳] 8.2 Authentication UI - `Todo`
+ - [⏳] 8.2 Authentication UI (OAuth2) - `Todo - Needs Update`
        → [View folder](./V1_MVP/08_Frontend/8.2_Authentication_UI/)
```

**Line 273**: Environment variables example  
```diff
- export JWT_SECRET="your-secret-key-here"
+ export KANIDM_URL="https://idm.example.com"
+ export KANIDM_OAUTH2_CLIENT_ID="anthill"
+ export KANIDM_OAUTH2_CLIENT_SECRET="your-client-secret"
```

---

## 📊 Statistics

### Files by Action Required:

| Action | Count | Category |
|--------|-------|----------|
| **DEPRECATED** | ~6 | Custom JWT auth implementation |
| **MAJOR REWRITE** | ~3 | Authentication testing |
| **MODERATE UPDATE** | ~2 | Frontend auth UI |
| **MINOR UPDATE** | ~5 | User management, extractors |
| **NO CHANGE** | ~15+ | Casbin, marketplace OAuth |
| **NEW TASKS** | 6 | Kanidm integration |

### Estimated Effort:

- **Documentation updates**: 2-3 hours
- **Test rewrites**: 4-6 hours  
- **Frontend OAuth2 implementation**: 6-8 hours
- **Total**: ~15-20 hours

---

## ✅ Action Plan

### Phase 1: Documentation (NOW)
1. ✅ Create this summary document
2. [ ] Add DEPRECATED notices to 3.1_Core_Authentication tasks
3. [ ] Update TASKS_OVERVIEW.md
4. [ ] Create new task files for Kanidm integration

### Phase 2: Code Changes (NEXT)
5. [ ] Create shared/kanidm_client library
6. [ ] Refactor shared/auth (remove JWT gen, add validation)
7. [ ] Delete shared/jwt
8. [ ] Update user_service for OAuth2

### Phase 3: Testing (AFTER CODE)
9. [ ] Delete old JWT tests
10. [ ] Write new Kanidm integration tests
11. [ ] Update frontend for OAuth2 flow

### Phase 4: Deployment (FINAL)
12. [ ] Setup Kanidm in docker-compose
13. [ ] Migrate existing users
14. [ ] Deploy and verify

---

## 📝 Next Steps

**Immediate**: 
1. Commit this document
2. Start marking deprecated tasks
3. Create new Kanidm integration task folder

**See**: `docs/KANIDM_MIGRATION_PLAN.md` for full migration plan.

---

**Last Updated**: 2025-11-03  
**Status**: Summary created, action plan defined
