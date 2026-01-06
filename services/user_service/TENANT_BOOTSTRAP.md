# Tenant Bootstrap & Owner Role Assignment

## Overview

This document describes the tenant bootstrap behavior during user registration in the Anthill multi-tenant inventory platform. When a user registers, the system determines whether to create a new tenant or join an existing one, and assigns the appropriate role accordingly.

## Core Concepts

### Option D: Single Role Per User

Anthill implements **Option D** from the RBAC strategy:
- Each user has exactly **one effective role** stored in `users.role`
- System roles (`owner`, `admin`, `user`) are protected from deletion/modification
- Casbin grouping policies maintain consistency with `users.role` for authorization enforcement

### System Roles

| Role | Description | Can Be Assigned By |
|------|-------------|-------------------|
| `owner` | Tenant creator with full privileges | System only (during registration) |
| `admin` | Full tenant administration access | Owner |
| `user` | Standard user with limited access | Owner, Admin |

### Tenant Ownership

- Each tenant has exactly **one owner** (`tenants.owner_user_id`)
- Owner is assigned automatically during tenant creation
- Ownership transfer requires explicit API call (future feature)

---

## Registration Bootstrap Rules

### Case A: New Tenant Creation

When a user registers with a **new tenant name** (tenant does not exist):

1. **Create Tenant**
   - Generate tenant with provided name
   - Generate slug from name (e.g., "Acme Corp" → "acme-corp")
   - Set default plan ("free") and status ("active")

2. **Create User**
   - Store user with `role = "owner"`
   - Hash password with bcrypt
   - Set `email_verified = false` (pending verification)

3. **Set Tenant Ownership**
   - Update `tenants.owner_user_id` to point to the new user
   - This establishes the ownership record

4. **Create Casbin Grouping**
   - Add grouping policy: `(user_id, "owner", tenant_id)`
   - Owner policies from seed migration apply automatically

5. **Issue Tokens**
   - Generate JWT access token with `role: "owner"`
   - Generate refresh token and session

### Case B: Join Existing Tenant

When a user registers and the tenant **already exists** (matched by slug):

1. **Resolve Tenant**
   - Find existing tenant by slug

2. **Create User**
   - Store user with `role = "user"` (default role)
   - Hash password with bcrypt
   - Set `email_verified = false`

3. **Create Casbin Grouping**
   - Add grouping policy: `(user_id, "user", tenant_id)`
   - User policies from seed migration apply automatically

4. **Issue Tokens**
   - Generate JWT access token with `role: "user"`
   - Generate refresh token and session

---

## API Reference

### Register Endpoint

```http
POST /api/v1/auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "SecurePass123!",
  "full_name": "John Doe",
  "tenant_name": "Acme Corp"
}
```

#### Request Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `email` | string | Yes | Valid email address |
| `password` | string | Yes | Min 8 characters |
| `full_name` | string | Yes | User's display name |
| `tenant_name` | string | No | Name for new/existing tenant |

#### Response (201 Created)

```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 900,
  "user": {
    "id": "01939f4c-1234-7890-abcd-ef1234567890",
    "email": "user@example.com",
    "full_name": "John Doe",
    "tenant_id": "01939f4c-0987-7890-abcd-ef0987654321",
    "role": "owner",
    "created_at": "2026-01-06T10:30:00Z"
  }
}
```

The `role` field indicates the bootstrap result:
- `"owner"` - User created a new tenant and is the owner
- `"user"` - User joined an existing tenant with default role

---

## Database Schema

### Tenants Table (Relevant Columns)

```sql
CREATE TABLE tenants (
    tenant_id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) UNIQUE NOT NULL,
    owner_user_id UUID REFERENCES users(user_id) ON DELETE SET NULL,
    -- other columns...
);
```

### Users Table (Role Constraint)

```sql
ALTER TABLE users ADD CONSTRAINT users_role_check
    CHECK (role IN ('owner', 'super_admin', 'admin', 'manager', 'user', 'viewer'));
```

### Casbin Grouping

The registration handler adds a grouping policy:

```sql
INSERT INTO casbin_rule (ptype, v0, v1, v2)
VALUES ('g', '{user_id}', '{role}', '{tenant_id}');
```

---

## Owner Role Policies

The owner role has a superset of admin permissions plus tenant-level management:

### Owner-Only Permissions

| Resource | Actions | Description |
|----------|---------|-------------|
| `/api/v1/tenant` | GET, PUT | View/update tenant settings |
| `/api/v1/tenant/settings` | GET, PUT | Manage tenant configuration |
| `/api/v1/tenant/billing` | GET, PUT | Manage billing information |
| `/api/v1/tenant/plan` | GET, PUT | Manage subscription plan |
| `/api/v1/tenant/danger/*` | POST | Dangerous operations |
| `/api/v1/tenant/export` | GET | Export tenant data |
| `/api/v1/tenant/delete` | POST | Delete tenant (irreversible) |

### Inherited from Admin

- Full user management (`/api/v1/users/*`)
- Role and policy management (`/api/v1/admin/roles/*`, `/api/v1/admin/policies/*`)
- All inventory, orders, and integration management

---

## Security Considerations

### Owner Protection

1. **Cannot Demote Self**: Owner cannot change their own role
2. **Cannot Be Deleted**: Owner account cannot be deleted without ownership transfer
3. **Single Owner**: Only one owner per tenant (enforced by application logic)
4. **System Role**: Owner role cannot be assigned via admin APIs

### Multi-Tenancy Isolation

- Registration resolves tenant by slug only within active tenants
- User-tenant relationship is immutable after creation
- Cross-tenant access is prevented at repository layer

### Bootstrap Security

- Password is hashed with bcrypt before storage
- JWT tokens are issued with correct tenant_id claim
- Casbin grouping ensures authorization consistency

---

## Error Handling

### Registration Errors

| Status | Code | Description |
|--------|------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid email/password format |
| 409 | `USER_ALREADY_EXISTS` | Email already registered in tenant |
| 500 | `INTERNAL_ERROR` | Database or system error |

### Casbin Errors

If Casbin grouping fails during registration:
- Registration succeeds (user is created)
- Error is logged for monitoring
- Casbin grouping can be fixed via admin APIs

---

## Testing

### New Tenant Registration

```bash
# Register as new tenant owner
curl -X POST http://localhost:8000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "owner@newcompany.com",
    "password": "SecurePass123!",
    "full_name": "Tenant Owner",
    "tenant_name": "New Company Inc"
  }'

# Expected: role = "owner" in response
```

### Join Existing Tenant

```bash
# Register to join existing tenant
curl -X POST http://localhost:8000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "employee@newcompany.com",
    "password": "SecurePass123!",
    "full_name": "New Employee",
    "tenant_name": "New Company Inc"
  }'

# Expected: role = "user" in response (tenant already exists)
```

### Verify Tenant Ownership

```sql
-- Check tenant owner
SELECT t.name, t.owner_user_id, u.email, u.role
FROM tenants t
JOIN users u ON t.owner_user_id = u.user_id
WHERE t.slug = 'new-company-inc';
```

### Verify Casbin Grouping

```sql
-- Check user's Casbin grouping
SELECT ptype, v0 AS user_id, v1 AS role, v2 AS tenant_id
FROM casbin_rule
WHERE ptype = 'g' AND v0 = '{user_id}';
```

---

## Related Files

- **Migration (Schema)**: `migrations/20260106000001_add_tenant_owner_and_owner_role.sql`
- **Migration (Policies)**: `migrations/20260106000002_seed_owner_role_policies.sql`
- **Handler**: `services/user_service/api/src/handlers.rs` (`register` function)
- **Service**: `services/user_service/infra/src/auth/service.rs` (`AuthServiceImpl::register`)
- **Repository**: `services/user_service/infra/src/auth/repository.rs` (`TenantRepository::set_owner`)
- **Task**: `PROJECT_TRACKING/V1_MVP/03_User_Service/3.3_User_Management/task_03.03.06_register_bootstrap_owner_and_default_role.md`

---

## Future Enhancements

- [ ] Ownership transfer API
- [ ] Multi-owner support (co-owners)
- [ ] Owner activity audit logging
- [ ] Owner-only action confirmations (2FA)
- [ ] Tenant deletion cooldown period
