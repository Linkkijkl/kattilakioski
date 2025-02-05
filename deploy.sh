#!/bin/sh

# Make sure front-end data dir exists
mkdir -p public

# Load environment variables
# shellcheck source=/dev/null
. ./.env

# Build, run and view output
docker compose -f docker-compose.production.yml -f docker-compose.yml build \
&& docker compose -f docker-compose.production.yml -f docker-compose.yml up -d \
&& docker compose -f docker-compose.production.yml -f docker-compose.yml logs -f

# Running this would take the server stack down
#docker compose -f docker-compose.production.yml -f docker-compose.yml down
