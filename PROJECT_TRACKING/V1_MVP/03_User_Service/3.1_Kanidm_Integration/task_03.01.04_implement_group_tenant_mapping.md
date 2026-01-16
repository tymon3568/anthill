# Task: Implement Group-to-Tenant Mapping

**Task ID:** V1_MVP/03_User_Service/3.1_Kanidm_Integration/task_03.01.04_implement_group_tenant_mapping.md  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.1_Kanidm_Integration  
**Priority:** ~~High~~ N/A  
**Status:** Cancelled
**Assignee:** ~~Claude~~  
**Created Date:** 2025-11-03  
**Last Updated:** 2026-01-04

## ⚠️ CANCELLED - Tech Stack Changed

**Reason:** The project has moved from Kanidm (external IdP) to **self-built email/password authentication** managed by User Service.

**Replacement:** Tenant context is now determined by:
- `X-Tenant-ID` header (for API clients/testing)
- Subdomain extraction from `Host` header (for browser clients)
- Direct association via `users.tenant_id` column in database

**Removed Components:**
- ~~`kanidm_tenant_groups` table~~ - No longer needed (migration kept for schema history)
- ~~Group-based tenant mapping logic~~ - Replaced by direct tenant_id in user record
- ~~Kanidm groups synchronization~~ - Not applicable

**See:** 
- `task_03.01.10_remove_kanidm_integration.md` - Kanidm removal task (completed)
- `3.6_Self_Auth_Enhancements/` - New production auth features

---

## Original Description (Archived)

~~Implement the mapping between Kanidm groups and Anthill tenants. When a user authenticates via Kanidm, their group memberships determine which tenant they belong to and their role within that tenant.~~

## Original Sub-tasks (Archived)

- [x] ~~1. Create `kanidm_tenant_groups` table migration~~ **CANCELLED**
- [x] ~~2. Define group naming convention (e.g., `tenant_{slug}_admins`, `tenant_{slug}_users`)~~ **CANCELLED**
- [x] ~~3. Implement `TenantRepository.find_by_kanidm_group()` method~~ **CANCELLED**
- [x] ~~4. Implement group-to-role mapping logic~~ **CANCELLED**
- [x] ~~5. Update OAuth2 callback handler to use group mapping~~ **CANCELLED**
- [x] ~~6. Create admin endpoint to manage group mappings~~ **CANCELLED**
- [x] ~~7. Write tests for group mapping logic~~ **CANCELLED**

## Notes

- The `kanidm_tenant_groups` migration exists but table is unused
- Group-based multi-tenancy was more complex than needed for MVP
- Direct `tenant_id` association in user record is simpler and sufficient
- Can revisit group-based tenancy post-MVP if SSO requirements emerge

## AI Agent Log:
---
*   2025-11-03: Task created for Kanidm group-to-tenant mapping
*   2026-01-04: Task CANCELLED - tech stack changed to self-built auth
    - Group mapping logic removed from user_service
    - Tenant context now derived from headers or user record
    - See task_03.01.10_remove_kanidm_integration.md for details
