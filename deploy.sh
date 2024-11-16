#!/bin/sh

# Make sure front-end dir exists
mkdir -p public

# shellcheck source=/dev/null
. ./.env
docker-compose -f docker-compose.production.yml -f docker-compose.yml build \
&& docker-compose -f docker-compose.production.yml -f docker-compose.yml up -d \
&& docker-compose logs -f
