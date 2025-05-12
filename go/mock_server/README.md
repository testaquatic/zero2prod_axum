# mock_server
Postmark API를 모사한다.  
__이중 중괄호( `{{이중 중괄호}}` )부분은 환경에 맞춰서 치환한다.__

# 사용법
```
./mock_server --help
```

# `PM_MOCK_HUB`
독립적인 `PM_MOCK_SERVER`를 생성하기 위한 헬퍼이다.

## 타임아웃
`--timeout`스위치를 사용하지 않으면 기본적으로 `10초` 이후에 자동으로 종료한다.  
시간을 연장하려면 `/health_check`에 `GET` 요청을 보내면 `--timeout`에 지정한 만큼 늘어난다.

## `/health_check`
작동을 확인한다.

### `GET`
바디가 없는 `200 OK` 응답을 반환한다.

- 요청
  ```
  http -v http://127.0.0.1:8800/health_check
  ```

- 응답
  - `200 OK`

## `/new_server`
새로운 `PM_MOCK_SERVER`를 생성한다.

```
http -v http://127.0.0.1:8800/new_server
```

- 응답
    - `200 OK`  
        {{PORT}}

# `PM_MOCK_SERVER`
테스트를 위헌 모사 서버이다.

## `/email`
이메일 API를 모사한다.

### `POST`
- 요청
    ```
    http --json -v POST 127.0.0.1:{{PORT}}/email \
        Accept:application/json \
        Content-Type:application/json \ 
        X-Postmark-Server-Token:{{TOKEN}} \
        From={{SENDER_EMAIl}} \
        To={{RECIPIENT_EMAIL}} \
        Subject={{SUBJECT}} \
        TextBody={{TEXT_BODY}} \
        HtmlBody={{HTML_BODY}}
    ```
- 응답
    - `200 OK`
        ```
        {
            "To": {{RECIPIENT_EMAIL}},
            "SubmittedAt": {{DATE_TIME}},
            "MessageID": {{UUID}},
            "ErrorCode": {{ERROR_CODE}},
            "Message": {{MESSAGE}}
        }
        ```

## `/debug`
요청을 확인한다.
- 요청
    ```
    http --json -v http://localhost:{{PORT}}/debug Content-Type:application/json command=get_all
    ```
    ```
    http --json -v http://localhost:{{PORT}}/debug Content-Type:application/json command=get uuid={{UUID}}
    ```
- 응답
    - `200 OK`
        ```
        [{
            "requests": {
                "body": {
                    "From": {{SENDER_EMAIL}},
                    "To": {{RECIPIENT_EMAIL}},
                    "HtmlBody": {{HTML_BODY}},
                    "TextBody": {{TEXT_BODY}},
                    "Subject": {{SUBJECT}}
                },
                "header": {
                    "Accept": [
                        "application/json"
                    ],
                    ...
                    "Content-Type": [
                        "application/json"
                    ],
                    ...
                    "X-Postmark-Server-Token": [
                        {{POSTMARK_SERVER_TOKEN}}
                    ]
                },
                "method": {{METHOD}}
            },
            "uuid": {{UUID}}
        }...]
        ```

## `/500/email`
모든 요청에 대해서 `500 INTERNAL_SERVER_ERROR`를 반환한다.

## `/delay/email`
`/email`과 동일하지만 응답을 100초 이후에 한다.