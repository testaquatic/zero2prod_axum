# zero2prod_axum

["제로부터 시작하는 러스트 백엔드 프로그래밍"](https://product.kyobobook.co.kr/detail/S000212216062)을 읽으면서 작성한 코드이다.  
저자 깃허브: [https://github.com/LukeMathWalker/zero-to-production](https://github.com/LukeMathWalker/zero-to-production)  

__[actix-web]( <https://docs.rs/actix-web/latest/actix_web/> ) 대신 [axum]( <https://docs.rs/axum/latest/axum/> )으로 작성했다.__  
__이중 중괄호( `{{이중 중괄호}}` )부분은 환경에 맞춰서 치환한다.__

# Endpoint

## `/heath_check`

### `GET`  

바디가 없는 `200 OK` 응답을 반환한다.

- 요청
  ```
  http -v http://127.0.0.1:8000/health_check
  ```

- 응답
  - `200 OK`

## `/subscriptions`

### `POST`

- 요청
    ```
    http -v --form POST localhost:8000/subscriptions email={{email}} name={{name}}
    ```

- 응답
  - `200 OK`  
    유효한 이름과 이메일 제공  
    - 이름의 유효성
      1. 앞뒤의 공백을 제외하고 1자이상 256자 이하여야 한다.
      2. '/', '(', ')', '"', '<', '>', '\\', '{', '}', ';', ':', '|'를 포함하지 않아야 한다.

  - `400 BAD REQUEST`  
    유효하지 않은 이름이나 이메일을 입력했다.

  - `422 UNPROCESSABLE ENTITY`  
    이름이나 이메일 필드가 누락됐다.

  - `500 INTERNAL_SERVER_ERROR`  
    내부 오류가 발생했다.

# 도커

## 이미지 빌드

```
docker build --tag zero2prod_axum --file Dockerfile .
```

## 테스트를 위한 docker-compose

- 시작
  ```
  docker-compose up -d
  ```

- 종료와 삭제
  ```
  docker-compose down
  ```

# 테스트는 [/go/mock_server/](/go/mock_server/)의 서버를 시작해야 한다.
책의 [wiremock]( <https://docs.rs/wiremock/latest/wiremock/index.html> ) 대신에 직접 GO로 코드를 작성했다.  
```
cd go/mock_server/ && go run main.go; cd ../..
```

# 데이터베이스 설정

PostgreSQL을 사용한다.
```
export {{DATABASE_URL}}
sqlx database create
sqlx migrate run
```

# 편의성을 위한 코드
[/go/](/go/)를 참고한다.
