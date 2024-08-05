# zero2prod_axum

"제로부터 시작하는 러스트 백엔드 프로그래밍"( <https://github.com/LukeMathWalker/zero-to-production> ) 연습용 저장소이다.

actix-web 대신 axum ( <https://docs.rs/axum/latest/axum/> )으로 작성했다.

## 준비물

1. cargo
1. `cargo install sqlx-cli`
1. 도커

## 도커

### 이미지 생성

`docker build --tag zero2prod_axum --file Dockerfile .`

### 이미지 실행

`docker run -p 8000:8000 zero2prod_axum`

## API

- /health_check

  작동 상태를 확인한다.

  `curl http://127.0.0.1:8000/health_check --verbose`

  - 200 OK

- /subscriptions

  뉴스레터 구독을 요청한다.

  `curl --request POST --data 'email=thomas_mann@hotmail.com&name=Tom' http://127.0.0.1:8000/subscriptions --verbose`

  - 200 OK
  - 500 Internal Server Error

  `curl --request POST --data 'email=thomas_mannotmail.com&name=Tom' http://127.0.0.1:8000/subscriptions --verbose`

  - 400 Bad Request

- /subscriptions/confirm

  확인 이메일의 링크를 통해서 이메일 주소의 유효성을 확인한다.

  `curl 'http://127.0.0.1:8000/subscriptions/confirm?subscription_token=token' --verbose`

  - 200 OK
  - 401 Unauthorized
  - 500 Internal Server Erorr

  `curl 'http://127.0.0.1:8000/subscriptions/confirm?subscriptions_token=token' --verbose`

  - 400 Bad Request

- /newsletters

  뉴스레터를 전송한다.

  `curl --request POST --header 'Content-Type: application/json' --data '{"title": "title", "content": {"html": "<p>html</p>", "text": "text"}}' 'http://127.0.0.1:8000/newsletters' --verbose`

  - 200 OK
  - 500 Internal Server Error

## Web

- /login

  로그인을 한다.

  ID: admin\
  비밀번호: everythinghastostartsomewhere

  - 303 See Other -> /admin/dashboard\
    => 로그인 성공

  - 303 See Other -> /login\
    => "사용자 확인을 실패했습니다."

  - 500 Internal SErver Error

- /admin/dashboard

  대시보드

  - 200 OK

  - 303 See Other -> /login\
    => 로그인하지 않고 접속

  - 303 See Other -> /admin/password\
    => "새로운 비밀번호가 일치하지 않습니다."\
    => "비밀번호는 12자 이상이어야 합니다."\
    => "비밀번호는 128자 이하이어야 합니다."\
    => "비밀번호를 잘못 입력했습니다."

  - 500 Internal Server Error

- /admin/password

  비밀번호를 변경한다.

  - 200 OK

  - 303 See Other -> /login\
    => 로그인하지 않고 접속

- /admin/newsletters

  뉴스레터를 전송한다.

  - 200 OK

  - 303 See Other -> /login\

  - 303 See Other -> /admin/newsletters\
    => "내용을 모두 입력해야 합니다."
    => "이메일 전송을 완료했습니다."

## scripts

### init_db.sh

- 테스트를 위한 Postgres 컨테이너 생성 및 마이그레이션을 한다.  
  `./scripts/init_db.sh`

  - 환경변수:
    - `SKIP_DOCKER`\
      도커 생성을 건너 뛴다.  
       `SKIP_DOCKER=true ./scripts/init_db.sh`

### docker-compose.sh

- 테스트용 docker compose를 실행한다.  
  `./scripts/docker-compose.sh`

## tests

- 환경변수

  - `TEST_LOG`

    테스트 로그를 출력한다.  
    `TEST_LOG=true cargo test health_check_works`

## phc_generator

Go로 작성한 PHC String, uuid 생성기

### 실행

go 컴파일러가 필요하다.

```
cd phc_generator && go get -u && go run phc_generator.go && cd ..
```

실행 결과 예시

```
uuid       : c30fd24a-250b-4855-bdee-156c7c833ae0
password   : 7AqcMm2UDmDu7m4I9aLvqyt7uczwS0w4UfjDqGS78iQznFEk2/aolRmCmdzMTl8iGel/MUiUcoEczW47oFpjZw
salt       : oqzMpf3z27gThieg//vo/g
PHC string : $argon2id$v=19$m=19456,t=2,p=1$b3F6TXBmM3oyN2dUaGllZy8vdm8vZw$WurnVz18P2lm3hd7Vj5n9aPz5ZJYXbFcXLYFhXlysDc
```

### 세부 스위치 확인

`go run phc_generator.go --help`
