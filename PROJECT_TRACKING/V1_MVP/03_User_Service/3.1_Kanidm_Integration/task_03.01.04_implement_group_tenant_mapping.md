# Task: Implement Kanidm Group to Tenant Mapping

**Task ID:** V1_MVP/03_User_Service/3.1_Kanidm_Integration/task_03.01.04_implement_group_tenant_mapping.md  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.1_Kanidm_Integration  
**Priority:** High  
**Status:** Done  
**Assignee:**  
**Created Date:** 2025-11-03  
**Last Updated:** 2025-11-03

## Detailed Description

Implement the mapping system between Kanidm groups and Anthill tenants to enable multi-tenancy. Users will be assigned to Kanidm groups, which will be mapped to tenant_id in PostgreSQL.

## Specific Sub-tasks

- [x] 1. Create database migration for `kanidm_tenant_groups` table
- [x] 2. Create repository trait for tenant group mapping
- [x] 3. Implement PostgreSQL repository for mapping
- [x] 4. Create service methods for:
  - [x] Map Kanidm group to tenant
  - [x] Get tenant from Kanidm groups (from JWT claims)
  - [x] List all groups for a tenant
  - [x] Sync group membership from Kanidm
- [x] 5. Update user creation logic to link kanidm_user_id
- [x] 6. Implement group extraction from Kanidm JWT
- [x] 7. Create admin API endpoints for managing mappings
- [x] 8. Write unit and integration tests

## Acceptance Criteria

- [x] `kanidm_tenant_groups` table created in database
- [x] Can create mapping: Kanidm group UUID → tenant_id
- [x] Can resolve tenant_id from JWT groups claim
- [x] User record includes kanidm_user_id
- [x] Admin can manage group-tenant mappings via API
- [x] Multiple groups can map to same tenant (e.g., users + admins)
- [x] Tests verify correct tenant isolation

## Dependencies

- Task 03.01.01 (Kanidm server with groups)
- Task 03.01.02 (KanidmClient for JWT validation)
- Task 03.01.03 (OAuth2 callback needs this)
- Migration 20250110000002 (users table)

## Files to Create/Modify

### Database Migration
```sql
-- Migration: 20250111000001_create_kanidm_tenant_groups.sql
CREATE TABLE kanidm_tenant_groups (
  tenant_id UUID NOT NULL REFERENCES tenants(tenant_id) ON DELETE CASCADE,
  kanidm_group_uuid UUID NOT NULL,
  kanidm_group_name TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (tenant_id, kanidm_group_uuid)
);

CREATE INDEX idx_kanidm_tenant_groups_name ON kanidm_tenant_groups(kanidm_group_name);

-- Modify users table
ALTER TABLE users 
  ADD COLUMN kanidm_user_id UUID UNIQUE,
  ADD COLUMN kanidm_synced_at TIMESTAMPTZ;

-- Add comment
COMMENT ON COLUMN users.kanidm_user_id IS 'User UUID from Kanidm (sub claim in JWT)';
```

### Core Domain
```
services/user_service/core/src/domains/
├── tenant/
│   ├── domain/
│   │   ├── model.rs          # Add TenantGroupMapping entity
│   │   └── repository.rs     # Add TenantGroupMappingRepository trait
│   └── dto/
│       └── tenant_dto.rs     # Add DTOs for mapping
└── auth/
    └── domain/
        └── service.rs        # Add resolve_tenant_from_groups() method
```

### Implementation Files
```
services/user_service/infra/src/
├── tenant/
│   └── mapping_repository.rs  # PostgreSQL implementation
└── auth/
    └── service.rs            # Implement tenant resolution
```

## Code Examples

### Domain Model
```rust
// Core domain model
#[derive(Debug, Clone)]
pub struct TenantGroupMapping {
    pub tenant_id: Uuid,
    pub kanidm_group_uuid: Uuid,
    pub kanidm_group_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Repository trait
#[async_trait]
pub trait TenantGroupMappingRepository: Send + Sync {
    async fn create(&self, mapping: &TenantGroupMapping) -> Result<(), AppError>;
    async fn find_tenant_by_group_name(&self, group_name: &str) -> Result<Option<Uuid>, AppError>;
    async fn find_tenant_by_group_uuid(&self, group_uuid: &Uuid) -> Result<Option<Uuid>, AppError>;
    async fn list_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<TenantGroupMapping>, AppError>;
    async fn delete(&self, tenant_id: Uuid, group_uuid: Uuid) -> Result<(), AppError>;
}
```

### Tenant Resolution Logic
```rust
impl AuthService {
    /// Resolve tenant_id from Kanidm JWT groups claim
    pub async fn resolve_tenant_from_groups(
        &self,
        groups: &[String],
    ) -> Result<Uuid, AppError> {
        // Try each group in order until we find a tenant mapping
        for group_name in groups {
            if let Some(tenant_id) = self
                .mapping_repo
                .find_tenant_by_group_name(group_name)
                .await?
            {
                return Ok(tenant_id);
            }
        }
        
        Err(AppError::TenantNotFound(
            "No tenant mapping found for user groups".to_string()
        ))
    }
    
    /// Handle OAuth callback with tenant mapping
    pub async fn handle_oauth_callback(
        &self,
        code: String,
        pkce_verifier: String,
    ) -> Result<OAuth2CallbackResp, AppError> {
        // 1. Exchange code for tokens
        let tokens = self.kanidm_client.exchange_code(&code, pkce_verifier).await?;
        
        // 2. Validate JWT and extract claims
        let claims = self.kanidm_client.validate_token(&tokens.access_token).await?;
        
        // 3. Resolve tenant from groups
        let tenant_id = self.resolve_tenant_from_groups(&claims.groups).await?;
        
        // 4. Create or update user
        let user = self.upsert_user_from_kanidm(&claims, tenant_id).await?;
        
        // 5. Return response
        Ok(OAuth2CallbackResp {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in: tokens.expires_in,
            token_type: "Bearer".to_string(),
            user: UserInfo::from(user),
        })
    }
    
    /// Create or update user from Kanidm claims
    async fn upsert_user_from_kanidm(
        &self,
        claims: &KanidmClaims,
        tenant_id: Uuid,
    ) -> Result<User, AppError> {
        let kanidm_user_id = Uuid::parse_str(&claims.sub)?;
        
        // Check if user exists
        if let Some(mut user) = self.user_repo.find_by_kanidm_id(kanidm_user_id).await? {
            // Update user info from Kanidm
            user.email = claims.email.clone();
            user.username = claims.preferred_username.clone();
            user.kanidm_synced_at = Some(Utc::now());
            self.user_repo.update(&user).await?;
            Ok(user)
        } else {
            // Create new user
            let user = User {
                user_id: Uuid::now_v7(),
                tenant_id,
                kanidm_user_id: Some(kanidm_user_id),
                email: claims.email.clone(),
                username: claims.preferred_username.clone(),
                role: "user".to_string(), // Default role
                password_hash: None, // No password, auth via Kanidm
                created_at: Utc::now(),
                updated_at: Utc::now(),
                kanidm_synced_at: Some(Utc::now()),
            };
            self.user_repo.create(&user).await?;
            Ok(user)
        }
    }
}
```

### Admin API for Managing Mappings
```rust
/// Create group-tenant mapping
#[utoipa::path(
    post,
    path = "/api/v1/admin/tenant-groups",
    tag = "admin",
    operation_id = "create_tenant_group_mapping",
    request_body = CreateTenantGroupMappingReq,
    responses(
        (status = 201, body = TenantGroupMappingResp),
        (status = 403, body = ErrorResp),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_tenant_group_mapping(
    RequireAdmin(user): RequireAdmin,
    State(state): State<AppState>,
    Json(payload): Json<CreateTenantGroupMappingReq>,
) -> Result<(StatusCode, Json<TenantGroupMappingResp>), AppError> {
    let mapping = TenantGroupMapping {
        tenant_id: payload.tenant_id,
        kanidm_group_uuid: payload.kanidm_group_uuid,
        kanidm_group_name: payload.kanidm_group_name,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    state.mapping_repo.create(&mapping).await?;
    
    Ok((StatusCode::CREATED, Json(TenantGroupMappingResp::from(mapping))))
}
```

## Testing Steps

```bash
# 1. Run migration
sqlx migrate run

# 2. Create Kanidm groups
docker-compose exec kanidm kanidm group create tenant_acme_users
docker-compose exec kanidm kanidm group create tenant_acme_admins
docker-compose exec kanidm kanidm group create tenant_globex_users

# 3. Create tenant-group mappings via API
curl -X POST http://localhost:3000/api/v1/admin/tenant-groups \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "uuid-of-acme-tenant",
    "kanidm_group_uuid": "uuid-from-kanidm",
    "kanidm_group_name": "tenant_acme_users"
  }'

# 4. Add user to Kanidm group
docker-compose exec kanidm kanidm group add-members tenant_acme_users alice@example.com

# 5. Test OAuth login as alice
# Verify tenant_id is correctly resolved from groups

# 6. Run integration tests
cargo test --package user_service_infra --test tenant_mapping_tests
```

## Kanidm Group Setup Commands

```bash
# Example: Setup groups for tenant "Acme Corp"
TENANT_NAME="acme"

# 1. Create groups in Kanidm
kanidm group create tenant_${TENANT_NAME}_users
kanidm group create tenant_${TENANT_NAME}_admins

# 2. Set display names
kanidm group set displayname tenant_${TENANT_NAME}_users "Acme Corp Users"
kanidm group set displayname tenant_${TENANT_NAME}_admins "Acme Corp Admins"

# 3. Get group UUIDs
kanidm group get tenant_${TENANT_NAME}_users

# 4. Update OAuth scope mapping (so groups appear in JWT)
kanidm system oauth2 update-scope-map anthill tenant_${TENANT_NAME}_users email openid profile groups
kanidm system oauth2 update-scope-map anthill tenant_${TENANT_NAME}_admins email openid profile groups

# 5. Add test user
kanidm person create testuser "Test User" testuser@acme.com
kanidm group add-members tenant_${TENANT_NAME}_users testuser
```

## References

- Kanidm Groups: https://kanidm.github.io/kanidm/master/accounts_and_groups.html
- OAuth2 Scope Mapping: https://kanidm.github.io/kanidm/master/integrations/oauth2.html#scope-maps

## Notes

- Group naming convention: `tenant_<slug>_<role>` (e.g., `tenant_acme_users`, `tenant_acme_admins`)
- One user can belong to multiple groups (multiple tenants or multiple roles in same tenant)
- For multi-role scenarios, use Casbin policies to differentiate permissions within tenant
- Consider caching group→tenant mappings (Redis) for performance
- Handle case where user has no groups (should fail gracefully with clear error)

## AI Agent Log:
---
*   2025-11-05 10:40: Task status updated by Claude
    - Verified completion from Kanidm migration Phase 3
    - Database migration and tenant mapping logic implemented
    - Status: Done ✓
