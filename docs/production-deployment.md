# Production Deployment Guide - Anthill SaaS

## Tổng Quan

Hướng dẫn triển khai production cho Anthill SaaS platform sử dụng **Docker Compose** để deploy lên VPS. Có thể sử dụng các công cụ như **Dokploy** hoặc **Komodo** để quản lý deployment.

## Kiến Trúc Triển Khai

### Docker Compose Services Structure

Mỗi microservice được triển khai như một **container riêng biệt** trong Docker Compose:

```
Docker Compose Stack
├── anthill-user-service (Port: 8000)
├── anthill-inventory-service (Port: 8001)
├── anthill-order-service (Port: 8002)
├── anthill-payment-service (Port: 8003)
├── anthill-integration-service (Port: 8004)
├── anthill-apisix-gateway (Port: 80/443/9180)
├── anthill-frontend (Port: 3000)
├── postgres (Port: 5432)
├── keydb (Port: 6379) - Redis-compatible cache
├── nats (Port: 4222)
└── rustfs (Port: 9000/9001) - S3-compatible storage
```

### Networking

- **Internal Communication**: Sử dụng Docker network hostname
  - `user-service:8000`
  - `inventory-service:8001`
  - etc.

- **External Access**: Chỉ qua APISIX Gateway
  - `https://your-domain.com/api/v1/*`

## Chuẩn Bị VPS

### 1. Cài Đặt Docker và Docker Compose

```bash
# Trên Ubuntu 22.04+/Debian 12+
sudo apt update
sudo apt install -y docker.io docker-compose-plugin
sudo systemctl start docker
sudo systemctl enable docker

# Thêm user vào docker group
sudo usermod -aG docker $USER
newgrp docker

# Verify installation
docker --version
docker compose version
```

### 2. Cấu Hình Domain

```bash
# Thêm A record cho domain
your-domain.com → VPS_IP
*.your-domain.com → VPS_IP

# Verify DNS
dig your-domain.com
```

### 3. SSL Certificates

```bash
# Install Certbot
sudo apt install certbot

# Generate certificates
sudo certbot certonly --standalone -d your-domain.com -d *.your-domain.com

# Certificates will be at:
# /etc/letsencrypt/live/your-domain.com/fullchain.pem
# /etc/letsencrypt/live/your-domain.com/privkey.pem
```

## Triển Khai với Docker Compose

### 1. Clone Repository

```bash
git clone <your-repo-url> /opt/anthill
cd /opt/anthill
```

### 2. Cấu Hình Environment Variables

```bash
# Copy example env file
cp .env.example .env.production

# Edit with your production values
nano .env.production
```

**Production Environment Variables**:
```bash
# Database
DATABASE_URL=postgres://inventory_user:STRONG_PASSWORD@postgres:5432/inventory_saas

# JWT
JWT_SECRET=CHANGE_THIS_STRONG_SECRET_64_CHARS_MIN

# KeyDB (Redis-compatible)
REDIS_URL=redis://keydb:6379
KEYDB_URL=redis://keydb:6379

# NATS
NATS_URL=nats://nats:4222

# RustFS (S3-compatible storage)
RUSTFS_ENDPOINT=http://rustfs:9000
RUSTFS_ACCESS_KEY=rustfsadmin
RUSTFS_SECRET_KEY=CHANGE_THIS_STRONG_PASSWORD
RUSTFS_BUCKET_NAME=anthill-files

# Self-auth (if using)
SELF_AUTH_URL=https://idm.your-domain.com
SELF_AUTH_CLIENT_ID=anthill
SELF_AUTH_CLIENT_SECRET=OAUTH2_CLIENT_SECRET
```

### 3. Start Stateful Services

```bash
# Start database, cache, message queue, storage
docker compose -f infra/docker_compose/docker-compose.yml up -d

# Verify services are healthy
docker compose -f infra/docker_compose/docker-compose.yml ps
```

### 4. Run Database Migrations

```bash
# Set DATABASE_URL
export DATABASE_URL=postgres://user:password@localhost:5432/inventory_db

# Run migrations
sqlx migrate run
```

### 5. Build and Deploy Microservices

```bash
# Build all service images
docker build -t anthill-user-service:latest -f services/user_service/Dockerfile .
docker build -t anthill-inventory-service:latest -f services/inventory_service/Dockerfile .
# ... repeat for other services

# Or use docker-compose for all services
docker compose -f docker-compose.production.yml up -d
```

## Production Docker Compose Example

Create `docker-compose.production.yml`:

```yaml
version: "3.8"

services:
  # Stateful Services
  postgres:
    image: postgres:16-alpine
    container_name: postgres
    environment:
      POSTGRES_USER: inventory_user
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: inventory_saas
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - anthill-network
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U inventory_user -d inventory_saas"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  keydb:
    image: eqalpha/keydb:latest
    container_name: keydb
    command: keydb-server --server-threads 2 --appendonly yes --requirepass ${KEYDB_PASSWORD}
    volumes:
      - keydb_data:/data
    networks:
      - anthill-network
    healthcheck:
      test: ["CMD", "keydb-cli", "-a", "${KEYDB_PASSWORD}", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  nats:
    image: nats:2.10-alpine
    container_name: nats
    command: "-js"
    networks:
      - anthill-network
    healthcheck:
      test: ["CMD-SHELL", "wget -q --spider http://localhost:8222/healthz || exit 1"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  rustfs:
    image: rustfs/rustfs:latest
    container_name: rustfs
    environment:
      RUSTFS_ROOT_USER: ${RUSTFS_ACCESS_KEY}
      RUSTFS_ROOT_PASSWORD: ${RUSTFS_SECRET_KEY}
    volumes:
      - rustfs_data:/data
    command: server /data --console-address ":9001"
    networks:
      - anthill-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9000/minio/health/ready"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  # Application Services
  user-service:
    image: anthill-user-service:latest
    container_name: user-service
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - JWT_SECRET=${JWT_SECRET}
      - REDIS_URL=redis://:${KEYDB_PASSWORD}@keydb:6379
      - NATS_URL=nats://nats:4222
      - PORT=8000
    networks:
      - anthill-network
    depends_on:
      postgres:
        condition: service_healthy
      keydb:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  inventory-service:
    image: anthill-inventory-service:latest
    container_name: inventory-service
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - JWT_SECRET=${JWT_SECRET}
      - REDIS_URL=redis://:${KEYDB_PASSWORD}@keydb:6379
      - NATS_URL=nats://nats:4222
      - PORT=8001
    networks:
      - anthill-network
    depends_on:
      postgres:
        condition: service_healthy
      keydb:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8001/health"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  # Add more services as needed...

  # Frontend
  frontend:
    image: anthill-frontend:latest
    container_name: frontend
    environment:
      - API_BASE_URL=http://apisix:9080/api/v1
    networks:
      - anthill-network
    restart: unless-stopped

  # APISIX Gateway
  apisix:
    image: apache/apisix:latest
    container_name: apisix
    ports:
      - "80:9080"
      - "443:9443"
      - "9180:9180"  # Admin API
    volumes:
      - ./infra/apisix/config.yaml:/usr/local/apisix/conf/config.yaml:ro
      - /etc/letsencrypt:/etc/letsencrypt:ro
    networks:
      - anthill-network
    depends_on:
      - user-service
      - inventory-service
      - frontend
    restart: unless-stopped

networks:
  anthill-network:
    driver: bridge

volumes:
  postgres_data:
  keydb_data:
  rustfs_data:
```

## Deployment với Dokploy / Komodo

### Dokploy Setup

1. **Install Dokploy** trên VPS:
   ```bash
   curl -fsSL https://get.dokploy.com | sh
   ```

2. **Truy cập Dokploy Dashboard** tại `http://your-vps-ip:3000`

3. **Create Project** và import Docker Compose configuration

4. **Configure Environment Variables** trong Dokploy UI

5. **Deploy** - Dokploy sẽ tự động build và deploy

### Komodo Setup

1. **Install Komodo**:
   ```bash
   curl -fsSL https://komodo.sh/install | sh
   ```

2. **Configure** với file `komodo.yml`

3. **Deploy**:
   ```bash
   komodo deploy
   ```

## Monitoring & Observability

### Prometheus + Grafana

```yaml
# Add to docker-compose.production.yml
  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    volumes:
      - ./infra/monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    networks:
      - anthill-network
    restart: unless-stopped

  grafana:
    image: grafana/grafana:latest
    container_name: grafana
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
    volumes:
      - grafana_data:/var/lib/grafana
    networks:
      - anthill-network
    restart: unless-stopped
```

### Application Metrics

Mỗi service expose `/metrics` endpoint với Prometheus format.

## Backup Strategy

### Database Backups

```bash
# Automated backup script
#!/bin/bash
BACKUP_DIR=/opt/backups
DATE=$(date +%Y%m%d_%H%M%S)

# Backup PostgreSQL
docker exec postgres pg_dump -U inventory_user inventory_saas > $BACKUP_DIR/db_$DATE.sql

# Backup RustFS
docker exec rustfs mc mirror /data $BACKUP_DIR/rustfs_$DATE

# Compress and upload to remote storage
gzip $BACKUP_DIR/*.sql
# Upload to S3/R2/etc...
```

### Cronjob Setup

```bash
# Add to crontab
0 2 * * * /opt/anthill/scripts/backup.sh >> /var/log/anthill-backup.log 2>&1
```

## Scaling & High Availability

### Horizontal Scaling

```bash
# Scale specific service
docker compose -f docker-compose.production.yml up -d --scale user-service=3
```

### Docker Swarm (Optional)

For higher availability, migrate to Docker Swarm:

```bash
# Initialize swarm
docker swarm init

# Deploy stack
docker stack deploy -c docker-compose.production.yml anthill
```

## Security Hardening

### Network Security

```bash
# UFW firewall
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow http
sudo ufw allow https
sudo ufw enable
```

### Container Security

- Run containers as non-root users
- Use read-only filesystems where possible
- Limit container resources (CPU, memory)
- Keep images updated

## Troubleshooting

### Common Issues

1. **Service Unhealthy**:
   ```bash
   docker compose logs user-service
   docker compose restart user-service
   ```

2. **Database Connection Failed**:
   ```bash
   # Check postgres logs
   docker compose logs postgres
   
   # Test connection
   docker exec postgres pg_isready -U inventory_user -d inventory_saas
   ```

3. **Storage Issues (RustFS)**:
   ```bash
   # Check RustFS health
   curl http://localhost:9000/minio/health/ready
   
   # Check logs
   docker compose logs rustfs
   ```

4. **Cache Issues (KeyDB)**:
   ```bash
   # Test KeyDB connection
   docker exec keydb keydb-cli ping
   
   # Check memory usage
   docker exec keydb keydb-cli info memory
   ```

### Debug Commands

```bash
# View all service statuses
docker compose ps

# View logs (last 100 lines)
docker compose logs --tail=100 <service-name>

# Enter container shell
docker compose exec <service-name> sh

# Check network connectivity
docker compose exec user-service ping postgres
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

---

**Last Updated**: January 2026
**Docker Version**: 24.0+
**Docker Compose Version**: 2.20+
