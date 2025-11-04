# Monitoring Setup Guide - Anthill SaaS

## Tổng Quan

Hướng dẫn thiết lập monitoring và observability cho Anthill SaaS platform sử dụng Prometheus, Grafana, Loki, và AlertManager.

## Kiến Trúc Monitoring

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Services      │    │   Prometheus    │    │    Grafana      │
│   (Metrics)     │───▶│   (Scraping)    │───▶│   (Dashboards)   │
│                 │    │                 │    │                 │
│ • /metrics      │    │ • Service       │    │ • Anthill        │
│ • Health checks │    │   Discovery     │    │   Overview      │
│ • Custom metrics│    │ • Alert Rules   │    │ • Service        │
└─────────────────┘    └─────────────────┘    │   Details       │
                                              └─────────────────┘
┌─────────────────┐    ┌─────────────────┐
│     Loki        │    │  AlertManager   │
│   (Logs)        │    │   (Alerts)      │
│                 │    │                 │
│ • Promtail      │    │ • Email         │
│ • Log aggregation│    │ • Slack        │
│ • Query logs    │    │ • PagerDuty     │
└─────────────────┘    └─────────────────┘
```

## Triển Khai Monitoring Stack

### Sử dụng Docker Compose

```yaml
# infra/docker_compose/docker-compose.monitoring.yml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--storage.tsdb.retention.time=200h'
      - '--web.enable-lifecycle'

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana/provisioning:/etc/grafana/provisioning
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_USERS_ALLOW_SIGN_UP=false

  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"
    volumes:
      - ./monitoring/loki/local-config.yaml:/etc/loki/local-config.yaml
      - loki_data:/loki
    command: -config.file=/etc/loki/local-config.yaml

  promtail:
    image: grafana/promtail:latest
    volumes:
      - ./monitoring/promtail/config.yml:/etc/promtail/config.yml
      - /var/log:/var/log
    command: -config.file=/etc/promtail/config.yml

  alertmanager:
    image: prom/alertmanager:latest
    ports:
      - "9093:9093"
    volumes:
      - ./monitoring/alertmanager/alertmanager.yml:/etc/alertmanager/config.yml
    command:
      - '--config.file=/etc/alertmanager/config.yml'
      - '--storage.path=/alertmanager'

volumes:
  prometheus_data:
  grafana_data:
  loki_data:
```

### Khởi Động Monitoring Stack

```bash
# Từ thư mục root của project
docker-compose -f infra/docker_compose/docker-compose.monitoring.yml up -d

# Kiểm tra services
docker-compose -f infra/docker_compose/docker-compose.monitoring.yml ps
```

## Cấu Hình Prometheus

### Prometheus Configuration

```yaml
# infra/monitoring/prometheus/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "alert_rules.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

scrape_configs:
  # Anthill Services
  - job_name: 'anthill-user-service'
    static_configs:
      - targets: ['host.docker.internal:8000']
    metrics_path: '/metrics'
    scrape_interval: 5s

  - job_name: 'anthill-inventory-service'
    static_configs:
      - targets: ['host.docker.internal:8001']
    metrics_path: '/metrics'
    scrape_interval: 5s

  - job_name: 'anthill-order-service'
    static_configs:
      - targets: ['host.docker.internal:8002']
    metrics_path: '/metrics'
    scrape_interval: 5s

  - job_name: 'anthill-payment-service'
    static_configs:
      - targets: ['host.docker.internal:8003']
    metrics_path: '/metrics'
    scrape_interval: 5s

  - job_name: 'anthill-integration-service'
    static_configs:
      - targets: ['host.docker.internal:8004']
    metrics_path: '/metrics'
    scrape_interval: 5s

  # Infrastructure
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'grafana'
    static_configs:
      - targets: ['grafana:3000']

  - job_name: 'loki'
    static_configs:
      - targets: ['loki:3100']

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres_exporter:9187']

  - job_name: 'redis'
    static_configs:
      - targets: ['redis_exporter:9121']

  - job_name: 'nats'
    static_configs:
      - targets: ['nats:8222']
```

### Alert Rules

```yaml
# infra/monitoring/prometheus/alert_rules.yml
groups:
  - name: anthill
    rules:
      # Service Health
      - alert: ServiceDown
        expr: up == 0
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Service {{ $labels.job }} is down"
          description: "Service {{ $labels.job }} has been down for more than 5 minutes."

      # High Error Rate
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m]) > 0.05
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate on {{ $labels.service }}"
          description: "Error rate is {{ $value }}% on {{ $labels.service }}"

      # High Latency
      - alert: HighLatency
        expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 2
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High latency on {{ $labels.service }}"
          description: "95th percentile latency is {{ $value }}s on {{ $labels.service }}"

      # Database Connection Issues
      - alert: DatabaseConnectionsHigh
        expr: pg_stat_activity_count > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High database connections"
          description: "Database has {{ $value }} active connections"

      # Disk Space
      - alert: DiskSpaceLow
        expr: (1 - node_filesystem_avail_bytes / node_filesystem_size_bytes) * 100 > 85
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Low disk space on {{ $labels.instance }}"
          description: "Disk usage is {{ $value }}%"

      # Memory Usage
      - alert: MemoryUsageHigh
        expr: (1 - node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) * 100 > 90
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High memory usage on {{ $labels.instance }}"
          description: "Memory usage is {{ $value }}%"
```

## Cấu Hình Grafana

### Data Sources

```yaml
# infra/monitoring/grafana/provisioning/datasources/datasources.yml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true

  - name: Loki
    type: loki
    access: proxy
    url: http://loki:3100
```

### Dashboards

```yaml
# infra/monitoring/grafana/provisioning/dashboards/dashboards.yml
apiVersion: 1

providers:
  - name: 'Anthill'
    type: file
    disableDeletion: false
    updateIntervalSeconds: 10
    allowUiUpdates: true
    options:
      path: /etc/grafana/provisioning/dashboards
```

### Anthill Overview Dashboard

```json
{
  "dashboard": {
    "title": "Anthill SaaS Overview",
    "tags": ["anthill", "overview"],
    "timezone": "browser",
    "panels": [
      {
        "title": "Service Health",
        "type": "stat",
        "targets": [
          {
            "expr": "up{job=~\"anthill-.*\"}",
            "legendFormat": "{{job}}"
          }
        ]
      },
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])",
            "legendFormat": "{{service}}"
          }
        ]
      },
      {
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total{status=~\"5..\"}[5m]) / rate(http_requests_total[5m]) * 100",
            "legendFormat": "{{service}}"
          }
        ]
      },
      {
        "title": "Response Time (95th percentile)",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "{{service}}"
          }
        ]
      },
      {
        "title": "Database Connections",
        "type": "graph",
        "targets": [
          {
            "expr": "pg_stat_activity_count",
            "legendFormat": "Active Connections"
          }
        ]
      },
      {
        "title": "System Resources",
        "type": "row",
        "panels": [
          {
            "title": "CPU Usage",
            "type": "graph",
            "targets": [
              {
                "expr": "100 - (avg by(instance) (irate(node_cpu_seconds_total{mode=\"idle\"}[5m])) * 100)",
                "legendFormat": "{{instance}}"
              }
            ]
          },
          {
            "title": "Memory Usage",
            "type": "graph",
            "targets": [
              {
                "expr": "(1 - node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) * 100",
                "legendFormat": "{{instance}}"
              }
            ]
          },
          {
            "title": "Disk Usage",
            "type": "graph",
            "targets": [
              {
                "expr": "(1 - node_filesystem_avail_bytes / node_filesystem_size_bytes) * 100",
                "legendFormat": "{{instance}}"
              }
            ]
          }
        ]
      }
    ]
  }
}
```

## Cấu Hình Loki (Logging)

### Loki Configuration

```yaml
# infra/monitoring/loki/local-config.yaml
auth_enabled: false

server:
  http_listen_port: 3100
  grpc_listen_port: 9096

ingester:
  lifecycler:
    address: 127.0.0.1
    ring:
      kvstore:
        store: inmemory
      replication_factor: 1
    final_sleep: 0s
  chunk_idle_period: 1h
  max_chunk_age: 1h
  chunk_target_size: 1048576
  chunk_retain_period: 30s
  max_transfer_retries: 0

schema_config:
  configs:
    - from: 2020-10-24
      store: boltdb-shipper
      object_store: filesystem
      schema: v11
      index:
        prefix: index_
        period: 24h

storage_config:
  boltdb_shipper:
    active_index_directory: /loki/boltdb-shipper-active
    cache_location: /loki/boltdb-shipper-cache
    cache_ttl: 24h
    shared_store: filesystem
  filesystem:
    directory: /loki/chunks

chunk_store_config:
  max_look_back_period: 0s

table_manager:
  retention_deletes_enabled: false
  retention_period: 0s

compactor:
  working_directory: /loki/boltdb-shipper-compactor
  shared_store: filesystem
```

### Promtail Configuration

```yaml
# infra/monitoring/promtail/config.yml
server:
  http_listen_port: 9080
  grpc_listen_port: 0

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  - job_name: anthill-services
    static_configs:
      - targets:
          - localhost
        labels:
          job: anthill
          __path__: /var/log/anthill/*.log
    pipeline_stages:
      - match:
          selector: '{job="anthill"}'
          stages:
            - json:
                expressions:
                  level: level
                  service: service
                  message: message
            - labels:
                level:
                service:

  - job_name: nginx
    static_configs:
      - targets:
          - localhost
        labels:
          job: nginx
          __path__: /var/log/nginx/*.log
    pipeline_stages:
      - match:
          selector: '{job="nginx"}'
          stages:
            - regex:
                expression: '^(?P<remote_addr>\S+) - (?P<remote_user>\S+) \[(?P<time_local>.*?)\] "(?P<request>.*?)" (?P<status>\d+) (?P<body_bytes_sent>\d+) "(?P<http_referer>.*?)" "(?P<http_user_agent>.*?)" "(?P<http_x_forwarded_for>.*?)" rt=(?P<request_time>.*?) uct="(?P<upstream_connect_time>.*?)" uht="(?P<upstream_header_time>.*?)" urt="(?P<upstream_response_time>.*?)"$'
            - labels:
                status:
                method:
                path:

  - job_name: system
    static_configs:
      - targets:
          - localhost
        labels:
          job: system
          __path__: /var/log/syslog
```

## Cấu Hình AlertManager

### AlertManager Configuration

```yaml
# infra/monitoring/alertmanager/alertmanager.yml
global:
  smtp_smarthost: 'smtp.gmail.com:587'
  smtp_from: 'alerts@your-domain.com'
  smtp_auth_username: 'alerts@your-domain.com'
  smtp_auth_password: 'your-app-password'

route:
  group_by: ['alertname']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'email'
  routes:
    - match:
        severity: critical
      receiver: 'email-critical'

receivers:
  - name: 'email'
    email_configs:
      - to: 'team@your-domain.com'
        subject: '[{{ .GroupLabels.alertname }}] {{ .Annotations.summary }}'
        body: |
          {{ range .Alerts }}
          Alert: {{ .Annotations.summary }}
          Description: {{ .Annotations.description }}
          Details:
          {{ range .Labels.SortedPairs }} • {{ .Name }}: {{ .Value }}
          {{ end }}
          {{ end }}

  - name: 'email-critical'
    email_configs:
      - to: 'oncall@your-domain.com'
        subject: '[CRITICAL] {{ .GroupLabels.alertname }} - {{ .Annotations.summary }}'
        body: |
          🚨 CRITICAL ALERT 🚨

          {{ range .Alerts }}
          Alert: {{ .Annotations.summary }}
          Description: {{ .Annotations.description }}
          Details:
          {{ range .Labels.SortedPairs }} • {{ .Name }}: {{ .Value }}
          {{ end }}
          {{ end }}

          Please investigate immediately!
```

## Application Metrics

### Thêm Metrics vào Rust Services

```rust
// Trong Cargo.toml
[dependencies]
prometheus = "0.13"
lazy_static = "1.4"

// Trong service code
use prometheus::{Encoder, TextEncoder, register_counter, register_histogram, register_gauge};
use lazy_static::lazy_static;

lazy_static! {
    static ref HTTP_REQUESTS_TOTAL: prometheus::CounterVec = register_counter!(
        "http_requests_total",
        "Total number of HTTP requests",
        &["method", "endpoint", "status"]
    ).unwrap();

    static ref HTTP_REQUEST_DURATION: prometheus::HistogramVec = register_histogram!(
        "http_request_duration_seconds",
        "HTTP request duration in seconds",
        &["method", "endpoint"]
    ).unwrap();

    static ref ACTIVE_CONNECTIONS: prometheus::Gauge = register_gauge!(
        "active_connections",
        "Number of active connections"
    ).unwrap();
}

// Middleware để track metrics
pub async fn metrics_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();

    let response = next.run(req).await;

    let duration = start.elapsed();
    let status = response.status().as_u16().to_string();

    HTTP_REQUESTS_TOTAL
        .with_label_values(&[method.as_str(), uri.path(), &status])
        .inc();

    HTTP_REQUEST_DURATION
        .with_label_values(&[method.as_str(), uri.path()])
        .observe(duration.as_secs_f64());

    Ok(response)
}

// Metrics endpoint
pub fn metrics_route() -> Router {
    Router::new().route("/metrics", get(|| async {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        (
            StatusCode::OK,
            [("content-type", "text/plain; charset=utf-8")],
            String::from_utf8(buffer).unwrap(),
        )
    }))
}
```

### Health Check Endpoint

```rust
// Health check với database connectivity
pub async fn health_check(db: Extension<PgPool>) -> Result<Json<HealthResponse>, AppError> {
    // Check database connection
    let db_status = match sqlx::query("SELECT 1").execute(&db).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy"
    };

    // Check Redis connection (if applicable)
    let redis_status = "healthy"; // Implement Redis check

    let response = HealthResponse {
        status: "ok".to_string(),
        timestamp: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        services: HashMap::from([
            ("database".to_string(), db_status.to_string()),
            ("redis".to_string(), redis_status.to_string()),
        ]),
    };

    Ok(Json(response))
}
```

## Scripts Tự Động Hóa

### Start Monitoring Script

```bash
#!/bin/bash
# scripts/start-monitoring.sh

set -e

echo "🚀 Starting Anthill Monitoring Stack..."

# Start monitoring services
docker-compose -f infra/docker_compose/docker-compose.monitoring.yml up -d

echo "⏳ Waiting for services to be ready..."
sleep 30

# Check service health
echo "🔍 Checking service health..."

# Prometheus
if curl -f http://localhost:9090/-/ready; then
    echo "✅ Prometheus is ready"
else
    echo "❌ Prometheus is not ready"
fi

# Grafana
if curl -f http://localhost:3000/api/health; then
    echo "✅ Grafana is ready"
else
    echo "❌ Grafana is not ready"
fi

# Loki
if curl -f http://localhost:3100/ready; then
    echo "✅ Loki is ready"
else
    echo "❌ Loki is not ready"
fi

echo "🎉 Monitoring stack started successfully!"
echo ""
echo "Access URLs:"
echo "- Prometheus: http://localhost:9090"
echo "- Grafana: http://localhost:3000 (admin/admin)"
echo "- AlertManager: http://localhost:9093"
echo "- Loki: http://localhost:3100"
```

### Health Check Script

```bash
#!/bin/bash
# scripts/monitoring-health-check.sh

echo "🏥 Anthill Health Check"
echo "======================"

# Check Docker services
echo "🐳 Docker Services:"
docker-compose -f infra/docker_compose/docker-compose.monitoring.yml ps

echo ""
echo "📊 Service Health:"

# Prometheus targets
echo "Prometheus targets:"
curl -s http://localhost:9090/api/v1/targets | jq '.data.activeTargets[] | select(.health != "up") | {job: .labels.job, health: .health, lastError: .lastError}'

# Grafana health
echo "Grafana health:"
curl -s http://localhost:3000/api/health

# Application health checks
echo ""
echo "🔍 Application Health:"

# User service
if curl -f http://localhost:8000/health; then
    echo "✅ User Service: Healthy"
else
    echo "❌ User Service: Unhealthy"
fi

# Inventory service
if curl -f http://localhost:8001/health; then
    echo "✅ Inventory Service: Healthy"
else
    echo "❌ Inventory Service: Unhealthy"
fi

# Other services...
```

## Troubleshooting

### Common Issues

1. **Prometheus không scrape được metrics**:
   ```bash
   # Check service is running
   curl http://localhost:8000/metrics

   # Check Prometheus targets
   curl http://localhost:9090/api/v1/targets
   ```

2. **Grafana không kết nối được Prometheus**:
   - Check data source configuration
   - Verify Prometheus URL: `http://prometheus:9090`

3. **Logs không hiển thị trong Loki**:
   ```bash
   # Check Promtail status
   docker-compose logs promtail

   # Check Loki ingestion
   curl http://localhost:3100/loki/api/v1/label
   ```

4. **Alerts không được gửi**:
   ```bash
   # Check AlertManager configuration
   curl http://localhost:9093/api/v2/status

   # Check Prometheus alert rules
   curl http://localhost:9090/api/v1/rules
   ```

### Performance Tuning

```yaml
# Prometheus performance settings
global:
  scrape_interval: 15s
  evaluation_interval: 15s

# Storage settings
storage:
  tsdb:
    retention.time: 30d
    retention.size: 50GB

# Resource limits
resources:
  requests:
    memory: 2Gi
    cpu: 1000m
  limits:
    memory: 4Gi
    cpu: 2000m
```

## Production Considerations

### Security

- **Network Isolation**: Monitoring stack chỉ accessible từ internal network
- **Authentication**: Enable Grafana authentication
- **TLS**: Configure TLS cho tất cả monitoring endpoints
- **Secrets**: Store credentials trong environment variables

### Backup

- **Grafana Dashboards**: Export dashboards as JSON
- **Prometheus Data**: Configure long-term storage (Thanos, Cortex)
- **Loki Logs**: Configure S3 storage cho log persistence

### Scaling

- **Prometheus Federation**: Cho multi-region setups
- **Loki Clustering**: Cho high-volume logging
- **AlertManager Clustering**: Cho high availability

---

**Last Updated**: November 4, 2025
**Stack Versions**: Prometheus 2.45+, Grafana 10.0+, Loki 2.8+
