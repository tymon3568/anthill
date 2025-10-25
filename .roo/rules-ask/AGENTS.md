# .roo/rules-ask/AGENTS.md

## ❓ Project Documentation Rules (Non-Obvious Only)

### 📚 Documentation Hierarchy (Critical Context)

**Primary Sources** (Always reference in order):
1. **ARCHITECTURE.md** - System design and deployment decisions
2. **STRUCTURE.md** - Project structure and 3-crate pattern explanation
3. **WARP.md** - Development guide and current status
4. **TODO.md** - Comprehensive task breakdown with priorities
5. **Cargo.toml** - Workspace configuration and dependencies

**Secondary Sources**:
- Database ERD: `docs/database-erd.dbml` (visualize at dbdiagram.io)
- Migration files: `migrations/` (SQL schema definitions)
- OpenAPI specs: `shared/openapi/` (after generation)

### 🔍 Key Project Facts (Essential Context)

**Current Status Reality**:
- **Phase 3**: User Service Production Ready (~30% complete)
- **Only user_service follows 3-crate pattern** (others need refactoring)
- **Multi-tenant**: Dual approach (RLS + Casbin) - both database and application level
- **Database**: Schema exists but services need implementation

**Architecture Decisions** (Non-obvious):
- **Against Postgres RLS only**: Uses application-level filtering for performance
- **Against K8s**: Uses CapRover + Docker Swarm for simplicity
- **Against service mesh**: Uses Docker Swarm overlay network
- **Against floating point**: Uses BIGINT cents for money

### 🏗️ Architecture Context (For Technical Questions)

**3-Crate Pattern** (user_service only):
```
services/user_service/
├── api/     # HTTP layer (Axum handlers, routing, OpenAPI)
├── core/    # Business logic (traits, entities, DTOs only)
└── infra/   # Infrastructure (PostgreSQL impl, services)
```

**Shared Libraries Purpose**:
- `shared/error` - AppError with IntoResponse for Axum
- `shared/jwt` - JWT encoding/decoding with Claims struct
- `shared/config` - Environment configuration loading
- `shared/types` - Common types (Uuid, DateTime, Money)
- `shared/db` - Database pool initialization
- `shared/auth` - Casbin RBAC with multi-tenant support

**Multi-Tenancy Implementation**:
- **Database**: RLS policies + manual tenant_id filters
- **Application**: Casbin RBAC with tenant-scoped permissions
- **JWT**: tenant_id claim required in all tokens
- **Repository**: Every query includes `WHERE tenant_id = $1`

### 💰 Business Logic Context (For Feature Questions)

**Inventory Management** (Future):
- **Products**: Item master with SKU, UoM, variants
- **Warehouses**: Multi-level storage locations
- **Stock tracking**: Lot/serial number traceability
- **Operations**: GRN, DO, transfers, stock takes

**Order Processing** (Future):
- **Event-driven**: NATS for inter-service communication
- **Stock reservation**: Automatic inventory allocation
- **Fulfillment**: Integration with warehouse operations

**Marketplace Integration** (Future):
- **Adapters**: Shopee, Lazada, Tiki APIs
- **Sync**: Products and orders bidirectional
- **Webhooks**: Real-time event handling

### 🚀 Development Context (For Implementation Questions)

**Environment Setup**:
- **Docker Compose**: Local development environment
- **Environment Variables**: Must load .env file in project root
- **Database**: PostgreSQL with UUID v7 and custom functions

**Testing Strategy**:
- **Unit tests**: Core crate business logic (no database)
- **Integration tests**: API endpoints (require database)
- **Tenant isolation**: Must test cross-tenant data access prevention

**Deployment Context**:
- **CapRover**: Production PaaS with automatic deployments
- **One-Click Apps**: PostgreSQL, Redis, NATS, Cube
- **Service Naming**: `srv-` prefix for overlay network