#!/bin/sh
set -e

echo "entry point started"

if [ -z "$DATABASE_URL" ]; then
  echo "❌ ERROR: DATABASE_URL is not set!"
  exit 1
fi

echo "=== Waiting for database to be ready ==="
timeout=30
while ! pg_isready -d "$DATABASE_URL" -q; do
  sleep 1
  timeout=$((timeout - 1))
  if [ $timeout -le 0 ]; then
    echo "❌ ERROR: Database is not ready after 30 seconds, exiting."
    exit 1
  fi
done

echo "✅ Database is ready"

echo "=== Running migrations with Goose ==="
goose -dir /migrations postgres "$DATABASE_URL" up

echo "=== Starting backend ==="
/usr/local/bin/app
