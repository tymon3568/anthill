#!/bin/bash

# RustFS initialization script
# This script creates the necessary buckets for the application

set -e

echo "Configuring mc alias for RustFS..."

# Use environment variables with defaults
RUSTFS_ENDPOINT="${RUSTFS_ENDPOINT:-http://localhost:9000}"
RUSTFS_ACCESS_KEY="${RUSTFS_ACCESS_KEY:-rustfsadmin}"
RUSTFS_SECRET_KEY="${RUSTFS_SECRET_KEY:-rustfsadmin}"

# Configure mc alias for RustFS - retry until successful
max_retries=30
retry_count=0

while [ $retry_count -lt $max_retries ]; do
  if mc alias set rustfs "$RUSTFS_ENDPOINT" "$RUSTFS_ACCESS_KEY" "$RUSTFS_SECRET_KEY" --api S3v4 2>/dev/null; then
    echo "mc alias configured successfully"
    break
  fi
  echo "RustFS is not ready yet, waiting... (attempt $((retry_count + 1))/$max_retries)"
  retry_count=$((retry_count + 1))
  sleep 2
done

if [ $retry_count -eq $max_retries ]; then
  echo "Failed to connect to RustFS after $max_retries attempts"
  exit 1
fi

echo "Creating buckets..."

# Create avatars bucket
echo "Creating 'avatars' bucket..."
mc mb rustfs/avatars --ignore-existing || true

# Create documents bucket
echo "Creating 'documents' bucket..."
mc mb rustfs/documents --ignore-existing || true

# Create anthill-files bucket (main application bucket)
echo "Creating 'anthill-files' bucket..."
mc mb rustfs/anthill-files --ignore-existing || true

# Set public download policy for anthill-files bucket (for avatars, etc.)
echo "Setting public download policy for 'anthill-files' bucket..."
mc anonymous set download rustfs/anthill-files || true

echo "Listing buckets..."
mc ls rustfs

echo "Buckets created successfully!"
echo "RustFS initialization complete."
