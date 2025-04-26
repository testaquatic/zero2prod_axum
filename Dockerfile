# 최신 러스트 이미지
FROM rust:latest

# 작업 디렉터리 지정
WORKDIR /app
# 필요한 패키지를 설치한다.
RUN apt update && apt upgrade -y
# 작업 환경의 파일을 도커 이미지로 복사한다.
COPY . .
# sqlx를 오프라인 모드로 설정한다.
ENV SQLX_OFFLINE=true
# 컴파일
RUN cargo build --release
# 실행 환경을 production으로 설정한다.
ENV APP_ENVIRONMENT=production
# 도커 이미지가 실행되면 실행한다.
ENTRYPOINT [ "./target/release/zero2prod_axum" ]
