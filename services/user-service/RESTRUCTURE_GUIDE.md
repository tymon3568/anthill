# User Service Restructure Guide

## 🎯 Goal: Migrate to Clean Axum Demo Architecture

**Status**: 🔄 IN PROGRESS

**Template**: https://github.com/sukjaelee/clean_axum_demo

## ✅ Completed

1. ✅ Backed up existing code to `.backup/`
2. ✅ Created directory structure:
   - `common/` - Cross-cutting concerns
   - `domains/auth/` - Auth domain with api/, domain/, dto/, infra/
3. ✅ Created `common/error.rs` - AppError with IntoResponse

## 🔄 TODO

### Common Layer (Next Priority)

- [ ] `common/config.rs` - Environment config loader
- [ ] `common/app_state.rs` - AppState with DB pool, config, JWT secret
- [ ] `common/bootstrap.rs` - Service initialization
- [ ] `common/jwt.rs` - JWT encode/decode
- [ ] `common/hash_util.rs` - Argon2 password hashing

### Auth Domain

- [ ] Migrate `.backup/models.rs` → `domains/auth/dto/auth_dto.rs`
- [ ] Migrate `.backup/handlers.rs` → `domains/auth/api/handlers.rs`
- [ ] Create `domains/auth/api/routes.rs`
- [ ] Create `domains/auth/domain/model.rs` - User entity
- [ ] Create `domains/auth/domain/repository.rs` - Trait
- [ ] Create `domains/auth/domain/service.rs` - Trait
- [ ] Create `domains/auth/infra/impl_repository.rs`
- [ ] Create `domains/auth/infra/impl_service.rs`
- [ ] Create `domains/auth/auth.rs` - Module entry

### App Setup

- [ ] Refactor `main.rs` - Bootstrap pattern
- [ ] Create `app.rs` - Router + middleware setup
- [ ] Update `lib.rs` - Module declarations
- [ ] Create `common.rs` - Common module entry
- [ ] Create `domains.rs` - Domains module entry

### Dependencies

Add to `Cargo.toml`:
```toml
bcrypt = "0.15"                    # Password hashing
jsonwebtoken = "9.2"               # JWT
validator = { version = "0.16", features = ["derive"] }
dotenvy = "0.15"                   # .env loader
async-trait = "0.1"                # Async traits
```

### OpenAPI Integration

- [ ] Move `openapi.rs` to `common/openapi.rs`
- [ ] Update OpenAPI to reference new domain structure
- [ ] Keep export-spec feature working

## 📋 Migration Checklist

### Files to Migrate

From `.backup/`:
- `handlers.rs` → `domains/auth/api/handlers.rs` (refactor to use service layer)
- `models.rs` → `domains/auth/dto/auth_dto.rs` (keep DTOs only)
- `openapi.rs` → Keep at root or move to `common/`

### New Files Needed

According to Clean Axum Demo template:

**common/**
- [x] error.rs
- [ ] config.rs
- [ ] app_state.rs
- [ ] bootstrap.rs
- [ ] jwt.rs
- [ ] hash_util.rs

**domains/auth/api/**
- [ ] handlers.rs
- [ ] routes.rs

**domains/auth/domain/**
- [ ] model.rs (User, Tenant entities)
- [ ] repository.rs (trait)
- [ ] service.rs (trait)

**domains/auth/dto/**
- [ ] auth_dto.rs (RegisterReq, LoginReq, AuthResp, etc.)

**domains/auth/infra/**
- [ ] impl_repository.rs
- [ ] impl_service.rs

**Root:**
- [ ] app.rs
- [ ] common.rs
- [ ] domains.rs

## 🚀 Build & Test

```bash
# After restructure is complete:
cargo build

# With OpenAPI export:
cargo build --features export-spec

# Run:
cargo run

# Test:
cargo test
```

## 📚 References

- **Template**: https://github.com/sukjaelee/clean_axum_demo
- **WARP.md**: `/home/arch/anthill/WARP.md` - Section "Axum Production Best Practices"
- **Backup**: `.backup/` - Original implementation for reference

## ⚠️ Important Notes

1. **Keep OpenAPI working**: Don't break Swagger UI at `/docs`
2. **Keep export-spec feature**: Must still generate `shared/openapi/user.yaml`
3. **Multi-tenant**: All queries must filter by `tenant_id`
4. **AppError**: Use throughout, never panic in handlers
5. **Dependency Injection**: Use AppState for DB pool, config, etc.

## 🎯 Success Criteria

- ✅ Clean Architecture layers: API → Domain → Infrastructure
- ✅ Dependency injection with AppState
- ✅ Proper error handling with AppError
- ✅ Swagger UI still works at `/docs`
- ✅ OpenAPI export still works
- ✅ No business logic in handlers
- ✅ Repository pattern for DB access
- ✅ Service layer for business logic
- ✅ Tests pass

## Next Steps

1. Continue with `common/config.rs`
2. Then `common/app_state.rs`
3. Then `common/bootstrap.rs`
4. Migrate domain layer
5. Update main.rs and app.rs
6. Test everything
7. Apply same pattern to other services
