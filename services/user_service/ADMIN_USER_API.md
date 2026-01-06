# Admin User Management API

This document describes the admin-only endpoints for user management in the Anthill User Service.

## Overview

Admin user management endpoints allow tenant administrators to create and manage users within their tenant. All endpoints enforce:

- **Authentication**: Valid JWT token required
- **Authorization**: Admin role required (enforced via Casbin)
- **Tenant Isolation**: All operations are scoped to the admin's tenant

## Endpoints

### Create User

Creates a new user in the admin's tenant.

**Endpoint:** `POST /api/v1/admin/users`

**Authorization:** Admin only

**Request Headers:**
```
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**Request Body:**
```json
{
  "email": "newuser@example.com",
  "password": "SecurePass123!",
  "full_name": "Jane Smith",
  "role": "user"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `email` | string | Yes | Valid email address (unique within tenant) |
| `password` | string | Yes | Min 8 characters, validated for strength |
| `full_name` | string | No | User's display name (max 255 chars) |
| `role` | string | No | Role to assign (default: "user") |

**Response (201 Created):**
```json
{
  "user_id": "01924a3b-4c5d-7e8f-9012-3456789abcde",
  "tenant_id": "01924a3b-1234-7e8f-9012-3456789abcde",
  "email": "newuser@example.com",
  "full_name": "Jane Smith",
  "role": "user",
  "created_at": "2026-01-06T12:00:00Z",
  "message": "User created successfully"
}
```

**Error Responses:**

| Status | Description |
|--------|-------------|
| 400 | Invalid request (email format, password too short, invalid role format) |
| 401 | Unauthorized - Missing or invalid JWT |
| 403 | Forbidden - Non-admin user or attempting to create owner role |
| 409 | Conflict - Email already exists in tenant |

## Role Assignment Rules

### System Roles

The following system roles are predefined:

| Role | Description |
|------|-------------|
| `owner` | Tenant owner (cannot be assigned via this endpoint) |
| `admin` | Full administrative access |
| `user` | Standard user access |

### Role Restrictions

1. **Owner Role**: Cannot be created via admin endpoints. Owner is only assigned during tenant bootstrap (registration).

2. **Single Role Per User**: Each user has exactly one role (Option D pattern). The role is stored in `users.role` and a corresponding Casbin grouping policy is created.

3. **Custom Roles**: Admin can create users with custom roles if those roles have been defined via the Role Management API (`POST /api/v1/admin/roles`).

### Role Name Format

Role names must follow this format:
- Lowercase letters and numbers only
- Underscores allowed as separators
- Must start with a letter
- Maximum 100 characters

Valid examples: `user`, `admin`, `inventory_manager`, `sales_staff`

Invalid examples: `Admin`, `SUPER-USER`, `123role`, `role@special`

## Security Considerations

### Password Requirements

Passwords are validated using zxcvbn-based strength checking:
- Minimum 8 characters
- Checked against common patterns
- Checked for similarity to email and full_name
- Hashed with bcrypt before storage

### Audit Logging

All admin user creation operations are logged with:
- Admin user ID
- Tenant ID
- Created user's email and role
- Timestamp

No sensitive data (passwords) is logged.

### Tenant Isolation

- Created users are always scoped to the admin's tenant (from JWT)
- Admin cannot create users in other tenants
- Email uniqueness is enforced per-tenant (same email can exist in different tenants)

## Examples

### Create a Basic User

```bash
curl -X POST http://localhost:8000/api/v1/admin/users \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "newuser@example.com",
    "password": "SecurePass123!"
  }'
```

### Create an Admin User

```bash
curl -X POST http://localhost:8000/api/v1/admin/users \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "newadmin@example.com",
    "password": "SecurePass123!",
    "full_name": "New Admin",
    "role": "admin"
  }'
```

### Create User with Custom Role

First, ensure the custom role exists:

```bash
# Create custom role
curl -X POST http://localhost:8000/api/v1/admin/roles \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "role_name": "inventory_manager",
    "description": "Can manage inventory",
    "permissions": [
      {"resource": "inventory", "action": "read"},
      {"resource": "inventory", "action": "write"}
    ]
  }'

# Create user with custom role
curl -X POST http://localhost:8000/api/v1/admin/users \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "inventory@example.com",
    "password": "SecurePass123!",
    "full_name": "Inventory Manager",
    "role": "inventory_manager"
  }'
```

## Related Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /api/v1/admin/users` | List users in tenant |
| `POST /api/v1/admin/users/{user_id}/roles/assign` | Change user's role |
| `DELETE /api/v1/admin/users/{user_id}/roles/{role_name}/remove` | Remove role from user |
| `GET /api/v1/admin/users/{user_id}/roles` | Get user's current role |

## OpenAPI Specification

The full OpenAPI specification is available at:
- JSON: `GET /api-docs/openapi.json`
- Swagger UI: `GET /swagger-ui/`

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-01-06 | Initial implementation |
