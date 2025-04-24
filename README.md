# zero2prod_axum

["제로부터 시작하는 러스트 백엔드 프로그래밍"](https://product.kyobobook.co.kr/detail/S000212216062)을 읽고 작성한 코드이다.  
저자 깃허브: [https://github.com/LukeMathWalker/zero-to-production](https://github.com/LukeMathWalker/zero-to-production)  

__actix-web 대신 [axum]( <https://docs.rs/axum/latest/axum/> )으로 작성했다.__

# Endpoint

## /heath_check

GET 요청을 받으면 바디가 없는 200 OK 응답을 반환한다.

- 요청
  - GET /heath_check

    ```
    http -v http://127.0.0.1:8000/health_check
    ```

- 응답
  - 200 OK

## /subscriptions
- 요청
  - POST /subscriptions/name={name}&email={email}  
    application/x-www-form-urlencoded

- 응답
  - 200 OK  
    유효한 이름과 이메일 제공

  - 422 UNPROCESSABLE ENTITY  
    이름이나 이메일이 누락됨