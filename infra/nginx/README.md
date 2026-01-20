# [DEPRECATED] Anthill Nginx - API Gateway & Reverse Proxy

> ⚠️ **DEPRECATED**: Dự án đã chuyển sang sử dụng **Apache APISIX** làm API Gateway.
> Xem cấu hình mới tại `infra/apisix/`.

---

## Migration Notes

Dự án đã migrate từ NGINX sang Apache APISIX vì các lý do sau:

1. **Dynamic Configuration**: APISIX hỗ trợ thay đổi cấu hình mà không cần restart.
2. **Plugin Ecosystem**: Hệ sinh thái plugins phong phú cho rate limiting, authentication, logging, etc.
3. **Dashboard UI**: Giao diện quản trị trực quan.
4. **Better API Management**: Hỗ trợ tốt hơn cho API versioning và documentation.

## APISIX Configuration

Xem hướng dẫn cấu hình APISIX tại:
- `infra/apisix/README.md`
- `infra/apisix/config.yaml`

## Legacy Files

Các file trong thư mục này được giữ lại để tham khảo:
- `nginx.conf` - Cấu hình NGINX cũ
- `Dockerfile` - Docker build cũ
- `conf.d/` - Cấu hình routing cũ

---

**Last Updated**: January 2026
**Status**: Deprecated - Use Apache APISIX instead
