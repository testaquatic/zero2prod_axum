# zero2prod_axum

["제로부터 시작하는 러스트 백엔드 프로그래밍"](https://product.kyobobook.co.kr/detail/S000212216062)을 읽으면서 작성한 코드이다.

저자 깃허브 : [zero-to-production](https://github.com/LukeMathWalker/zero-to-production)

[Actix Web](https://actix.rs/)과 [sqlx](https://docs.rs/sqlx/latest/sqlx/) 대신 [axum](https://github.com/tokio-rs/axum), [sea-orm](https://www.sea-ql.org/SeaORM/)을 사용했다.

내용이 알찬 책이다.  
언어의 문법을 배우면 뭔가를 만들어 보고 싶은 생각이 든다.  
단순한 지식 제공을 넘어서 단계적인 접근법을 책 한권에 잘 압축해 놨다.

## 실행 환경 설정

### POSTGRES

1. 도커 이미지 생성

2. 데이터베이스 마이그레이션  
   자세한 내용은 [Setting Up Migration](https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/) 문서를 참고한다.

1과 2의 작업은 ./go/init_db.go를 사용해서 쉽게 할 수 있다.

```bash
go run ./go/init_db.go
```

```bash
go run ./go/init_db.go --help
Usage of ./go/init_db:
  -entity string
        Specifies the directory in which to store a entity. If not specified, the entity will not be created.
  -skip-docker
        Skip Docker setup
```

### /configuration

설정 파일을 저장하는 디렉토리이다.  
json5 형식으로 저장해야 한다.

-   base.json5  
    기본적인 설정을 지정한다.  
    가장 순위가 낮다.

-   local.json5
    APP_ENVIRONMENT 환경변수가 설정되어 있지 않았을 때 읽는다.  
    base.json5보다 순위가 높다.

-   {APP_ENVIRONMENT}.json5  
    APP_ENVIRONMENT 환경변수가 'production'이라면 'production.json5'을 읽는다.
    local.json5보다 순위가 높다.

높은 순위의 설정이 우선 적용된다.

```json5
{
    application: {
        host: "127.0.0.1",
        port: 8000,
    },
    database: {
        host: "127.0.0.1",
        port: 5432,
        username: "postgres",
        password: "password",
        database_name: "newsletter",
        require_ssl: true,
    },
    email_client: {
        base_url: "localhost",
        sender_email: "test@gmail.com",
        authorization_token: "token",
        timeout_milliseconds: 10000,
    },
}
```

## API

### /health_check

-   GET

    ```bash
    http -v GET http://localhost:8000/health_check
    ```

    -   응답

        -   200 OK  
            응답본문은 비어 있다.

---

### /subscriptions

#### POST

    ```bash
    http -v --form POST http://127.0.0.1:8000/subscriptions \
    Content-Type:application/x-www-form-urlencoded \
    email=thomas_mann@hotmail.com \
    name=Tom
    ```

-   x-www-form-urlencoded  
    name과 email 필드는 반드시 있어야 한다.

    하나 이상의 필수적인 필드가 비어 있을 때는 422 Unprocessable Entity를 반환한다.

-   name 필드

    1. 최소길이는 1이다.
    1. 공백문자만 넣을 수 없다.
    1. 256자 이하이어야 한다.
    1. '/', '(', ')', '"', '<', '>', '\\', '{', '}', ';'은 넣을 수 없다.

    유효성 검증에 실패하면 400 Bad Request를 반환한다.

-   email 필드

    유효한 형식의 이메일 주소를 입력해야 한다.

-   응답

    1. 200 OK

    2. 400 Bad Request

    3. 422 Unprocessable Entity

    4. 500 Internal Server Error  
       데이터베이스 오류  
       이메일 전송 오류

## Dcokerfile

이미지 생성

```bash
docker buildx build --tag zero2prod_axum  --file Dockerfile .
```

이미지 실행

```bash
docker run -p 8000:8000 zero2prod_axum
```
