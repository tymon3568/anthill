# Database Migrations

This directory contains SQL migrations for the Inventory SaaS Platform.

## Migration Tool

We use **sqlx-cli** for managing migrations.

### Installation

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

### Setup

1. Create `.env` file in project root with DATABASE_URL:
```bash
DATABASE_URL=postgres://user:password@localhost:5432/inventory_saas
```

2. Create database:
```bash
sqlx database create
```

### Migration Commands

```bash
# Create a new migration
sqlx migrate add <migration_name>

# Run all pending migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Check migration status
sqlx migrate info
```

## Migration Naming Convention

Format: `<timestamp>_<description>.sql`

Example:
- `20250110_001_initial_extensions.sql`
- `20250110_002_create_tenants_users.sql`
- `20250110_003_create_casbin_tables.sql`

## Migration Guidelines

### 1. Multi-Tenancy
- All tenant-specific tables MUST have `tenant_id UUID NOT NULL`
- Create composite indexes: `(tenant_id, <other_columns>)`
- Use application-level filtering (no Postgres RLS)

### 2. UUID Standard
- Use UUID v7 for all primary keys (timestamp-based)
- Better index locality and performance

### 3. Timestamps
- Use `TIMESTAMPTZ` (timezone-aware)
- Standard columns: `created_at`, `updated_at`, `deleted_at`
- Set defaults: `DEFAULT NOW()`

### 4. Money/Currency
- Store as `BIGINT` (smallest unit: cents, xu)
- Example: $10.50 → 1050, 100.000 VND → 100000
- No floating-point types for money!

### 5. Soft Delete
- Add `deleted_at TIMESTAMPTZ` to important tables
- Create partial index: `WHERE deleted_at IS NULL`

### 6. Foreign Keys
- Include `tenant_id` in composite foreign keys
- Example: `FOREIGN KEY (tenant_id, product_id) REFERENCES products(tenant_id, product_id)`

## Migration Structure

### Phase 1: Foundation
1. Extensions and functions (uuid-ossp, pgcrypto, uuid_generate_v7)
2. Core tables (tenants, users, sessions)
3. Casbin RBAC tables (casbin_rule)

### Phase 2: Business Logic
4. Products and inventory
5. Warehouses
6. Orders and order items
7. Integrations
8. Payments

### Phase 3: Optimization
9. Additional indexes
10. Performance optimizations

## Testing Migrations

Before applying to production:

1. Test on local database
2. Backup production database
3. Test migration in staging environment
4. Review all indexes and constraints
5. Check query performance

## Rollback Strategy

- Each migration should be reversible
- Test rollback on staging before production
- Keep backups before major migrations
