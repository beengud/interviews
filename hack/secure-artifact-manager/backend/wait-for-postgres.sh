#!/bin/sh

set -e

host="${POSTGRES_HOST:-postgres}"
port="${POSTGRES_PORT:-5432}"

echo "Waiting for Postgres at $host:$port..."

# Wait until Postgres is ready
until nc -z "$host" "$port"; do
  echo "Postgres is unavailable - sleeping"
  sleep 1
done

echo "Postgres is up - executing command"
exec "$@"