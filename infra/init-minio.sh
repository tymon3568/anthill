#!/bin/bash

# MinIO initialization script
# This script creates the necessary buckets for the application

set -e

echo "Waiting for MinIO to be ready..."
until curl -f http://localhost:9000/minio/health/ready; do
  echo "MinIO is not ready yet, waiting..."
  sleep 5
done

echo "MinIO is ready. Creating buckets..."

# Create avatars bucket
echo "Creating 'avatars' bucket..."
mc alias set local http://localhost:9000 minioadmin minioadmin
mc mb local/avatars --ignore-existing

# Set bucket policy to allow public read access for avatars
mc policy set download local/avatars

echo "Buckets created successfully!"
echo "MinIO initialization complete."
