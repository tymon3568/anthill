# Anthill Nginx - API Gateway & Reverse Proxy

Production-ready nginx configuration serving as API Gateway for the Anthill SaaS platform.

## 🎯 Overview

This nginx setup provides:
- **API Gateway**: Routes requests to appropriate microservices
- **Load Balancing**: Distributes traffic across service instances
- **SSL/TLS Termination**: Handles HTTPS encryption
- **Rate Limiting**: Protects against abuse
- **Security Headers**: Implements security best practices
- **Multi-Tenant Support**: Routes based on tenant context

## 📁 Structure

```
nginx/
├── Dockerfile              # Container build configuration
├── nginx.conf             # Main nginx configuration
├── conf.d/
│   ├── upstreams.conf     # Backend service definitions
│   ├── api-gateway.conf   # Routing rules
│   └── ssl.conf          # SSL/TLS settings
├── ssl/                   # SSL certificates (gitignored)
├── docker-compose.override.yml
├── .dockerignore
└── README.md
```

## 🚀 Quick Start

### Development Setup

1. **Generate self-signed SSL certificates**:
   ```bash
   # Use the SSL generation script
   ./scripts/generate-ssl-cert.sh selfsigned

   # Or manually create certificates
   mkdir -p infra/nginx/ssl
   openssl req -x509 -newkey rsa:4096 -keyout infra/nginx/ssl/anthill.key \
           -out infra/nginx/ssl/anthill.crt -days 365 -nodes \
           -subj "/CN=localhost"
   ```

2. **Start with nginx**:
   ```bash
   docker-compose -f docker-compose.yml -f infra/nginx/docker-compose.override.yml up nginx
   ```

3. **Test the setup**:
   ```bash
   # Health check
   curl http://localhost/health

   # API through nginx
   curl https://localhost/api/v1/auth/health -k  # Skip SSL verification for self-signed
   ```

### Production Setup

1. **Get SSL certificates**:
   ```bash
   # Let's Encrypt (recommended)
   ./scripts/generate-ssl-cert.sh letsencrypt

   # Or place your certificates in infra/nginx/ssl/
   # - anthill.crt (certificate chain)
   # - anthill.key (private key)
   ```

2. **Update environment variables**:
   ```bash
   # In your .env.production file
   SSL_CERT_PATH=/etc/ssl/anthill/anthill.crt
   SSL_KEY_PATH=/etc/ssl/anthill/anthill.key
   ```

3. **Deploy with production compose**:
   ```bash
   docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
   ```

## 🔧 Configuration

### Routing Rules

Requests are routed based on URL patterns:

- `/api/v1/auth/*` → `user-service:8000`
- `/api/v1/users/*` → `user-service:8000`
- `/api/v1/products/*` → `inventory-service:8001`
- `/api/v1/orders/*` → `order-service:8002`
- `/api/v1/payments/*` → `payment-service:8003`
- `/api/v1/integrations/*` → `integration-service:8004`

### Rate Limiting

- **API endpoints**: 10 requests/second per IP
- **Auth endpoints**: 5 requests/minute per IP
- **Tenant endpoints**: 100 requests/second per tenant

### Security Features

- **HSTS**: Forces HTTPS connections
- **CORS**: Configured for allowed origins
- **Security Headers**: XSS protection, content type sniffing prevention
- **SSL/TLS**: Modern cipher suites, session resumption

### Multi-Tenant Support

Tenant context is passed via headers:
- `X-Tenant-ID`: Tenant identifier
- `X-Tenant-Slug`: Tenant slug for URL generation
- `X-Request-ID`: Unique request identifier

## 🔍 Monitoring

### Health Checks

- **Internal**: `/health` endpoint returns JSON status
- **Services**: Automatic health checks for upstream servers
- **SSL**: Certificate validity monitoring

### Logs

Access and error logs are available at:
- `/var/log/nginx/access.log`
- `/var/log/nginx/error.log`

### Metrics

Prometheus metrics available at `/metrics` (internal network only).

## 🛠️ Development

### Adding New Services

1. **Add upstream in `conf.d/upstreams.conf`**:
   ```nginx
   upstream new_service {
       least_conn;
       server new-service:8005 max_fails=3 fail_timeout=30s;
       keepalive 32;
   }
   ```

2. **Add routing in `conf.d/api-gateway.conf`**:
   ```nginx
   location ~ ^/api/v1/new-endpoint {
       proxy_pass http://new_service;
       # ... proxy headers ...
   }
   ```

3. **Rebuild nginx container**:
   ```bash
   docker-compose -f infra/nginx/docker-compose.override.yml build nginx
   ```

### SSL Certificate Management

For production, certificates are automatically renewed via cron job. Monitor renewal logs:

```bash
# Check renewal status
sudo systemctl status certbot.timer

# Manual renewal
sudo certbot renew

# Check certificate expiry
openssl x509 -in /etc/ssl/anthill/anthill.crt -text -noout | grep "Not After"
```

## 🚨 Troubleshooting

### Common Issues

1. **502 Bad Gateway**: Service is down or unreachable
   - Check service health: `docker-compose ps`
   - Check service logs: `docker-compose logs <service>`

2. **SSL Certificate Errors**: Certificate expired or invalid
   - Renew certificates: `./scripts/generate-ssl-cert.sh letsencrypt`
   - Check certificate: `openssl x509 -in ssl/anthill.crt -text`

3. **Rate Limiting**: Too many requests
   - Check nginx logs for rate limit hits
   - Adjust limits in `nginx.conf` if needed

### Debug Mode

Enable debug logging by setting in `nginx.conf`:
```nginx
error_log /var/log/nginx/error.log debug;
```

## 📚 References

- [Nginx Documentation](https://nginx.org/en/docs/)
- [Let's Encrypt](https://letsencrypt.org/)
- [SSL/TLS Best Practices](https://ssl-config.mozilla.org/)
