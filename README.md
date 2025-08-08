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

1과 2의 작업은 ./go/init_db.go를 사용한다.

```
go run ./go/init_db.go
```

```
go run ./go/init_db.go -h
Usage of init_db
  -skip-docker
        Skip Docker
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

예시

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
}
```

## API

### /health_check

-   GET

    ```
    http -v GET http://localhost:8000/health_check
    ```

    -   응답

        -   200 OK

---

### /subscriptions

-   POST

    x-www-form-urlencoded  
     name과 email 필드는 반드시 있어야 한다.

    ```
    http -v --form POST http://127.0.0.1:8000/subscriptions \
    Content-Type:application/x-www-form-urlencoded \
    email=thomas_mann@hotmail.com \
    name=Tom
    ```

## Dcokerfile

이미지 생성

```
docker buildx build --file Dockerfile .
```
