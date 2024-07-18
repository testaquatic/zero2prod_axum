# zero2prod_axum

"제로부터 시작하는 러스트 백엔드 프로그래밍"( https://github.com/LukeMathWalker/zero-to-production ) 연습용 저장소

actix-web 대신 axum ( https://docs.rs/axum/latest/axum/ )으로 작성했다.

## 준비물

1. cargo
1. `cargo install sqlx-cli`
1. 도커

## 참고사항

### /scripts

- `./scripts/init_db.sh`
  테스트를 위한 Postgres 컨테이너 생성 및 마이그레이션
  - 환경변수
    - `SKIP_DOCKER=true`  
      도커 생성을 건너 뛴다.
