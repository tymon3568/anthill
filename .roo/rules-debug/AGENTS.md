# .roo/rules-debug/AGENTS.md

## 🪲 Project Debug Rules (Non-Obvious Only)

### 🏗️ Architecture Debugging (Critical)

**Service Communication Issues**:
- **srv- prefix**: Use `srv-inventory-svc:8000` for internal service calls, not external hostnames
- **Docker Swarm**: Services run in overlay network, not accessible via localhost
- **Port conflicts**: Each service needs unique port in docker-compose.yml

**3-Crate Pattern Debugging**:
- **Compilation errors**: Check dependency flow `api → infra → core → shared/*`
- **Circular dependencies**: Verify imports don't create cycles between crates
- **Test failures**: Unit tests in core don't need database, integration tests do

### 🗄️ Database Debugging (Multi-Tenant)

**Connection Issues**:
- **Environment**: Must load `.env` file with `DATABASE_URL` in project root
- **Migration errors**: Use `scripts/migrate.sh` instead of `sqlx migrate` directly
- **Pool exhaustion**: Check `shared/db` configuration for connection limits

**Multi-Tenancy Debugging**:
- **RLS not working**: Verify RLS policies are enabled on tables
- **Tenant isolation fails**: Check `tenant_id` in JWT claims matches database records
- **Casbin permission denied**: Verify policies in `casbin_rule` table with correct tenant_id

**Query Debugging**:
- **Slow queries**: Check composite indexes `(tenant_id, other_columns)`
- **N+1 problems**: Use `sqlx::query!` with joins instead of multiple queries
- **Type mismatches**: SQLx provides compile-time checking, trust the errors

### 🔐 Authentication & Authorization Debugging

**JWT Issues**:
- **Token decode fails**: Check `JWT_SECRET` environment variable is set
- **Claims missing**: Verify `tenant_id`, `user_id`, `role` in token payload
- **Expiration errors**: Check `JWT_EXPIRATION` and `JWT_REFRESH_EXPIRATION` values

**Casbin Debugging**:
- **Policy not found**: Check `casbin_rule` table has correct `v0` (subject), `v1` (tenant), `v2` (resource), `v3` (action)
- **Permission denied**: Use Casbin debug logging to see evaluation flow
- **Multi-tenant isolation**: Ensure `v1` field matches tenant_id in JWT

**Password Security**:
- **Hash verification fails**: Ensure bcrypt cost factor is consistent
- **Strength validation**: Check zxcvbn score calculation with context
- **Migration needed**: TODO: Migrate from bcrypt to Argon2id

### 🚀 Development Environment Debugging

**Build Issues**:
- **Incremental compilation**: Use `cargo check --workspace` for fast feedback
- **Feature flags**: OpenAPI export requires `--features export-spec`
- **Binary naming**: Use `cargo run --bin user-service` (kebab-case)

**Test Environment**:
- **Integration tests fail**: Ensure PostgreSQL is running with correct DATABASE_URL
- **Unit tests pass**: Core crate tests don't need database connection
- **Mock setup**: Use mock services for API testing without external dependencies

**Docker Issues**:
- **Service discovery**: Services communicate via Docker Swarm overlay network
- **Volume mounting**: Check docker-compose.yml for persistent database storage
- **Network isolation**: Internal services not accessible from host machine

## 🛠️ Debugging Tools & Commands

### Essential Commands
```bash
# Check service logs
docker-compose logs [service-name]

# Database connection test
sqlx ping --database-url $DATABASE_URL

# Migration status
./scripts/migrate.sh status

# Casbin policy check
# Query casbin_rule table directly

# JWT decode (for debugging)
# Use online tools or shared/jwt crate functions
```

### Environment Validation
```bash
# Check all required environment variables
echo "DATABASE_URL: $DATABASE_URL"
echo "JWT_SECRET set: $(test -n "$JWT_SECRET" && echo 'YES' || echo 'NO')"
echo "JWT_EXPIRATION: $JWT_EXPIRATION"

# Verify database connectivity
sqlx query "SELECT 1 as test" --database-url $DATABASE_URL
```

### Log Analysis
- **tracing**: Use `RUST_LOG=debug` for detailed tracing logs
- **SQLx**: Set `SQLX_LOG=debug` for query logging
- **Casbin**: Enable debug logging to see policy evaluation

## ⚠️ Common Debug Scenarios

### Scenario: "Tenant isolation not working"
1. **Check JWT**: Verify `tenant_id` claim in token
2. **Check RLS**: Query `pg_policies` table for RLS policies
3. **Check query**: Ensure all queries include `tenant_id` filter
4. **Check Casbin**: Verify policies in `casbin_rule` table

### Scenario: "Service communication fails"
1. **Check network**: Verify `srv-` prefix usage
2. **Check ports**: Ensure services use different ports
3. **Check health**: Test each service `/health` endpoint
4. **Check logs**: Look for connection refused errors

### Scenario: "Tests failing"
1. **Check database**: Ensure PostgreSQL is running
2. **Check migrations**: Run `./scripts/migrate.sh run`
3. **Check environment**: Verify DATABASE_URL is set
4. **Check test type**: Unit vs integration test requirements