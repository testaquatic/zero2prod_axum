# 러스트의 최신 릴리즈를 사용한다.
FROM rust:latest AS builder

WORKDIR /app

# 작업 환경의 파일을 도커 이미지로 복사한다.
COPY . .

RUN  cargo build --release

FROM debian:stable-slim AS runtime

WORKDIR /app

RUN apt update -y \
    && apt install -y --no-install-recommends ca-certificates \
    && apt autoremove -y \
    && apt clean -y

COPY --from=builder /app/target/release/zero2prod_axum zero2prod_axum
COPY web/dist web/dist
COPY configuration configuration

# production.json5를 읽는다.
ENV APP_ENVIRONMENT=production

ENTRYPOINT [ "./zero2prod_axum" ]