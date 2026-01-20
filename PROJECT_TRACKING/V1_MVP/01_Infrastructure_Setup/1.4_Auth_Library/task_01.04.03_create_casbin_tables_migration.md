# Task: Create Casbin Database Tables Migration

**Task ID:** V1_MVP/01_Infrastructure_Setup/1.4_Auth_Library/task_01.04.03_create_casbin_tables_migration.md
**Version:** V1_MVP
**Phase:** 01_Infrastructure_Setup
**Module:** 1.4_Auth_Library
**Priority:** High
**Status:** Done
**Assignee:** Claude
**Created Date:** 2025-01-21
**Last Updated:** 2025-01-21

## Detailed Description:
Create database migration for Casbin RBAC tables. The `casbin_rule` table will store all policies and role assignments for multi-tenant authorization.

## Specific Sub-tasks:
- [ ] 1. Create new migration file: `migrations/20250110000003_create_casbin_tables.sql`
- [ ] 2. Create `casbin_rule` table with proper columns (ptype, v0, v1, v2, v3, v4, v5)
- [ ] 3. Add performance indexes on ptype and v1 (tenant_id) columns
- [ ] 4. Create helper views: casbin_policies and casbin_groupings (optional)
- [ ] 5. Test migration by running `sqlx migrate run`

## Acceptance Criteria:
- [x] Migration file created with proper timestamp naming convention
- [x] `casbin_rule` table structure matches Casbin requirements:
  - `ptype VARCHAR(12)` - 'p' (policy) or 'g' (grouping)
  - `v0 VARCHAR(128)` - subject (user_id or role name)
  - `v1 VARCHAR(128)` - domain (tenant_id)
  - `v2 VARCHAR(128)` - object (resource path)
  - `v3 VARCHAR(128)` - action (permission)
- [x] Indexes created for performance optimization
- [x] Migration applies successfully without errors

## Dependencies:
- V1_MVP/02_Database_Foundations/2.2_Migration_Testing/task_02.02.01_setup_migration_environment.md

## Related Documents:
- `migrations/20250110000003_create_casbin_tables.sql` (file to be created)
- `migrations/README.md`

## Notes / Discussion:
---
* Table structure must match Casbin's expected format exactly
* v1 column (tenant_id) needs index for performance in multi-tenant queries
* This migration should be run after core tables are created

## AI Agent Log:
---
*   2025-11-05 10:58: Task status updated by Claude
    - Casbin tables migration created and applied early in project
    - Still valid and in use for storing RBAC policies with self-built email/password auth
    - Status: Done ✓
*   2026-01-04: Note updated - Self-auth removed from tech stack, Casbin continues to be used for RBAC
