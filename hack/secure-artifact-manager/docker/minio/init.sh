#!/bin/sh

set -e

# Wait for MinIO server to be ready
sleep 5

# Create buckets
mc alias set local http://localhost:9000 "$MINIO_ROOT_USER" "$MINIO_ROOT_PASSWORD"

mc mb -p local/artifacts || echo "Bucket already exists"