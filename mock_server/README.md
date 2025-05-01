# mock_server
Postmark API를 모사한다.
__대괄호({})부분은 환경에 맞춰서 치환한다.__

# 엔드포인트

## /health_check
작동을 확인한다.

### GET
바디가 없는 200 OK 응답을 반환한다.

- 요청
  ```
  http -v http://127.0.0.1:8000/health_check
  ```

- 응답
  - 200 OK

## /email
이메일 API를 모사한다.

### POST
- 요청
    ```
    http --json -v POST 127.0.0.1:8800/email \
        Accept:application/json Content-Type:application/json X-Postmark-Server-Token:{TOKEN} \
        From={SENDER_EMAIl} To={RECIPIENT_EMAIL} \
        Subject={SUBJECT} \
        TextBody={TEXT_BODY} \
        HtmlBody={HTML_BODY}
    ```
- 응답
    - 200 OK
        ```
        {
            "To": {RECIPIENT_EMAIL},
            "SubmittedAt": {DATE_TIME},
            "MessageID": {UUID},
            "ErrorCode": {ERROR_CODE:number},
            "Message": {MESSAGE}
        }
        ```

## /debug
요청을 확인한다.
- 요청
    ```
    http --json -v http://localhost:8800/debug Content-Type:application/json command=get uuid={UUID}
    ```
- 응답
    - 200 OK
        ```
        {
            "body": {
                "From": {SENDER_EMAIL},
                "To": {RECIPIENT_EMAIL},
                "HtmlBody": {HTML_BODY},
                "TextBody": {TEXT_BODY},
                "Subject": {SUBJECT}
            },
            "header": {
                "Accept": [
                "application/json"
                ],
                "Accept-Encoding": [
                    {ACCEPT_ENCODING}
                ],
                "Connection": [
                    "keep-alive"
                ],
                "Content-Length": [
                    {CONTENT_LENGTH}
                ],
                "Content-Type": [
                    "application/json"
                ],
                "User-Agent": [
                    {USER_AGENT}
                ],
                "X-Postmark-Server-Token": [
                    {POSTMARK_SERVER_TOKEN}
                ]
            },
            "method": {METHOD}
        }
        ```

## /500/email
모든 요청에 대해서 500 INTERNAL_SERVER_ERROR를 반환한다.

## /delay/email
/email과 동일하지만 응답을 100초 이후에 한다.