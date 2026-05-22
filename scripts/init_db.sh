#!/usr/bin/env bash
set -x
set -eo pipefail

if ! [ -x "$(command -v psql)" ]; then
  echo >&2 "Error: psql is not installed."
  exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 " cargo install sqlx-cli"
  echo >&2 "to install it."
  exit 1
fi

# 커스텀 유저가 설정되었는지 확인한다.
# 기본값 : 'postgres'
DB_USER=${POSTGRES_USER:=postgres}
# 커스텀 패스워드가 설정되었는지 확인한다.
# 기본값 : 'password'
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
# 커스텀 데이터베이스가 설정되었는지 확인한다.
# 기본값 : 'newsletter'
DB_NAME="${POSTGRES_DB:=newsletter}"
# 커스텀 포트가 설정되었는지 확인한다.
# 기본값 : '5432'
DB_PORT="${POSTGRES_PORT:=5432}"

if [[ -z "${SKIP_DOCKER}" ]]; then
  # 도커를 사용해서 postgres를 구동한다
  docker run \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASSWORD} \
    -e POSTGRES_DB=${DB_NAME} \
    -p "${DB_PORT}":5432 \
    --name zero2prod-db \
    -d \
    postgres:18 \
    postgres -N 1000
fi

export PGPASSWORD="${DB_PASSWORD}"
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
  >&2 echo "postgres still not ready - sleeping"
  sleep 1
done

>&2 echo "postgres is up and running on port ${DB_PORT}"

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
export DATABASE_URL
cargo sqlx database create
cargo sqlx migrate run

>&2 echo "postgres has been migrated, ready to go!"