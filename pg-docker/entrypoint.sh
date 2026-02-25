#!/bin/bash
set -e

# Default value for POSTGRES_DB if not set
POSTGRES_DB=${POSTGRES_DB:-postgres}

# Append the cron.database_name configuration to postgresql.conf.sample
echo "cron.database_name='${POSTGRES_DB}'" >> /usr/share/postgresql/postgresql.conf.sample

# Execute the original entrypoint script
exec docker-entrypoint.sh "$@"