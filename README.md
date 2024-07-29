# zero2prod_axum

"제로부터 시작하는 러스트 백엔드 프로그래밍"( https://github.com/LukeMathWalker/zero-to-production ) 연습용 저장소이다.

actix-web 대신 axum ( https://docs.rs/axum/latest/axum/ )으로 작성했다.

## 준비물

1. cargo
1. `cargo install sqlx-cli`
1. 도커

## 도커

### 이미지 생성

`docker build --tag zero2prod_axum --file Dockerfile .`

### 이미지 실행

`docker run -p 8000:8000 zero2prod_axum`

## 엔드포인트

- /health_check

  작동 상태를 확인한다.

  `curl http://127.0.0.1:8000/health_check --verbose`

        200 OK

- /subscriptions

  뉴스레터 구독을 요청한다.

  `curl --request POST --data 'email=thomas_mann@hotmail.com&name=Tom' http://127.0.0.1:8000/subscriptions --verbose`

        200 OK

        500 Internal Server Error
            => 데이터 베이스 오류(이메일 중복)
            => 이메일 전송 실패가 발생

  `curl --request POST --data 'email=thomas_mannotmail.com&name=Tom' http://127.0.0.1:8000/subscriptions --verbose`

        400 Bad Request
            => 잘못된 형식의 요청

- /subscriptions/confirm

  확인 이메일의 링크를 통해서 이메일 주소의 유효성을 확인한다.

  `curl 'http://127.0.0.1:8000/subscriptions/confirm?subscription_token=token' --verbose`

        200 OK

        401 Unauthorized
            => 유효하지 않은 토큰

        500 Internal Server Erorr
            => 내부 서버 오류(데이터베이스 등)

  `curl 'http://127.0.0.1:8000/subscriptions/confirm?subscriptions_token=token' --verbose`

        400 Bad Request
            => 잘못된 형식의 요청

- /newsletters

  뉴스레터를 전송한다.

  `curl --request POST --header 'Content-Type: application/json' --data '{"title": "title", "content": {"html": "<p>html</p>", "text": "text"}}' 'http://127.0.0.1:8000/newsletters' --verbose`

        200 OK

        500 Internal Server Error
            => 내부 서버 오류(데이터베이스 오류, 잘못된 이메일 주소, 이메일 전송 실패)

## /scripts

### init_db.sh

- 테스트를 위한 Postgres 컨테이너 생성 및 마이그레이션을 한다.  
  `./scripts/init_db.sh`

  - 환경변수:
    - `SKIP_DOCKER`
      도커 생성을 건너 뛴다.  
       `SKIP_DOCKER=true ./scripts/init_db.sh`

### docker-compose.sh

- 테스트용 docker compose를 실행한다.  
  `./scripts/docker-compose.sh`

## /tests

- 환경변수

  - `TEST_LOG`

    테스트 로그를 출력한다.  
    `TEST_LOG=true cargo test health_check_works`
