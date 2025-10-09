# Kiến Trúc Hệ Thống - Inventory SaaS Platform (CapRover Edition)

## 🎯 Triết Lý Kiến Trúc

Kiến trúc này được thiết kế dựa trên triết lý thực dụng: **"Sử dụng công cụ phù hợp nhất cho từng công việc"**. Chúng ta ưu tiên các công cụ hạ tầng phổ biến, hiệu suất cao và đã được chứng minh (`battle-tested`), đồng thời tập trung sức mạnh của **Rust** vào nơi nó tạo ra nhiều giá trị nhất: **core business logic**. Nền tảng triển khai là **CapRover**, một PaaS mạnh mẽ giúp đơn giản hóa tối đa việc vận hành.

- **Đơn giản & Hiệu quả**: Tận dụng tối đa các tính năng tự động của CapRover để giảm thiểu công sức quản lý hạ tầng.
- **Hiệu năng cao**: Sử dụng các công cụ tiêu chuẩn ngành (NGINX, Docker Swarm, PostgreSQL, Redis) kết hợp với các microservice viết bằng Rust.
- **An toàn & Bảo mật**: Tận dụng mạng nội bộ của Docker và các cơ chế bảo mật của CapRover, kết hợp với sự an toàn bộ nhớ của Rust.

## 🏗️ Kiến Trúc Tổng Thể trên CapRover

CapRover xây dựng trên Docker Swarm, cung cấp một môi trường PaaS tiện lợi. Kiến trúc của chúng ta sẽ xoay quanh các khái niệm "App" và "One-Click App" của CapRover.

```
                 Internet
                     │
┌────────────────────▼─────────────────────┐
│              CapRover Cluster            │
│          (1 hoặc nhiều server)           │
│ ┌──────────────────────────────────────┐ │
│ │     CapRover NGINX Ingress Proxy     │ │ (Gateway Tự Động)
│ │   (Load Balancing, SSL, Routing)     │ │
│ └──────────────────┬───────────────────┘ │
│                    │ (Route tới app qua Hostname)
│ ┌──────────────────┴───────────────────┐ │
│ │       Docker Swarm Overlay Network     │ │ (Mạng nội bộ an toàn)
│ │                                      │ │
│ │ ┌──────────────┐   ┌───────────────┐ │ │
│ │ │  Rust Service  │   │ Rust Service  │ │ │
│ │ │   (App 1)      ├─► │   (App 2)     │ │ │  (Core Logic)
│ │ │ inventory-svc  │   │  order-svc    │ │ │
│ │ └──────▲───────┘   └───────▲───────┘ │ │
│ │        │                   │         │ │
│ │ ┌──────┴─────────┐  ┌──────┴───────┐ │ │
│ │ │   PostgreSQL   │  │ NATS / Redis │ │ │  (Stateful Services)
│ │ │ (One-Click App)│  │(One-Click App)│ │ │
│ │ └────────────────┘  └──────────────┘ │ │
│ └──────────────────────────────────────┘ │
└──────────────────────────────────────────┘
```

## 🧩 Chi tiết các thành phần

### 1. Gateway & Routing: NGINX của CapRover

- **Công cụ**: NGINX được tích hợp sẵn và quản lý hoàn toàn bởi CapRover.
- **Vai trò**:
  - **Edge Gateway**: Là điểm vào duy nhất cho tất cả traffic từ internet.
  - **Load Balancer**: Tự động cân bằng tải giữa các instance của một app.
  - **SSL Termination**: Tự động cài đặt và gia hạn chứng chỉ Let's Encrypt.
  - **Routing**: Route traffic đến các service Rust dựa trên tên miền được cấu hình cho mỗi app (ví dụ: `api.yourdomain.com`, `inventory-api.yourdomain.com`).
- **Lợi ích**: Không cần quản lý gateway riêng. Cấu hình cực kỳ đơn giản qua giao diện CapRover.

### 2. Backend Microservices: Rust & Axum

- **Công nghệ**: Rust, Axum, Tokio, SQLx.
- **Vai trò**: Đây là nơi chứa đựng toàn bộ business logic của hệ thống (User, Inventory, Order, Integration...). Mỗi service là một project Rust riêng biệt.
- **Triển khai**:
  1.  Mỗi service có một `Dockerfile` để đóng gói thành một image.
  2.  Trong CapRover, mỗi service được định nghĩa là một "App".
  3.  Kết nối CapRover với GitHub/GitLab, và nó sẽ tự động build và deploy mỗi khi có `git push`.
  4.  Scaling (tăng/giảm số container) được thực hiện dễ dàng qua giao diện.

### 3. Giao tiếp giữa các Service: Docker Swarm Network

- **Công nghệ**: Docker Swarm Overlay Network.
- **Vai trò**: Tạo một mạng ảo riêng tư và an toàn cho tất cả các app trong CapRover.
- **Cách hoạt động**: Các service có thể gọi nhau qua tên app. CapRover tự động tạo một hostname là `srv-<app-name>`. Ví dụ, từ `order-service`, bạn có thể kết nối tới `inventory-service` qua địa chỉ `http://srv-inventory-svc:8000`.
- **Lợi ích**: Đơn giản, an toàn, không cần cấu hình service discovery phức tạp như Consul hay Etcd.

### 4. Database & Message Queue: CapRover One-Click Apps

- **Công nghệ**: Sử dụng kho ứng dụng có sẵn của CapRover.
- **Các lựa chọn**:
  - **Database**: **PostgreSQL** (đã được chứng minh, viết bằng C).
  - **Cache**: **Redis** (tiêu chuẩn ngành, viết bằng C).
  - **Message Queue**: **NATS** (hiệu năng cao, viết bằng Go).
  - **Analytics**: **Cube.js** có thể được triển khai như một app riêng.
- **Triển khai**:
  - Vào mục "One-Click Apps", tìm và triển khai các ứng dụng trên chỉ với vài cú click.
  - CapRover tự động quản lý việc lưu trữ dữ liệu bền vững (persistent storage) cho chúng.

### 5. Authorization: Casbin-rs

- **Công nghệ**: Crate `casbin-rs`.
- **Vai trò**:
  - Vẫn tích hợp trực tiếp vào các microservice Rust (đặc biệt là User Service và API Gateway nếu tự build).
  - Models và policies có thể được lưu trong PostgreSQL, sử dụng `casbin-sqlx-adapter`.
  - Một middleware trong Axum sẽ load enforcer và kiểm tra quyền hạn cho mỗi request.

## 🔧 Technology Stack Summary (CapRover Edition)

### Core Application
- **Backend Language**: Rust (Stable)
- **Web Framework**: Axum
- **Async Runtime**: Tokio
- **Database Driver**: SQLx

### Infrastructure & Platform
- **PaaS**: CapRover
- **Container Orchestration**: Docker Swarm (do CapRover quản lý)
- **API Gateway**: NGINX (do CapRover quản lý)
- **Service Networking**: Docker Swarm Overlay Network

### Stateful Services & Middleware ( развернуто как One-Click Apps)
- **Database**: PostgreSQL
- **Cache**: Redis
- **Message Queue**: NATS
- **Analytics**: Cube

### DevOps
- **CI/CD**: Tích hợp sẵn trong CapRover (Webhook từ Git) hoặc dùng GitHub Actions để build Docker image và trigger deploy trên CapRover.
- **Monitoring**: Netdata (thường có sẵn trong CapRover One-Click Apps).

## 🚀 Quy trình phát triển & triển khai

1.  **Local Dev**: Sử dụng `docker_compose` để mô phỏng môi trường CapRover (các service Rust, Postgres, Redis, NATS).
2.  **Code**: Viết logic cho các microservice bằng Rust.
3.  **Push**: Đẩy code lên GitHub.
4.  **Deploy**: CapRover nhận webhook, tự động build image từ `Dockerfile` và triển khai phiên bản mới.
5.  **Scale/Manage**: Sử dụng giao diện CapRover để theo dõi logs, scaling, và quản lý các biến môi trường.

Kiến trúc này vừa hiện đại, hiệu năng cao, vừa cực kỳ thực tế và dễ vận hành cho đội ngũ nhỏ, cho phép bạn tập trung vào việc xây dựng sản phẩm.
