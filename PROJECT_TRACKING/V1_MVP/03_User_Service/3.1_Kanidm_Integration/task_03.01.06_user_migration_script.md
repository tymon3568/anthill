# Task: Create User Migration Script (Password → Kanidm)

**Task ID:** V1_MVP/03_User_Service/3.1_Kanidm_Integration/task_03.01.06_user_migration_script.md  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.1_Kanidm_Integration  
**Priority:** ~~Medium~~ N/A  
**Status:** Cancelled
**Assignee:** ~~Claude~~  
**Created Date:** 2025-11-03  
**Last Updated:** 2026-01-04

## ⚠️ CANCELLED - Tech Stack Changed

**Reason:** The project has moved from Kanidm (external IdP) to **self-built email/password authentication** managed by User Service.

**Why this task is no longer needed:**
- No migration from password to Kanidm required
- Users continue to use email/password authentication
- No external IdP to migrate to
- User data remains in PostgreSQL (no sync needed)

**See:** 
- `task_03.01.10_remove_kanidm_integration.md` - Kanidm removal task (completed)
- `3.6_Self_Auth_Enhancements/` - New production auth features (email verification, password reset, rate limiting)

---

## Original Description (Archived)

~~Create scripts to migrate existing users from password-based authentication to Kanidm OAuth2 authentication. This includes:~~
- ~~Creating users in Kanidm from PostgreSQL user records~~
- ~~Sending migration invitation emails~~
- ~~Tracking migration progress~~
- ~~Supporting dual-auth transition period~~

## Original Sub-tasks (Archived)

- [x] ~~1. Create `scripts/migrate-user-to-kanidm.sh` for single user migration~~ **CANCELLED**
- [x] ~~2. Create `scripts/migrate-tenant-to-kanidm.sh` for bulk tenant migration~~ **CANCELLED**
- [x] ~~3. Create `scripts/sync-kanidm-users.sh` for periodic sync~~ **CANCELLED**
- [x] ~~4. Add migration tracking columns to users table~~ **CANCELLED**
- [x] ~~5. Implement invitation email sending~~ **CANCELLED**
- [x] ~~6. Create migration progress dashboard/report~~ **CANCELLED**
- [x] ~~7. Document migration runbook~~ **CANCELLED**

## Notes

- The migration scripts were partially developed but never used
- All Kanidm-related scripts have been deleted from `scripts/` directory
- Database columns for migration tracking (`migration_invited_at`, `migration_completed_at`) exist but are unused
- Can repurpose these columns for other features if needed (e.g., onboarding tracking)

## Deleted Files

The following files were removed as part of Kanidm cleanup:
- `scripts/migrate-users-to-kanidm.sh`
- `scripts/export-users-for-kanidm.sh`
- `scripts/setup-kanidm-tenant-groups.sh`
- `scripts/test-kanidm-init.sh`

## AI Agent Log:
---
*   2025-11-03: Task created for user migration planning
*   2026-01-04: Task CANCELLED - tech stack changed to self-built auth
    - No migration to external IdP needed
    - Migration scripts deleted from repository
    - See task_03.01.10_remove_kanidm_integration.md for details
