# OpenAPI Specifications

⚠️ **DO NOT EDIT BY HAND** ⚠️

This directory contains auto-generated OpenAPI specifications from the backend services.

## How It Works

1. Each backend service (user-service, inventory-service, etc.) defines its API using `utoipa` annotations
2. During build with `--features export-spec`, each service exports its spec to `<service-name>.yaml`
3. CI/CD pipeline merges all service specs into `api.yaml`
4. Frontend (SvelteKit) auto-generates type-safe SDK from `api.yaml`

## Files

- `user.yaml` - User service API (auth, tenants, users)
- `inventory.yaml` - Inventory service API (products, stock, warehouses)
- `order.yaml` - Order service API (orders, fulfillment)
- `payment.yaml` - Payment service API (payments, gateways)
- `integration.yaml` - Integration service API (marketplace sync)
- `api.yaml` - **Merged final spec** (used by frontend)

## Local Development

To regenerate specs locally:

```bash
# Generate spec for a specific service
cd services/user-service
cargo build --features export-spec

# Or generate all services
cd /home/arch/anthill
for service in services/*/; do
  (cd "$service" && cargo build --features export-spec)
done
```

## CI/CD Pipeline

On every push to `services/**`, GitHub Actions will:
1. Build each service with `--features export-spec`
2. Use `@redocly/cli` to bundle all specs into `api.yaml`
3. Commit changes back to repo
4. Trigger frontend SDK regeneration

## Viewing Docs

Each service exposes Swagger UI during development:

- User Service: http://localhost:3000/docs
- Inventory Service: http://localhost:3001/docs
- Order Service: http://localhost:3002/docs
- Integration Service: http://localhost:3003/docs
- Payment Service: http://localhost:3004/docs

## Troubleshooting

**Spec not updating?**
- Make sure you're building with `--features export-spec`
- Check that `[features]` section exists in `Cargo.toml`
- Verify `src/openapi.rs` is properly configured

**Duplicate operationId error?**
- Each endpoint must have unique `operationId` across all services
- Use prefix convention: `user_login`, `inventory_list_products`, etc.

**Frontend types out of sync?**
- Ensure CI completed successfully
- Check that `api.yaml` was committed
- Run `pnpm orval` in frontend directory
