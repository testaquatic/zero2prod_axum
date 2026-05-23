FROM rust:trixie AS planner

WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:trixie AS builder
# 레이어 캐싱을 이용한다
WORKDIR /app
RUN cargo install cargo-chef

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin zero2prod_axum


FROM debian:trixie-slim AS runtime

WORKDIR /app

RUN apt-get update -y \
  && apt-get install -y --no-install-recommends ca-certificates \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/zero2prod_axum zero2prod_axum
COPY configuration configuration
ENV APP_ENVIRONMENT=production

ENTRYPOINT ["./zero2prod_axum"]