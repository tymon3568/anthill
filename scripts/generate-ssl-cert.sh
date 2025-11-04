#!/bin/bash
# Production SSL Certificate Generation Script for Anthill SaaS
# This script generates production-ready SSL certificates using Let's Encrypt or self-signed

set -e

# Configuration
DOMAIN=${DOMAIN:-"your-domain.com"}
EMAIL=${EMAIL:-"admin@your-domain.com"}
CERT_DIR=${CERT_DIR:-"/etc/ssl/anthill"}
STAGING=${STAGING:-false}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Create certificate directory
create_cert_dir() {
    log_info "Creating certificate directory: $CERT_DIR"
    sudo mkdir -p "$CERT_DIR"
    sudo chmod 755 "$CERT_DIR"
}

# Generate self-signed certificate (for development/testing)
generate_self_signed() {
    log_info "Generating self-signed certificate for domain: $DOMAIN"

    # Create private key
    sudo openssl genrsa -out "$CERT_DIR/anthill.key" 4096

    # Create certificate signing request
    sudo openssl req -new -key "$CERT_DIR/anthill.key" -out "$CERT_DIR/anthill.csr" \
        -subj "/C=US/ST=Production/L=Production/O=Anthill/CN=$DOMAIN"

    # Create config for SAN
    cat > /tmp/anthill-openssl.cnf <<EOF
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req
prompt = no

[req_distinguished_name]
C = US
ST = Production
L = Production
O = Anthill
CN = $DOMAIN

[v3_req]
subjectAltName = @alt_names

[alt_names]
DNS.1 = $DOMAIN
DNS.2 = *.$DOMAIN
EOF

    # Generate self-signed certificate with SAN (valid for 365 days)
    sudo openssl x509 -req -days 365 -in "$CERT_DIR/anthill.csr" \
        -signkey "$CERT_DIR/anthill.key" -out "$CERT_DIR/anthill.crt" \
        -extensions v3_req -extfile /tmp/anthill-openssl.cnf

    # Set proper permissions
    sudo chmod 600 "$CERT_DIR/anthill.key"
    sudo chmod 644 "$CERT_DIR/anthill.crt"

    # Clean up
    sudo rm -f "$CERT_DIR/anthill.csr"
    rm -f /tmp/anthill-openssl.cnf

    log_info "Self-signed certificate generated successfully"
    log_info "Certificate: $CERT_DIR/anthill.crt"
    log_info "Private key: $CERT_DIR/anthill.key"
}

# Generate Let's Encrypt certificate (for production)
generate_letsencrypt() {
    log_info "Generating Let's Encrypt certificate for domain: $DOMAIN"

    # Check if certbot is installed
    if ! command -v certbot &> /dev/null; then
        log_error "Certbot is not installed. Please install it first:"
        log_error "Ubuntu/Debian: sudo apt install certbot"
        log_error "CentOS/RHEL: sudo yum install certbot"
        exit 1
    fi

    # Determine certbot command based on staging flag
    CERTBOT_CMD="sudo certbot certonly --standalone"
    if [ "$STAGING" = true ]; then
        CERTBOT_CMD="$CERTBOT_CMD --staging"
        log_warn "Using Let's Encrypt STAGING environment (test certificates)"
    fi

    # Generate certificate
    $CERTBOT_CMD -d "$DOMAIN" --email "$EMAIL" --agree-tos --non-interactive

    # Copy certificates to our directory
    sudo cp "/etc/letsencrypt/live/$DOMAIN/fullchain.pem" "$CERT_DIR/anthill.crt"
    sudo cp "/etc/letsencrypt/live/$DOMAIN/privkey.pem" "$CERT_DIR/anthill.key"

    # Set proper permissions
    sudo chmod 600 "$CERT_DIR/anthill.key"
    sudo chmod 644 "$CERT_DIR/anthill.crt"

    log_info "Let's Encrypt certificate generated successfully"
    log_info "Certificate: $CERT_DIR/anthill.crt"
    log_info "Private key: $CERT_DIR/anthill.key"
}

# Setup auto-renewal for Let's Encrypt
setup_auto_renewal() {
    log_info "Setting up automatic certificate renewal"

    # Create renewal script
    cat > /tmp/renew-anthill-cert.sh <<EOF
#!/bin/bash
# Renew Anthill SSL certificate and reload services

set -e

DOMAIN="$DOMAIN"
CERT_DIR="$CERT_DIR"

# Renew certificate
certbot renew

# Copy renewed certificates
cp "/etc/letsencrypt/live/\$DOMAIN/fullchain.pem" "\$CERT_DIR/anthill.crt"
cp "/etc/letsencrypt/live/\$DOMAIN/privkey.pem" "\$CERT_DIR/anthill.key"

# Set proper permissions
chmod 600 "\$CERT_DIR/anthill.key"
chmod 644 "\$CERT_DIR/anthill.crt"

# Reload services (adjust commands based on your deployment)
# systemctl reload nginx
# docker restart anthill-user-service

echo "Certificate renewed successfully"
EOF

    sudo mv /tmp/renew-anthill-cert.sh /usr/local/bin/renew-anthill-cert.sh
    sudo chmod +x /usr/local/bin/renew-anthill-cert.sh

    # Add to crontab for weekly renewal check
    (sudo crontab -l 2>/dev/null; echo "0 12 * * 1 /usr/local/bin/renew-anthill-cert.sh") | sudo crontab -

    log_info "Auto-renewal setup complete"
}

# Verify certificate
verify_certificate() {
    log_info "Verifying certificate..."

    # Check certificate validity
    sudo openssl x509 -in "$CERT_DIR/anthill.crt" -text -noout | head -20

    # Check private key
    sudo openssl rsa -in "$CERT_DIR/anthill.key" -check

    log_info "Certificate verification complete"
}

# Main script
main() {
    echo "🔐 Anthill SSL Certificate Generation"
    echo "===================================="
    echo "Domain: $DOMAIN"
    echo "Email: $EMAIL"
    echo "Certificate Directory: $CERT_DIR"
    echo

    # Validate inputs
    if [ "$DOMAIN" = "your-domain.com" ]; then
        log_error "Please set DOMAIN environment variable to your actual domain"
        exit 1
    fi

    create_cert_dir

    # Choose certificate type
    if [ "$1" = "letsencrypt" ]; then
        generate_letsencrypt
        setup_auto_renewal
    elif [ "$1" = "selfsigned" ]; then
        generate_self_signed
    else
        log_error "Usage: $0 [letsencrypt|selfsigned]"
        log_error "Example: DOMAIN=yourdomain.com EMAIL=admin@yourdomain.com $0 letsencrypt"
        exit 1
    fi

    verify_certificate

    log_info "✅ SSL certificate setup complete!"
    log_info "Update your .env file with:"
    log_info "SSL_CERT_PATH=$CERT_DIR/anthill.crt"
    log_info "SSL_KEY_PATH=$CERT_DIR/anthill.key"
}

main "$@"
