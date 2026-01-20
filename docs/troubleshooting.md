# Troubleshooting Guide - Anthill SaaS

## Tổng Quan

Hướng dẫn khắc phục sự cố phổ biến trong quá trình phát triển và triển khai Anthill SaaS platform.

## Development Issues

### Cargo Build Errors

#### 1. Compilation Errors

**Error**: `error[E0432]: unresolved import`
```
error[E0432]: unresolved import `shared::auth`
```

**Solution**:
```bash
# Clean và rebuild workspace
cargo clean
cargo build --workspace

# Hoặc check dependencies
cargo tree
```

#### 2. SQLx Compile-time Errors

**Error**: `error: query parameters count mismatch`
```
error: query parameters count mismatch: 2 parameters, 1 columns
```

**Solution**:
```bash
# Re-run sqlx prepare để update query metadata
cargo sqlx prepare --workspace

# Hoặc rebuild với offline mode
SQLX_OFFLINE=true cargo build
```

#### 3. UUID v7 Feature Flag

**Error**: `error: use of unstable library feature 'uuid_unstable'`
```
error: use of unstable library feature 'uuid_unstable'
```

**Solution**:
```bash
# Build với unstable flag
RUSTFLAGS="--cfg uuid_unstable" cargo build
```

### Database Issues

#### 1. Connection Refused

**Error**: `PoolTimedOut`
```
PoolTimedOut: timeout waiting for connection
```

**Solution**:
```bash
# Check PostgreSQL is running
docker-compose ps postgres

# Restart database
docker-compose restart postgres

# Check connection string
echo $DATABASE_URL
```

#### 2. Migration Errors

**Error**: `migration already applied`
```
error: migration 20250110000001_initial_extensions is already applied
```

**Solution**:
```bash
# Revert migration
sqlx migrate revert

# Hoặc skip và continue
sqlx migrate run --ignore-missing
```

#### 3. Permission Denied

**Error**: `permission denied for table users`
```
permission denied for table users
```

**Solution**:
```bash
# Check tenant_id context
# Ensure JWT contains correct tenant_id claim
# Verify repository methods include tenant filtering
```

### Authentication Issues

#### 1. JWT Validation Failed

**Error**: `Invalid JWT token`
```
AuthError: Invalid JWT token
```

**Solution**:
```bash
# Check JWT_SECRET environment variable
echo $JWT_SECRET

# Verify token structure
# Check expiry time
# Validate claims (sub, tenant_id, exp)
```

#### 2. Self-auth Integration Issues

**Error**: `Self-auth authentication failed`
```
Self-authError: invalid_client
```

**Solution**:
```bash
# Check Self-auth configuration
echo $SELF_AUTH_URL
echo $SELF_AUTH_CLIENT_ID
echo $SELF_AUTH_CLIENT_SECRET

# Verify Self-auth server is running
curl https://your-self-auth-domain.com/.well-known/openid-configuration

# Check OAuth2 client registration
```

#### 3. Casbin Authorization Failed

**Error**: `Permission denied`
```
CasbinError: permission denied for resource /api/v1/products
```

**Solution**:
```bash
# Check Casbin policies
# Query casbin_rule table
psql $DATABASE_URL -c "SELECT * FROM casbin_rule;"

# Verify user role assignment
# Check JWT claims for role information
```

### API Testing Issues

#### 1. Request Timeout

**Error**: `Request timeout`
```
ClientError: request timeout
```

**Solution**:
```bash
# Check service is running
curl http://localhost:8000/health

# Check service logs
docker-compose logs user-service

# Verify port mapping
docker-compose ps
```

#### 2. CORS Errors

**Error**: `CORS policy blocked`
```
Access to XMLHttpRequest blocked by CORS policy
```

**Solution**:
```bash
# Check CORS_ORIGINS environment variable
echo $CORS_ORIGINS

# Verify request origin matches allowed origins
# Check CORS middleware configuration
```

#### 3. Validation Errors

**Error**: `ValidationError`
```
ValidationError: email: invalid email format
```

**Solution**:
```bash
# Check input data format
# Verify validator crate annotations
# Check custom validation logic
```

## Testing Issues

### Unit Test Failures

#### 1. Database Test Connection

**Error**: `Connection refused in tests`
```
thread 'test_register_user' panicked at 'Connection refused'
```

**Solution**:
```rust
// Use test database URL
#[cfg(test)]
mod tests {
    use std::env;

    fn setup() {
        env::set_var("DATABASE_URL", "postgres://test:test@localhost:5433/test_db");
    }
}
```

#### 2. Async Test Timeouts

**Error**: `test timed out`
```
test test_register_user ... FAILED
thread 'test_register_user' panicked at 'test timed out', library/std/src/panics.rs:1:9
```

**Solution**:
```rust
#[tokio::test]
#[timeout(30000)] // 30 seconds
async fn test_register_user() {
    // Test code
}
```

### Integration Test Issues

#### 1. Service Dependencies

**Error**: `Service not available`
```
Connection refused (os error 111)
```

**Solution**:
```bash
# Start all services for integration tests
docker-compose up -d

# Wait for services to be ready
./scripts/wait-for-services.sh

# Run integration tests
cargo test --test integration
```

#### 2. Test Database Setup

**Error**: `relation "users" does not exist`
```
relation "users" does not exist
```

**Solution**:
```bash
# Run migrations on test database
sqlx migrate run --database-url $TEST_DATABASE_URL

# Or use test-specific migrations
./scripts/setup-test-db.sh
```

## Deployment Issues

### Docker Build Issues

#### 1. Build Context Too Large

**Error**: `build context too large`
```
build context too large: 2.1GB
```

**Solution**:
```dockerfile
# Use .dockerignore
# Exclude target/, .git/, docs/, etc.
```

#### 2. Multi-stage Build Issues

**Error**: `failed to compute cache key`
```
failed to compute cache key: "/app/target/release/user-service" not found
```

**Solution**:
```dockerfile
# Ensure correct build order
FROM rust:1.70-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/user-service /usr/local/bin/
```

### CapRover Deployment Issues

#### 1. App Creation Failed

**Error**: `App creation failed`
```
Failed to create app: invalid app name
```

**Solution**:
```bash
# Use lowercase with hyphens
# Valid: anthill-user-service
# Invalid: anthill_user_service, AnthillUserService
```

#### 2. Image Pull Failed

**Error**: `Image pull failed`
```
Failed to pull image: manifest unknown
```

**Solution**:
```bash
# Check image exists in registry
docker pull your-registry/anthill-user-service:latest

# Verify registry credentials
docker login your-registry.com

# Check image tag
docker images | grep anthill
```

#### 3. Environment Variables Not Set

**Error**: `environment variable not found`
```
DATABASE_URL environment variable not found
```

**Solution**:
```bash
# In CapRover dashboard
# Apps → [app-name] → Environment Variables
# Add all required variables
```

### APISIX Gateway Issues

#### 1. 502 Bad Gateway

**Error**: `502 Bad Gateway`
```
502 Bad Gateway
```

**Solution**:
```bash
# Check upstream service health
curl http://localhost:8000/health

# Check APISIX configuration
docker-compose exec apisix apisix test

# Check APISIX logs
docker-compose logs apisix
```

#### 2. SSL Certificate Issues

**Error**: `SSL certificate problem`
```
SSL certificate problem: unable to get local issuer certificate
```

**Solution**:
```bash
# Check certificate paths
ls -la /etc/ssl/anthill/

# Verify certificate validity
openssl x509 -in /etc/ssl/anthill/anthill.crt -text -noout

# Renew certificate
./scripts/generate-ssl-cert.sh letsencrypt
```

## Performance Issues

### High Memory Usage

**Symptoms**: Service consuming excessive memory

**Diagnosis**:
```bash
# Check memory usage
docker stats

# Check application metrics
curl http://localhost:8000/metrics

# Profile with heap dump
cargo build --release --features heap
```

**Solutions**:
```rust
// Use streaming responses for large data
// Implement connection pooling
// Add memory limits in Docker
```

### Slow Database Queries

**Symptoms**: API responses slow

**Diagnosis**:
```bash
# Enable query logging
ALTER DATABASE inventory_saas SET log_statement = 'all';

# Check slow queries
SELECT * FROM pg_stat_activity WHERE state = 'active';

# Use EXPLAIN ANALYZE
EXPLAIN ANALYZE SELECT * FROM users WHERE tenant_id = $1;
```

**Solutions**:
```sql
-- Add indexes
CREATE INDEX idx_users_tenant_email ON users(tenant_id, email);

-- Optimize queries
-- Use prepared statements
-- Implement query result caching
```

### High CPU Usage

**Symptoms**: Service consuming high CPU

**Diagnosis**:
```bash
# Check CPU usage
docker stats

# Profile application
cargo flamegraph --bin user-service
```

**Solutions**:
```rust
// Optimize hot code paths
// Use async/await efficiently
// Implement rate limiting
// Add CPU limits in Docker
```

## Monitoring & Alerting Issues

### Prometheus Scraping Issues

**Error**: `target down`
```
Target anthill-user-service:8000 is down
```

**Solution**:
```bash
# Check service metrics endpoint
curl http://localhost:8000/metrics

# Check Prometheus configuration
curl http://localhost:9090/config

# Verify service discovery
curl http://localhost:9090/api/v1/targets
```

### Grafana Dashboard Issues

**Error**: `No data`
```
No data to display
```

**Solution**:
```bash
# Check data source configuration
# Verify Prometheus connection
# Check query syntax
# Validate metric names
```

### AlertManager Issues

**Error**: `alerts not firing`
```
Alerts not firing as expected
```

**Solution**:
```bash
# Check alert rules
curl http://localhost:9090/api/v1/rules

# Check alert state
curl http://localhost:9090/api/v1/alerts

# Verify AlertManager configuration
curl http://localhost:9093/api/v2/status
```

## Security Issues

### Exposed Secrets

**Issue**: API keys or passwords in logs

**Solution**:
```bash
# Use structured logging
// Instead of
println!("User password: {}", password);

// Use
tracing::info!("User registered successfully");
// Don't log sensitive data
```

### Rate Limiting Issues

**Issue**: Services overwhelmed by requests

**Solution**:
```nginx
# Nginx rate limiting
limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
limit_req zone=api burst=20 nodelay;
```

### CORS Misconfiguration

**Issue**: Unauthorized cross-origin requests

**Solution**:
```rust
// Strict CORS configuration
let cors = CorsLayer::new()
    .allow_origin(["https://your-domain.com".parse()?])
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
    .allow_credentials(true);
```

## Emergency Procedures

### Service Outage

1. **Assess Impact**:
   ```bash
   # Check service status
   docker-compose ps

   # Check monitoring dashboards
   # Review recent deployments
   ```

2. **Isolate Issue**:
   ```bash
   # Check service logs
   docker-compose logs --tail=100 user-service

   # Check system resources
   docker stats

   # Check database connectivity
   psql $DATABASE_URL -c "SELECT 1;"
   ```

3. **Implement Fix**:
   ```bash
   # Restart service
   docker-compose restart user-service

   # Rollback deployment if needed
   caprover rollback anthill-user-service
   ```

4. **Communicate**:
   - Update monitoring alerts
   - Notify team/stakeholders
   - Document incident

### Data Loss

1. **Stop Services**:
   ```bash
   docker-compose stop
   ```

2. **Assess Damage**:
   ```bash
   # Check database integrity
   psql $DATABASE_URL -c "SELECT COUNT(*) FROM users;"

   # Verify backups
   ls -la /path/to/backups/
   ```

3. **Restore from Backup**:
   ```bash
   # Restore database
   pg_restore -d inventory_saas /path/to/backup.sql

   # Verify data integrity
   # Run consistency checks
   ```

4. **Resume Operations**:
   ```bash
   docker-compose start
   ```

## Useful Commands

### Development
```bash
# Clean rebuild
cargo clean && cargo build --workspace

# Run with debug logging
RUST_LOG=debug cargo run --bin user-service

# Test specific module
cargo test --package user_service_core

# Format code
cargo fmt --all

# Lint code
cargo clippy --all -- -D warnings
```

### Database
```bash
# Connect to database
psql $DATABASE_URL

# Run migrations
sqlx migrate run

# Check migration status
sqlx migrate status

# Reset database
sqlx migrate revert && sqlx migrate run
```

### Docker
```bash
# View logs
docker-compose logs -f user-service

# Execute into container
docker-compose exec user-service bash

# Check resource usage
docker stats

# Clean up
docker system prune -a
```

### Monitoring
```bash
# Check service health
curl http://localhost:8000/health

# View metrics
curl http://localhost:8000/metrics

# Query Prometheus
curl "http://localhost:9090/api/v1/query?query=up"

# Check Grafana
curl http://localhost:8000/api/health
```

### CapRover
```bash
# Deploy app
caprover deploy --appName anthill-user-service

# View logs
caprover logs --appName anthill-user-service

# Restart app
caprover restart --appName anthill-user-service

# Update environment
caprover set-env --appName anthill-user-service --envVars DATABASE_URL=...
```

## Getting Help

### Internal Resources
- **Documentation**: Check `docs/` folder
- **Team Chat**: Ask in development channel
- **Code Review**: Check recent PRs for similar issues

### External Resources
- **Rust Documentation**: https://doc.rust-lang.org/
- **Axum Documentation**: https://docs.rs/axum/
- **SQLx Documentation**: https://docs.rs/sqlx/
- **CapRover Documentation**: https://caprover.com/docs/

### Escalation
1. Check documentation and known issues
2. Ask team member for help
3. Create GitHub issue with full context
4. Escalate to senior developer if critical

---

**Last Updated**: November 4, 2025
**Version**: Anthill v1.0.0
