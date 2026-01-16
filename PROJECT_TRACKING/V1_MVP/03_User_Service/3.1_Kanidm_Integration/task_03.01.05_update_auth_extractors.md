# Task: Update Auth Extractors for Kanidm JWT

**Task ID:** V1_MVP/03_User_Service/3.1_Kanidm_Integration/task_03.01.05_update_auth_extractors.md  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.1_Kanidm_Integration  
**Priority:** ~~High~~ N/A  
**Status:** Done
**Assignee:** ~~Claude~~  
**Created Date:** 2025-11-03  
**Last Updated:** 2026-01-04

## ⚠️ CANCELLED - Tech Stack Changed

**Reason:** The project has moved from Kanidm (external IdP) to **self-built email/password authentication** managed by User Service.

**Replacement:** Auth extractors have been simplified:
- `AuthUser` struct no longer includes `kanidm_user_id` field
- `KanidmClientProvider` trait removed
- JWT validation uses shared secret instead of Kanidm public key (JWKS)
- Single JWT validation path (no dual Kanidm/legacy detection)

**Current Auth Extractors:**
- `AuthUser` - Extracts user_id, tenant_id, role, email from JWT claims
- `RequireAdmin` - Requires admin role in JWT
- `RequirePermission` - Casbin permission check
- `JwtSecretProvider` - Provides JWT signing secret

**Removed Components:**
- ~~`KanidmClientProvider` trait~~ - No longer needed
- ~~`from_kanidm_claims()` method~~ - No longer needed
- ~~`is_kanidm_user()` method~~ - No longer needed
- ~~Dual token validation (Kanidm + legacy)~~ - Simplified to single path

**See:** 
- `task_03.01.10_remove_kanidm_integration.md` - Kanidm removal task (completed)
- `shared/auth/src/extractors.rs` - Current simplified implementation

---

## Original Description (Archived)

~~Update the auth extractors in `shared/auth` to support both Kanidm JWTs and legacy custom JWTs during the migration period. The extractors should auto-detect token type and validate accordingly.~~

## Original Sub-tasks (Archived)

- [x] ~~1. Add `KanidmClientProvider` trait to extractors.rs~~ **CANCELLED**
- [x] ~~2. Add `kanidm_user_id` field to `AuthUser` struct~~ **CANCELLED**
- [x] ~~3. Implement `from_kanidm_claims()` for AuthUser~~ **CANCELLED**
- [x] ~~4. Implement dual token validation (try Kanidm first, fall back to legacy)~~ **CANCELLED**
- [x] ~~5. Add `is_kanidm_user()` helper method~~ **CANCELLED**
- [x] ~~6. Update `AuthzState` to include KanidmClient~~ **CANCELLED**
- [x] ~~7. Write tests for both token types~~ **CANCELLED**

## Notes

- The `shared/auth/src/extractors.rs` was previously updated to support dual auth
- All Kanidm-related code has been removed as part of task 03.01.10
- JWT validation is now simpler and more maintainable
- No external dependencies for token validation

## AI Agent Log:
---
*   2025-11-03: Task created for Kanidm auth extractor updates
*   2026-01-04: Task CANCELLED - tech stack changed to self-built auth
    - `KanidmClientProvider` trait removed from extractors.rs
    - `kanidm_user_id` field removed from AuthUser
    - Dual validation logic removed
    - See task_03.01.10_remove_kanidm_integration.md for details
