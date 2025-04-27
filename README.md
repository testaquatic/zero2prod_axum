# zero2prod_axum

["제로부터 시작하는 러스트 백엔드 프로그래밍"](https://product.kyobobook.co.kr/detail/S000212216062)을 읽고 작성한 코드이다.  
저자 깃허브: [https://github.com/LukeMathWalker/zero-to-production](https://github.com/LukeMathWalker/zero-to-production)  

__actix-web 대신 [axum]( <https://docs.rs/axum/latest/axum/> )으로 작성했다.__

# Endpoint

## /heath_check

GET 요청을 받으면 바디가 없는 200 OK 응답을 반환한다.

#### 요청
  - GET /heath_check

    ```
    http -v http://127.0.0.1:8000/health_check
    ```

#### 응답
  - 200 OK

## /subscriptions
### POST
#### 요청
  - /subscriptions/name={name}&email={email}  
    body: application/x-www-form-urlencoded

    ```
    http -v --form POST localhost:8000/subscriptions email={email} name={name}
    ```
#### 응답
  - 200 OK  
    유효한 이름과 이메일 제공  
    - 이름의 유효성
      1. 앞뒤의 공백을 제외하고 1자이상 256자 이하여야 한다.
      2. '/', '(', ')', '"', '<', '>', '\\', '{', '}', ';', ':', '|'를 포함하지 않아야 한다.

  - 422 UNPROCESSABLE ENTITY  
    이름이나 이메일 필드가 누락됐다.

# 도커

## 이미지 빌드

```
docker build --tag zero2prod_axum --file Dockerfile .
```

# 데이터베이스 마이그레이션
PostgreSQL을 사용한다.
```
export {DATABASE_URL}
sqlx database create
sqlx migrate run
```

# 편의성을 위한 코드
[/scripts](/scripts/)를 참고한다.