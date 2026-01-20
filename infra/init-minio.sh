#!/bin/bash

# MinIO initialization script
# This script creates the necessary buckets for the application

set -e

echo "Configuring mc alias for MinIO..."

# Use environment variables with defaults
MINIO_ENDPOINT="${MINIO_ENDPOINT:-http://localhost:9000}"
MINIO_ROOT_USER="${MINIO_ROOT_USER:-minioadmin}"
MINIO_ROOT_PASSWORD="${MINIO_ROOT_PASSWORD:-minioadmin}"

# Configure mc alias for MinIO - retry until successful
max_retries=30
retry_count=0

while [ $retry_count -lt $max_retries ]; do
  if mc alias set minio "$MINIO_ENDPOINT" "$MINIO_ROOT_USER" "$MINIO_ROOT_PASSWORD" --api S3v4 2>/dev/null; then
    echo "mc alias configured successfully"
    break
  fi
  echo "MinIO is not ready yet, waiting... (attempt $((retry_count + 1))/$max_retries)"
  retry_count=$((retry_count + 1))
  sleep 2
done

if [ $retry_count -eq $max_retries ]; then
  echo "Failed to connect to MinIO after $max_retries attempts"
  exit 1
fi

echo "Creating buckets..."

# Create avatars bucket
echo "Creating 'avatars' bucket..."
mc mb minio/avatars --ignore-existing || true

# Create documents bucket
echo "Creating 'documents' bucket..."
mc mb minio/documents --ignore-existing || true

# Create anthill-files bucket (main application bucket)
echo "Creating 'anthill-files' bucket..."
mc mb minio/anthill-files --ignore-existing || true

echo "Listing buckets..."
mc ls minio

echo "Buckets created successfully!"
echo "MinIO initialization complete."
