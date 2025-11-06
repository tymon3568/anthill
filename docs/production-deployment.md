# Production Deployment Guide - Anthill SaaS

## Tổng Quan

Hướng dẫn triển khai production cho Anthill SaaS platform sử dụng **CapRover** làm PaaS (Platform as a Service) trên VPS.

## Kiến Trúc Triển Khai

### CapRover Apps Structure

Mỗi microservice được triển khai như một **App riêng biệt** trong CapRover:

```
CapRover Dashboard
├── anthill-user-service (Port: 8000)
├── anthill-inventory-service (Port: 8001)
├── anthill-order-service (Port: 8002)
├── anthill-payment-service (Port: 8003)
├── anthill-integration-service (Port: 8004)
├── anthill-nginx-gateway (Port: 80/443)
├── anthill-frontend (Port: 3000)
├── anthill-postgres (One-Click App)
├── anthill-redis (One-Click App)
└── anthill-nats (One-Click App)
```

### Networking

- **Internal Communication**: Sử dụng Docker overlay network hostname
  - `srv-user-service:8000`
  - `srv-inventory-service:8001`
  - etc.

- **External Access**: Chỉ qua Nginx Gateway
  - `https://your-domain.com/api/v1/*`

## Chuẩn Bị VPS

### 1. Cài Đặt CapRover

```bash
# Trên Ubuntu 20.04+/Debian 10+
sudo apt update
sudo apt install docker.io docker-compose
sudo systemctl start docker
sudo systemctl enable docker

# Cài CapRover
docker run -p 80:80 -p 443:443 -p 3000:3000 -e ACCEPTED_TERMS=true -v /var/run/docker.sock:/var/run/docker.sock -v /captain:/captain caprover/cli-cpu:latest
```

Truy cập `http://your-server-ip:3000` để setup CapRover.

### 2. Cấu Hình Domain

```bash
# Thêm A record cho domain
your-domain.com → VPS_IP
*.your-domain.com → VPS_IP

# Trong CapRover: Apps → anthill-nginx-gateway → Domains
# Thêm: your-domain.com, *.your-domain.com
```

### 3. SSL Certificates

```bash
# Sử dụng script generate SSL
DOMAIN=your-domain.com ./scripts/generate-ssl-cert.sh letsencrypt

# Hoặc tự động qua CapRover (recommended)
# CapRover Dashboard → Apps → anthill-nginx-gateway → HTTP Settings → Force HTTPS
```

## Triển Khai Kanidm Authentication

### Kanidm Server Setup

1. **Deploy Kanidm Server**:
   ```bash
   # One-Click App hoặc custom deployment
   caprover deploy --appName anthill-kanidm --image kanidm/server:latest
   ```

2. **Kanidm Configuration**:
   ```bash
   # Environment Variables cho Kanidm
   KANIDM_DOMAIN=idm.your-domain.com
   KANIDM_ORIGIN=https://idm.your-domain.com
   KANIDM_TLS_CHAIN=/data/chain.pem
   KANIDM_TLS_KEY=/data/key.pem
   KANIDM_DB_PATH=/data/kanidm.db
   ```

3. **SSL Certificates cho Kanidm**:
   ```bash
   # Generate certificates
   ./scripts/generate-ssl-cert.sh kanidm idm.your-domain.com
   ```

### OAuth2 Client Configuration

1. **Tạo OAuth2 Client trong Kanidm**:
   ```bash
   # Via Kanidm CLI hoặc Web UI
   kanidm system oauth2 create \
     --name anthill \
     --displayname "Anthill SaaS" \
     --origin https://your-domain.com \
     --scope openid profile email groups
   ```

2. **Lấy Client Credentials**:
   ```bash
   # Sau khi tạo client
   kanidm system oauth2 show-basic-secret --name anthill
   # Output: Client ID và Secret
   ```

### Service Authentication Configuration

Mỗi microservice cần cấu hình Kanidm:

```bash
# Environment Variables cho tất cả services
KANIDM_URL=https://idm.your-domain.com
KANIDM_CLIENT_ID=anthill
KANIDM_CLIENT_SECRET=YOUR_OAUTH2_CLIENT_SECRET
JWT_SECRET=CHANGE_THIS_STRONG_SECRET_64_CHARS_MIN
OAUTH2_REDIRECT_URI=https://your-domain.com/api/v1/auth/oauth/callback
```

### Frontend Authentication Configuration

```bash
# Frontend Environment Variables
PUBLIC_KANIDM_AUTHORIZE_URL=https://idm.your-domain.com/ui/oauth2
PUBLIC_KANIDM_TOKEN_URL=https://idm.your-domain.com/oauth2/token
PUBLIC_KANIDM_CLIENT_ID=anthill
PUBLIC_OAUTH2_REDIRECT_URI=https://your-domain.com/api/v1/auth/oauth/callback
PUBLIC_API_BASE_URL=https://your-domain.com/api/v1
```

### Tenant Mapping Setup

1. **Tạo Kanidm Groups cho Tenants**:
   ```bash
   # Tạo groups trong Kanidm
   kanidm group create tenant_acme_users
   kanidm group create tenant_acme_admins
   kanidm group create tenant_xyz_users
   ```

2. **Database Mapping**:
   ```sql
   -- Insert tenant-group mappings
   INSERT INTO kanidm_tenant_groups (tenant_id, kanidm_group_uuid, kanidm_group_name)
   VALUES
     ('550e8400-e29b-41d4-a716-446655440000', 'group-uuid-1', 'tenant_acme_users'),
     ('550e8400-e29b-41d4-a716-446655440001', 'group-uuid-2', 'tenant_xyz_users');
   ```

### Authentication Flow Testing

Sau khi deploy, test authentication flow:

```bash
# 1. Test OAuth2 authorize endpoint
curl -X GET "https://your-domain.com/api/v1/auth/oauth/authorize" \
  -H "Accept: application/json"

# Should redirect to Kanidm login

# 2. Test protected endpoint
curl -X GET "https://your-domain.com/api/v1/users/profile" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# Should return user profile or 401 if not authenticated

# 3. Test token refresh
curl -X POST "https://your-domain.com/api/v1/auth/oauth/refresh" \
  -H "Content-Type: application/json" \
  -d '{"refresh_token": "YOUR_REFRESH_TOKEN"}'

# Should return new access token
```

### Monitoring Authentication

Thêm authentication metrics vào monitoring:

```rust
// Trong mỗi service, track auth events
use prometheus::{register_counter, register_histogram};

lazy_static! {
    static ref AUTH_REQUESTS: Counter = register_counter!(
        "auth_requests_total",
        "Total number of authentication requests"
    ).unwrap();

    static ref AUTH_SUCCESS: Counter = register_counter!(
        "auth_success_total",
        "Total number of successful authentications"
    ).unwrap();

    static ref AUTH_FAILURES: Counter = register_counter!(
        "auth_failures_total",
        "Total number of authentication failures"
    ).unwrap();

    static ref TOKEN_REFRESH_DURATION: Histogram = register_histogram!(
        "token_refresh_duration_seconds",
        "Time taken for token refresh"
    ).unwrap();
}
```

### Security Considerations

1. **Token Storage**: HttpOnly cookies, never localStorage
2. **CORS Policy**: Restrict origins to your domain only
3. **Rate Limiting**: Implement OAuth2 endpoint rate limiting
4. **Audit Logging**: Log all authentication events
5. **Session Management**: Short-lived access tokens (15 min), refresh tokens (24h)

### Backup & Recovery

1. **Kanidm Database Backup**:
   ```bash
   # Backup Kanidm data
   caprover exec --appName anthill-kanidm --command "sqlite3 /data/kanidm.db .backup /backup/kanidm-$(date +%Y%m%d).db"
   ```

2. **JWT Secrets Backup**:
   - Store JWT secrets in secure vault (HashiCorp Vault, AWS Secrets Manager)
   - Rotate secrets quarterly
   - Document secret rotation procedure

### Troubleshooting Authentication

#### Common Issues

1. **OAuth2 Redirect URI Mismatch**:
   ```
   Error: redirect_uri does not match
   Solution: Verify OAUTH2_REDIRECT_URI matches Kanidm client configuration
   ```

2. **Token Validation Failed**:
   ```
   Error: JWT signature verification failed
   Solution: Check JWT_SECRET consistency across services
   ```

3. **Tenant Context Missing**:
   ```
   Error: No tenant found for user
   Solution: Verify kanidm_tenant_groups table has correct mappings
   ```

4. **CORS Errors**:
   ```
   Error: CORS policy blocked
   Solution: Check Nginx CORS configuration for auth endpoints
   ```

#### Debug Commands

```bash
# Check Kanidm service status
caprover logs --appName anthill-kanidm

# Test OAuth2 endpoints
curl -v "https://idm.your-domain.com/.well-known/openid-configuration"

# Check JWT token contents
curl -X POST "https://your-domain.com/api/v1/auth/oauth/callback" \
  -d "code=test-code&state=test-state" \
  -v

# Verify tenant mapping
caprover exec --appName anthill-postgres --command "psql -c 'SELECT * FROM kanidm_tenant_groups;'"
```

### PostgreSQL Database

1. **CapRover One-Click App**:
   - Tìm "PostgreSQL" trong One-Click Apps
   - App Name: `anthill-postgres`
   - Version: Latest stable
   - Persistent Storage: Enable

2. **Environment Variables**:
   ```
   POSTGRES_DB=inventory_saas
   POSTGRES_USER=inventory_user
   POSTGRES_PASSWORD=CHANGE_THIS_STRONG_PASSWORD
   ```

3. **Backup Configuration**:
   - Enable automated backups
   - Schedule: Daily at 2 AM
   - Retention: 30 days

### Redis Cache

1. **One-Click App**: `anthill-redis`
2. **Persistent Storage**: Enable
3. **Environment**:
   ```
   REDIS_PASSWORD=CHANGE_THIS_STRONG_PASSWORD
   ```

### NATS Message Queue

1. **One-Click App**: `anthill-nats`
2. **Configuration**:
   ```
   NATS_CLUSTER_NAME=anthill-cluster
   ```

## Triển Khai Microservices

### Chuẩn Bị Docker Images

Mỗi service cần `Dockerfile` trong thư mục gốc:

```dockerfile
# services/user_service/Dockerfile
FROM rust:1.70-slim AS builder

WORKDIR /app
COPY . .

# Build với optimizations
RUN cargo build --release --bin user-service

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/user-service /usr/local/bin/user-service

EXPOSE 8000
CMD ["user-service"]
```

### Build & Push Images

```bash
# Login Docker registry (Docker Hub, GitHub Container Registry, etc.)
docker login

# Build và push từng service
cd services/user_service
docker build -t your-registry/anthill-user-service:latest .
docker push your-registry/anthill-user-service:latest

# Lặp lại cho các service khác
```

### CapRover App Configuration

Cho mỗi service, tạo app với cấu hình:

#### User Service Example

1. **App Name**: `anthill-user-service`
2. **Source**:
   - Method: `ImageName`
   - Image: `your-registry/anthill-user-service:latest`

3. **Environment Variables**:
   ```bash
   DATABASE_URL=postgres://inventory_user:PASSWORD@srv-anthill-postgres:5432/inventory_saas
   JWT_SECRET=CHANGE_THIS_STRONG_SECRET_64_CHARS
   REDIS_URL=redis://srv-anthill-redis:6379
   NATS_URL=nats://srv-anthill-nats:4222
   KANIDM_URL=https://idm.your-domain.com
   KANIDM_CLIENT_ID=anthill
   KANIDM_CLIENT_SECRET=OAUTH2_CLIENT_SECRET
   ```

4. **Port Mapping**: `8000` (internal port)

5. **Persistent Storage**: Không cần (stateless)

6. **Health Check**:
   - HTTP Path: `/health`
   - Initial Delay: 30s
   - Period: 10s
   - Timeout: 5s

### Lặp Lại Cho Các Service Khác

- **Inventory Service**: Port 8001
- **Order Service**: Port 8002
- **Payment Service**: Port 8003
- **Integration Service**: Port 8004

## Triển Khai Nginx Gateway

### Dockerfile

```dockerfile
# infra/nginx/Dockerfile
FROM nginx:alpine

COPY nginx.conf /etc/nginx/nginx.conf
COPY conf.d/ /etc/nginx/conf.d/

EXPOSE 80 443
CMD ["nginx", "-g", "daemon off;"]
```

### CapRover Configuration

1. **App Name**: `anthill-nginx-gateway`
2. **Source**: Image from `infra/nginx/`
3. **Port Mapping**: `80`, `443`
4. **Domains**: `your-domain.com`, `*.your-domain.com`
5. **SSL**: Force HTTPS, Let's Encrypt auto-renewal

### Nginx Configuration

```nginx
# infra/nginx/conf.d/upstreams.conf
upstream user_service {
    server srv-anthill-user-service:8000;
}
upstream inventory_service {
    server srv-anthill-inventory-service:8001;
}
# ... other services

# infra/nginx/conf.d/api-gateway.conf
location ~ ^/api/v1/auth {
    proxy_pass http://user_service;
    # proxy headers...
}
location ~ ^/api/v1/products {
    proxy_pass http://inventory_service;
}
# ... routing rules
```

## Triển Khai Frontend

### Build Configuration

```dockerfile
# Dockerfile for SvelteKit
FROM node:18-alpine AS builder

WORKDIR /app
COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build

FROM node:18-alpine AS runtime

WORKDIR /app
COPY --from=builder /app/package*.json ./
COPY --from=builder /app/build ./
COPY --from=builder /app/node_modules ./node_modules

EXPOSE 3000
CMD ["node", "build/index.js"]
```

### CapRover App

1. **App Name**: `anthill-frontend`
2. **Source**: Image hoặc GitHub integration
3. **Environment**:
   ```
   API_BASE_URL=https://your-domain.com/api/v1
   ```

## Monitoring & Observability

### Prometheus + Grafana

1. **Deploy Prometheus**: One-Click App `anthill-prometheus`
2. **Deploy Grafana**: One-Click App `anthill-grafana`
3. **Configure Data Source**: Prometheus URL
4. **Import Dashboards**: Anthill overview dashboard

### Application Metrics

Mỗi service expose `/metrics` endpoint với Prometheus format:

```rust
use prometheus::{Encoder, TextEncoder, register_counter};
use warp::Filter;

let request_counter = register_counter!("requests_total", "Total number of requests").unwrap();

let metrics_route = warp::path("metrics")
    .map(|| {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    });
```

## Database Migrations

### Chạy Migrations Trên Production

```bash
# Via CapRover CLI
caprover exec --appName anthill-postgres --command "psql -U inventory_user -d inventory_saas -f /path/to/migrations.sql"

# Hoặc tạo migration runner service
caprover exec --appName anthill-user-service --command "./migration-runner"
```

## Backup Strategy

### Database Backups

```bash
# Automated via CapRover
# Apps → anthill-postgres → Backup
# - Schedule: 0 2 * * * (daily 2 AM)
# - Retention: 30 days
# - Storage: External S3/MinIO
```

### Application Backups

- **Docker Images**: Push to registry with tags
- **Configuration**: Git repository
- **SSL Certificates**: Backup `/etc/ssl/anthill/`

## Scaling & Performance

### Horizontal Scaling

```bash
# Scale service instances
caprover scale --appName anthill-user-service --replicas 3

# Load balancing tự động qua CapRover
```

### Database Optimization

```sql
-- Connection pooling
ALTER SYSTEM SET max_connections = '200';

-- Shared buffers (25% of RAM)
ALTER SYSTEM SET shared_buffers = '512MB';

-- Work memory
ALTER SYSTEM SET work_mem = '4MB';
```

## Security Hardening

### Network Security

- **Firewall**: Chỉ mở ports 80, 443, 22
- **Internal Network**: Services chỉ communicate qua internal network
- **SSL/TLS**: Force HTTPS everywhere

### Application Security

- **Environment Variables**: Không hardcode secrets
- **JWT Tokens**: Short expiry (15 minutes) + refresh tokens
- **Rate Limiting**: Implement via Nginx
- **CORS**: Restrict to allowed origins

### Database Security

```sql
-- Create read-only user cho analytics
CREATE USER analytics_user WITH PASSWORD 'STRONG_PASSWORD';
GRANT CONNECT ON DATABASE inventory_saas TO analytics_user;
GRANT USAGE ON SCHEMA public TO analytics_user;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO analytics_user;

-- Row Level Security (future)
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
```

## Troubleshooting

### Common Issues

1. **Service Unhealthy**:
   ```bash
   caprover logs --appName anthill-user-service
   caprover restart --appName anthill-user-service
   ```

2. **Database Connection Failed**:
   - Check `DATABASE_URL` environment variable
   - Verify PostgreSQL service is running
   - Check network connectivity: `caprover exec --appName anthill-user-service --command "ping srv-anthill-postgres"`

3. **SSL Certificate Issues**:
   ```bash
   # Renew certificates
   ./scripts/generate-ssl-cert.sh letsencrypt
   caprover restart --appName anthill-nginx-gateway
   ```

### Monitoring Commands

```bash
# Check service status
caprover ps

# View logs
caprover logs --appName anthill-user-service --lines 100

# Check resource usage
caprover info --appName anthill-user-service

# Debug container
caprover exec --appName anthill-user-service --command "bash"
```

## Maintenance Tasks

### Weekly
- Review monitoring dashboards
- Check SSL certificate expiry
- Monitor disk usage

### Monthly
- Update Docker images
- Review and rotate secrets
- Test backup restoration

### Quarterly
- Security updates
- Performance optimization
- Capacity planning

## Emergency Procedures

### Service Outage
1. Check CapRover dashboard for service status
2. Review logs for error messages
3. Restart affected service
4. If database issue, check PostgreSQL logs
5. Communicate with users about downtime

### Data Loss
1. Stop all services immediately
2. Restore from latest backup
3. Verify data integrity
4. Restart services gradually
5. Investigate root cause

## Cost Optimization

### VPS Selection
- **Development**: 2 vCPU, 4GB RAM ($20/month)
- **Production Small**: 4 vCPU, 8GB RAM ($40/month)
- **Production Medium**: 8 vCPU, 16GB RAM ($80/month)

### Resource Allocation
- **PostgreSQL**: 2 vCPU, 4GB RAM
- **Redis**: 1 vCPU, 2GB RAM
- **NATS**: 1 vCPU, 1GB RAM
- **Each Service**: 1 vCPU, 2GB RAM (scale as needed)

## Migration Strategy

### Blue-Green Deployment
1. Deploy new version to `anthill-user-service-green`
2. Test thoroughly
3. Switch traffic via Nginx upstream
4. Keep old version as rollback option

### Zero-Downtime Updates
- Use database migrations with backward compatibility
- Deploy services one by one
- Monitor health checks during rollout
- Rollback plan ready

---

**Last Updated**: November 4, 2025
**CapRover Version**: Latest stable
**Docker Version**: 24.0+
