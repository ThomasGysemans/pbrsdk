#!/bin/sh

set -e

if [ -n "$POCKETBASE_EMAIL" ] && [ -n "$POCKETBASE_PASSWORD" ]; then
    echo "Creating user named $POCKETBASE_EMAIL with password $POCKETBASE_PASSWORD"
else
    exit 1
fi

/usr/local/bin/pocketbase --dir /pb_data superuser upsert "$POCKETBASE_EMAIL" "$POCKETBASE_PASSWORD"
exec /usr/local/bin/pocketbase serve --dir /pb_data --http 0.0.0.0:8090