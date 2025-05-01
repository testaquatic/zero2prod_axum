# mock_server
Postmark API를 모사한다.

# 요청
```
http --json -v POST 127.0.0.1:8800/email Accept:application/json Content-Type:application/json X-Postmark-Server-Token:"server token" From="sender@example.com" To=receiver@example.com Subject="Postmark test" TextBody="Hello dear Postmark user." HtmlBody="<html><body><strong>Hello</strong> dear Postmark user.</body></html>"
```

# 보낸 요청 확인
```
http --json -v 127.0.0.1:8800/debug Content-Type:application/json command=get uuid={uuid}
```