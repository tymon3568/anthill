#!/bin/sh
# Generate self-signed certificates for Kanidm development
# This script creates certificates that Kanidm can use for HTTPS

# Create certificate directory if it doesn't exist
mkdir -p /data

# Generate private key
openssl genrsa -out /data/key.pem 2048

# Create OpenSSL config with SAN
cat > /tmp/openssl.cnf <<EOF
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req
prompt = no

[req_distinguished_name]
C = US
ST = Dev
L = Dev
O = Anthill
CN = localhost

[v3_req]
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
DNS.2 = kanidm
DNS.3 = *.kanidm
IP.1 = 127.0.0.1
EOF

# Generate certificate signing request with SAN
openssl req -new -key /data/key.pem -out /data/cert.csr -config /tmp/openssl.cnf

# Generate self-signed certificate with SAN (valid for 365 days)
openssl x509 -req -days 365 -in /data/cert.csr -signkey /data/key.pem \
    -out /data/chain.pem -extensions v3_req -extfile /tmp/openssl.cnf

# Set proper permissions
chmod 600 /data/key.pem
chmod 644 /data/chain.pem

# Clean up
rm /data/cert.csr /tmp/openssl.cnf

echo "✅ Certificates generated with SAN:"
echo "   Private key: /data/key.pem"
echo "   Certificate: /data/chain.pem"
echo "   SAN: DNS:localhost, DNS:kanidm, IP:127.0.0.1"
