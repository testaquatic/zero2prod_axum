#!/usr/bin/env bash
set -x
set -eo pipefail

>&2 echo "Starting containers."
docker compose up -d

>&2 echo "Running migration."
SKIP_DOCKER=true ./scripts/init_db.sh
>&2 echo "The containers have been started."
>&2 eoch "To stop containers, use \"docker compose down\""

docker compose logs -f -t zero2prod_axum
