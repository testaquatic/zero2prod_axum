# Builder 단계
# 최신 러스트 이미지
FROM rust:latest AS builder

# 작업 디렉터리 지정
WORKDIR /app
# 작업 환경의 파일을 도커 이미지로 복사한다.
COPY . .
# sqlx를 오프라인 모드로 설정한다.
ENV SQLX_OFFLINE=true
# 컴파일
RUN cargo build --release

# Runtime 단계
FROM debian:stable-slim AS runtime
WORKDIR /app
# 필요한 의존성을 설치한다.
# OpenSSL - 일부 디펜던시에 의해 동적으로 링크된다.
# ca-certificates - TLS
RUN apt update -y \
    && apt upgrade -y \
    && apt install -y openssl ca-certificates \
    && apt autoremove --purge -y \
    && apt clean -y
# 컴파일한 바이너리를 runtime 환경으로 복사한다.
COPY --from=builder /app/target/release/zero2prod_axum zero2prod_axum
# runtime에서 필요한 설정 파일을 복사한다.
COPY configuration configuration
# 실행 환경을 production으로 설정한다.
ENV APP_ENVIRONMENT=production
# 도커 이미지가 실행되면 실행한다.
ENTRYPOINT [ "./zero2prod_axum" ]
