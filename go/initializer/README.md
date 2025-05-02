# initializer
개발 환경 초기화를 위한 코드이다.

## `/init_db/`
책의 bash 스크립트를 GO로 재작성했다.
Postgres를 초기화한다.
```
./init_db --help
```

## `/docker_compose/`
테스트용 Docker Compose 구동 파일이다.
```
./docker_compose --help
```

## `/mock_server/`
PostMark의 이메일 api를 모사한다.
[/go/mock_server/](mock_server/)를 참고한다.