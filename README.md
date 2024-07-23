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
   `curl -v http://127.0.0.1:8000/health_check`

      200 OK

- /subscriptions  
   `curl --request POST --data 'email=thomas_mann@hotmail.com&name=Tom' --verbose http://127.0.0.1:8000/subscriptions`

      200 OK
          => 정상 작동

      500 Internal Server Error
          => 데이터 베이스 오류(이메일 중복)
          => 이메일 전송 실패가 발생했다.

  `curl --request POST --data 'email=thomas_mannotmail.com&name=Tom' --verbose http://127.0.0.1:8000/subscriptions`

      400 Bad Request
          => 필드에 잘못된 값을 입력했다.

  `curl --request POST --data 'email=thomas_mann@hotmail.com' --verbose http://127.0.0.1:8000/subscriptions`

      422 Unprocessable Entity
          => 일부 또는 전체 필드가 없다

## /scripts

### init_db.sh

- 테스트를 위한 Postgres 컨테이너 생성 및 마이그레이션을 한다.  
  `./scripts/init_db.sh`

  - 환경변수:
    - `SKIP_DOCKER`
      도커 생성을 건너 뛴다.  
       `SKIP_DOCKER=true ./scripts/init_db.sh`

### docker-compose.sh

- 테스트를 위해서 docker compose를 실행한다.
  `./scripts/docker-compose.sh`

## /tests

- `TEST_LOG`  
  테스트 할 때 로그를 출력한다.  
   `TEST_LOG=true cargo test health_check_works`
