# Anthill Nginx - Production API Gateway
FROM nginx:alpine

# Install additional tools for health checks and SSL
RUN apk add --no-cache \
    curl \
    openssl \
    && rm -rf /var/cache/apk/*

# Copy nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

# Copy configuration includes
COPY conf.d/ /etc/nginx/conf.d/

# Create SSL certificate directory
RUN mkdir -p /etc/ssl/anthill && \
    chmod 755 /etc/ssl/anthill

# Create log directory
RUN mkdir -p /var/log/nginx && \
    chown nginx:nginx /var/log/nginx

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost/health || exit 1

# Expose ports
EXPOSE 80 443

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
