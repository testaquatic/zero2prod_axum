# zero2prod_axum

["제로부터 시작하는 러스트 백엔드 프로그래밍"](https://product.kyobobook.co.kr/detail/S000212216062)을 읽으면서 작성한 코드이다.

저자 깃허브 : [zero-to-production](https://github.com/LukeMathWalker/zero-to-production)

[Actix Web](https://actix.rs/) 대신 [axum](https://github.com/tokio-rs/axum)을 사용했다.  
[sqlx](https://docs.rs/sqlx/latest/sqlx/) 대신 [sea-orm](https://www.sea-ql.org/SeaORM/)을 사용했다.

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


### configuration.json5

-   JSON5 형식이다.


예시
```json5
{
    application_port: 8000,
    database: {
        host: "127.0.0.1",
        port: 5432,
        username: "postgres",
        password: "password",
        database_name: "newsletter",
    },
}
```

## API

### /health_check

-   GET

    ```
    http GET http://localhost:8000/health_check -v
    ```

    -   응답

        -   200 OK

---

### /subscriptions

-   POST

    x-www-form-urlencoded  
    name과 email 필드는 반드시 있어야 한다.
