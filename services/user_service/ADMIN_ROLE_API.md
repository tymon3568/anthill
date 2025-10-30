# Admin Role & Permission Management APIs

## Overview

This document describes the admin-only endpoints for managing roles and permissions in the User Service. These endpoints allow administrators to create custom roles, assign permissions, and manage user role assignments.

## Authentication

All endpoints require:
- Valid JWT token in `Authorization: Bearer <token>` header
- User must have `admin` role in their tenant

## Endpoints

### Role Management

#### 1. Create Custom Role

Create a new role with specified permissions.

```http
POST /api/v1/admin/roles
Content-Type: application/json
Authorization: Bearer <admin_token>

{
  "role_name": "inventory_manager",
  "description": "Manages inventory and stock levels",
  "permissions": [
    {
      "resource": "products",
      "action": "read"
    },
    {
      "resource": "products",
      "action": "write"
    },
    {
      "resource": "inventory",
      "action": "read"
    },
    {
      "resource": "inventory",
      "action": "write"
    },
    {
      "resource": "inventory",
      "action": "adjust"
    }
  ]
}
```

**Response (201 Created):**
```json
{
  "role_name": "inventory_manager",
  "description": "Manages inventory and stock levels",
  "permissions_count": 5,
  "message": "Role 'inventory_manager' created successfully with 5 permissions"
}
```

**Validation:**
- `role_name`: 1-100 chars, lowercase alphanumeric + underscores only
- `description`: max 500 chars (optional)
- `permissions`: at least 1 permission required
- Each permission must have valid `resource` and `action`

**Errors:**
- `400`: Invalid request format
- `409`: Role already exists

---

#### 2. List All Roles

Get all roles in the current tenant with their permissions and user counts.

```http
GET /api/v1/admin/roles
Authorization: Bearer <admin_token>
```

**Response (200 OK):**
```json
{
  "roles": [
    {
      "role_name": "admin",
      "description": null,
      "permissions": [
        {
          "resource": "users",
          "action": "read"
        },
        {
          "resource": "users",
          "action": "write"
        }
      ],
      "user_count": 2
    },
    {
      "role_name": "inventory_manager",
      "description": null,
      "permissions": [
        {
          "resource": "products",
          "action": "read"
        },
        {
          "resource": "products",
          "action": "write"
        },
        {
          "resource": "inventory",
          "action": "adjust"
        }
      ],
      "user_count": 5
    }
  ],
  "total": 2
}
```

---

#### 3. Update Role Permissions

Update permissions for an existing role (replaces all existing permissions).

```http
PUT /api/v1/admin/roles/{role_name}
Content-Type: application/json
Authorization: Bearer <admin_token>

{
  "description": "Updated description",
  "permissions": [
    {
      "resource": "products",
      "action": "read"
    },
    {
      "resource": "inventory",
      "action": "read"
    }
  ]
}
```

**Response (200 OK):**
```json
{
  "role_name": "inventory_manager",
  "permissions_count": 2,
  "message": "Role 'inventory_manager' updated with 2 permissions"
}
```

**Restrictions:**
- Cannot modify system roles (`admin`, `user`)

**Errors:**
- `403`: Attempting to modify system role
- `404`: Role not found

---

#### 4. Delete Role

Delete a custom role (cannot delete system roles or roles assigned to users).

```http
DELETE /api/v1/admin/roles/{role_name}
Authorization: Bearer <admin_token>
```

**Response (200 OK):**
```json
{
  "role_name": "inventory_manager",
  "message": "Role 'inventory_manager' deleted successfully"
}
```

**Restrictions:**
- Cannot delete system roles (`admin`, `user`)
- Cannot delete roles currently assigned to users

**Errors:**
- `403`: Attempting to delete system role
- `404`: Role not found
- `409`: Role is assigned to users

---

### User Role Assignment

#### 5. Assign Role to User

Assign a role to a user.

```http
POST /api/v1/admin/users/{user_id}/roles/assign
Content-Type: application/json
Authorization: Bearer <admin_token>

{
  "role_name": "inventory_manager"
}
```

**Response (200 OK):**
```json
{
  "user_id": "01939f4c-1234-7890-abcd-ef1234567890",
  "role_name": "inventory_manager",
  "message": "Role 'inventory_manager' assigned to user successfully"
}
```

**Validation:**
- User must exist in admin's tenant
- Role must exist in tenant

**Errors:**
- `404`: User or role not found
- `409`: User already has this role

---

#### 6. Remove Role from User

Remove a role from a user.

```http
DELETE /api/v1/admin/users/{user_id}/roles/{role_name}/remove
Authorization: Bearer <admin_token>
```

**Response (200 OK):**
```json
{
  "user_id": "01939f4c-1234-7890-abcd-ef1234567890",
  "role_name": "inventory_manager",
  "message": "Role 'inventory_manager' removed from user successfully"
}
```

**Restrictions:**
- Cannot remove user's only role (users must have at least one role)

**Errors:**
- `400`: Attempting to remove user's last role
- `404`: User doesn't have this role

---

#### 7. Get User's Roles

Get all roles assigned to a specific user.

```http
GET /api/v1/admin/users/{user_id}/roles
Authorization: Bearer <admin_token>
```

**Response (200 OK):**
```json
{
  "user_id": "01939f4c-1234-7890-abcd-ef1234567890",
  "roles": [
    "user",
    "inventory_manager"
  ]
}
```

---

### Permission Management

#### 8. List Available Permissions

Get all available permissions in the system.

```http
GET /api/v1/admin/permissions
Authorization: Bearer <admin_token>
```

**Response (200 OK):**
```json
{
  "permissions": [
    {
      "resource": "users",
      "actions": ["read", "write", "delete"],
      "description": "Manage user accounts and profiles"
    },
    {
      "resource": "products",
      "actions": ["read", "write", "delete", "import"],
      "description": "Manage product catalog and inventory"
    },
    {
      "resource": "orders",
      "actions": ["read", "write", "delete", "approve", "fulfill"],
      "description": "Manage customer orders and fulfillment"
    },
    {
      "resource": "inventory",
      "actions": ["read", "write", "adjust", "transfer"],
      "description": "Manage stock levels and transfers"
    },
    {
      "resource": "integrations",
      "actions": ["read", "write", "delete", "sync"],
      "description": "Manage third-party integrations"
    },
    {
      "resource": "payments",
      "actions": ["read", "write", "refund"],
      "description": "Manage payment transactions"
    },
    {
      "resource": "reports",
      "actions": ["read", "export"],
      "description": "View and export analytics reports"
    },
    {
      "resource": "settings",
      "actions": ["read", "write"],
      "description": "Manage tenant settings and configuration"
    }
  ],
  "total": 8
}
```

---

## Common Patterns

### Creating a Sales Staff Role

```bash
curl -X POST http://localhost:3000/api/v1/admin/roles \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "role_name": "sales_staff",
    "description": "Sales team members",
    "permissions": [
      {"resource": "orders", "action": "read"},
      {"resource": "orders", "action": "write"},
      {"resource": "products", "action": "read"},
      {"resource": "customers", "action": "read"}
    ]
  }'
```

### Assigning Role to User

```bash
curl -X POST http://localhost:3000/api/v1/admin/users/${USER_ID}/roles/assign \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "role_name": "sales_staff"
  }'
```

### Viewing User's Permissions

```bash
# First get user's roles
curl http://localhost:3000/api/v1/admin/users/${USER_ID}/roles \
  -H "Authorization: Bearer ${ADMIN_TOKEN}"

# Then check each role's permissions
curl http://localhost:3000/api/v1/admin/roles \
  -H "Authorization: Bearer ${ADMIN_TOKEN}"
```

---

## Multi-Tenancy Notes

- All operations are scoped to the admin's tenant
- Roles and permissions are tenant-isolated
- User role assignments are tenant-specific
- An admin in Tenant A cannot see/modify roles in Tenant B

## Security Considerations

1. **Admin-Only Access**: All endpoints require `admin` role
2. **System Role Protection**: Cannot modify/delete `admin` and `user` roles
3. **Role Deletion Safety**: Cannot delete roles currently assigned to users
4. **User Safety**: Users must always have at least one role
5. **Tenant Isolation**: All operations are scoped to admin's tenant
6. **Audit Logging**: All role/permission changes should be logged (future enhancement)

## Error Codes

| Status | Code | Description |
|--------|------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid request format or data |
| 401 | `UNAUTHORIZED` | Missing or invalid JWT token |
| 403 | `FORBIDDEN` | Not admin or attempting restricted operation |
| 404 | `NOT_FOUND` | Role, user, or assignment not found |
| 409 | `CONFLICT` | Resource conflict (duplicate role, etc.) |
| 500 | `INTERNAL_ERROR` | Server error |

---

## Implementation Details

### Casbin Integration

Roles and permissions are stored in Casbin's policy table:

**Policy (p) format:**
```
[role, tenant_id, resource, action]
```

**Grouping (g) format:**
```
[user_id, role, tenant_id]
```

### Database Schema

Uses existing Casbin tables:
- `casbin_rule` table stores both policies and grouping policies
- No additional tables needed for basic role management
- Role descriptions currently stored in memory (future: add `roles` table)

---

## OpenAPI Documentation

Full OpenAPI/Swagger documentation available at:
- **Swagger UI**: `http://localhost:3000/docs`
- **OpenAPI JSON**: `http://localhost:3000/api-docs/openapi.json`
- **OpenAPI YAML**: `shared/openapi/user.yaml`

Tags:
- `admin-roles`: Role CRUD operations
- `admin-users`: User role assignment operations
- `admin-permissions`: Permission listing

---

## Testing

Example test flow:

```bash
# 1. Login as admin
TOKEN=$(curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"password"}' \
  | jq -r '.access_token')

# 2. List available permissions
curl http://localhost:3000/api/v1/admin/permissions \
  -H "Authorization: Bearer $TOKEN"

# 3. Create a custom role
curl -X POST http://localhost:3000/api/v1/admin/roles \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "role_name": "warehouse_manager",
    "permissions": [
      {"resource": "inventory", "action": "read"},
      {"resource": "inventory", "action": "write"},
      {"resource": "inventory", "action": "adjust"}
    ]
  }'

# 4. List all roles
curl http://localhost:3000/api/v1/admin/roles \
  -H "Authorization: Bearer $TOKEN"

# 5. Assign role to user
USER_ID="01939f4c-1234-7890-abcd-ef1234567890"
curl -X POST http://localhost:3000/api/v1/admin/users/$USER_ID/roles/assign \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"role_name": "warehouse_manager"}'

# 6. Get user's roles
curl http://localhost:3000/api/v1/admin/users/$USER_ID/roles \
  -H "Authorization: Bearer $TOKEN"
```

---

## Future Enhancements

- [ ] Add `roles` table to store role descriptions persistently
- [ ] Implement role hierarchies (role inheritance)
- [ ] Add audit logging for all role/permission changes
- [ ] Add bulk role assignment API
- [ ] Add role templates for common use cases
- [ ] Implement permission dependencies
- [ ] Add role search and filtering
- [ ] Add pagination for role listing

---

**Related Files:**
- DTOs: `services/user_service/core/src/domains/auth/dto/admin_dto.rs`
- Handlers: `services/user_service/api/src/admin_handlers.rs`
- Routing: `services/user_service/api/src/main.rs`
- OpenAPI: `services/user_service/api/src/openapi.rs`
