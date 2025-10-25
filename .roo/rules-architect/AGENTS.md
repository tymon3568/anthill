# .roo/rules-architect/AGENTS.md

## 🏗️ Project Architecture Rules (Non-Obvious Only)

### 🎯 Architecture Philosophy (Critical Context)

**Pragmatic Tool Selection**:
- **"Use the right tool for the job"** - Not "everything in Rust"
- **CapRover over Kubernetes** - Simplicity for small teams
- **Docker Swarm over service mesh** - Built-in networking
- **NGINX over Traefik** - CapRover managed gateway

**Performance-First Decisions**:
- **Application-level filtering over RLS** - Better performance for complex queries
- **UUID v7 over v4** - Timestamp-based for index locality
- **BIGINT cents over DECIMAL** - No floating-point rounding issues

### 🏢 Multi-Tenancy Architecture (Dual Layer)

**Database-Level Security**:
- **PostgreSQL RLS**: Automatic query filtering by tenant_id
- **Policy Creation**: Enable RLS on all tenant-specific tables
- **Backup**: RLS policies preserved in schema dumps

**Application-Level Authorization**:
- **Casbin RBAC**: Fine-grained permissions with shared/auth crate
- **Policy Storage**: casbin_rule table in PostgreSQL
- **Middleware**: JWT + permission validation in Axum

**Defense in Depth**:
- **Triple Protection**: RLS + Casbin + Manual tenant_id checks
- **Fail-Safe**: If one layer fails, others provide protection
- **Audit Trail**: All access logged with tenant_id context

### 🏭 Microservices Architecture (3-Crate Pattern)

**Service Structure** (user_service only currently):
```
services/user_service/
├── api/     # HTTP layer - handlers, routing, OpenAPI
├── core/    # Business logic - traits, entities, DTOs
└── infra/   # Infrastructure - database, external APIs
```

**Dependency Flow** (enforce strictly):
- **Direction**: `api → infra → core → shared/*`
- **No Cycles**: Prevent circular dependencies between crates
- **Testability**: Core crate has zero infrastructure dependencies

**Shared Libraries Purpose**:
- **Cross-Cutting**: Common functionality across all services
- **DRY Principle**: No code duplication between services
- **Versioning**: Workspace dependencies for consistency

### 🗄️ Database Architecture (Multi-Tenant)

**Schema Design**:
- **Shared Database**: Single PostgreSQL instance for all tenants
- **Tenant Isolation**: tenant_id column in every table
- **Composite Indexes**: `(tenant_id, other_columns)` for performance

**Query Patterns**:
- **Mandatory Filtering**: Every query includes `WHERE tenant_id = $1`
- **Repository Pattern**: Database access through trait implementations
- **Connection Pooling**: Shared pool configuration via shared/db

**Migration Strategy**:
- **SQLx Migrations**: Type-safe schema changes
- **Helper Script**: Use `scripts/migrate.sh` for all operations
- **No Downtime**: Forward-only migrations with careful planning

### 🔐 Security Architecture (Authentication & Authorization)

**JWT Strategy**:
- **Access Token**: 15 minutes expiry (configurable)
- **Refresh Token**: 7 days expiry with rotation
- **Claims**: user_id, tenant_id, role mandatory

**Password Security**:
- **Current**: bcrypt with strength validation (zxcvbn)
- **Future**: TODO migrate to Argon2id for better security
- **Context-Aware**: Validation against user email, name, tenant

**Session Management**:
- **Database Storage**: Session table with token hashes
- **IP Tracking**: Store client IP and User-Agent
- **Rotation**: Refresh tokens rotated on use

### 🌐 Service Communication Architecture

**Internal Communication**:
- **Overlay Network**: Docker Swarm srv- prefix (srv-inventory-svc:8000)
- **Direct Calls**: Service-to-service via internal DNS
- **No External**: Internal services not accessible from internet

**External Communication**:
- **API Gateway**: CapRover NGINX handles routing and SSL
- **Load Balancing**: Automatic across service instances
- **SSL Termination**: Let's Encrypt certificates automatic

**Event-Driven Communication** (Future):
- **NATS Server**: Message queue for inter-service events
- **Event Types**: inventory.updated, order.confirmed, payment.completed
- **Saga Pattern**: Orchestration for complex business processes

### 📊 Data Architecture (Inventory Domain)

**Product Master** (Future):
- **Single Source**: products table as truth source
- **Variants**: JSONB for product variations
- **UoM**: Unit of measure conversions and calculations

**Inventory Tracking** (Future):
- **Stock Ledger**: Immutable stock_moves table for audit trail
- **Lot/Serial**: Traceability with supplier linkage
- **Valuation**: FIFO/AVCO/Standard cost methods

**Warehouse Management** (Future):
- **Multi-Level**: Zone → Aisle → Shelf → Bin hierarchy
- **Virtual Locations**: Customer, Supplier, Transit, QC locations
- **Put-Away**: Intelligent storage location assignment

## ⚠️ Critical Architectural Constraints

**Implementation Order** (Must Follow):
1. **Complete user_service** - Only fully implemented service
2. **Refactor other services** - Apply 3-crate pattern to all services
3. **Implement inventory domain** - Core business logic first
4. **Add event-driven features** - NATS integration for async processing

**Scaling Considerations**:
- **Database**: Shard by tenant_id when growing beyond single instance
- **Services**: Horizontal scaling via CapRover app instances
- **Caching**: Redis for session storage and frequently accessed data

**Migration Constraints**:
- **No Downtime**: Plan migrations for maintenance windows
- **Data Preservation**: Backup before schema changes
- **Rollback Plan**: Though forward-only, have recovery procedures