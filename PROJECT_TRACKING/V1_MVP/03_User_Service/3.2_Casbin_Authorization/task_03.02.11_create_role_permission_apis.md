# Task: Create Role and Permission Management APIs

**Task ID:** V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.11_create_role_permission_apis.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.2_Casbin_Authorization
**Priority:** High
**Status:** Todo
**Assignee:**
**Created Date:** 2025-01-21
**Last Updated:** 2025-01-21

## Detailed Description:
Create REST APIs for managing roles and permissions in the Casbin RBAC system, allowing administrators to assign roles and permissions to users.

## Specific Sub-tasks:
- [ ] 1. Create `POST /api/v1/admin/roles` - Create custom roles
- [ ] 2. Create `GET /api/v1/admin/roles` - List all roles in tenant
- [ ] 3. Create `PUT /api/v1/admin/roles/{role_id}` - Update role permissions
- [ ] 4. Create `DELETE /api/v1/admin/roles/{role_id}` - Delete custom role
- [ ] 5. Create `POST /api/v1/admin/users/{user_id}/roles` - Assign role to user
- [ ] 6. Create `DELETE /api/v1/admin/users/{user_id}/roles/{role_id}` - Remove role from user
- [ ] 7. Create `GET /api/v1/admin/permissions` - List all available permissions
- [ ] 8. Implement proper authorization (admin only endpoints)

## Acceptance Criteria:
- [ ] Role management APIs fully functional
- [ ] Permission assignment APIs working correctly
- [ ] Admin-only access properly enforced
- [ ] Changes reflected immediately in Casbin enforcer
- [ ] Proper error handling for invalid operations
- [ ] OpenAPI documentation updated
- [ ] Unit tests for all endpoints

## Dependencies:
- V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.08_create_role_management_apis.md

## Related Documents:
- `services/user_service/api/src/handlers/admin.rs` (file to be created)
- `services/user_service/core/src/domains/auth/dto/admin_dto.rs` (file to be created)
- `shared/openapi/user.yaml` (update required)

## Notes / Discussion:
---
* All admin endpoints require admin role authorization
* Role changes should be audited in the system
* Consider implementing role inheritance and hierarchies
* Validate role names and permission formats

## AI Agent Log:
---
* (Log will be automatically updated by AI agent when starting and executing task)