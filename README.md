# zero2prod_axum

["제로부터 시작하는 러스트 백엔드 프로그래밍"](https://product.kyobobook.co.kr/detail/S000212216062)을 읽으면서 작성한 코드이다.

저자 깃허브 : [zero-to-production](https://github.com/LukeMathWalker/zero-to-production)

단순한 지식 제공을 넘어서 단계적인 접근법을 책 한권에 잘 압축해 놨다.

# 테스트 환경 설정

## Postgresql

```bash
./scripts/init_db.sh
```

## Docker Compose

클라우드 대신에 빌드 테스트를 해보려고 만들었다.  
환경변수를 수정해야 한다.

# 환경변수

- TEST_LOG=true: 테스트에 로그를 표시한다.
- SKIP_DOCKER=true : `./scripts/init_db.sh`를 실행할 때 도커 컨테이너 생성을 건너뛴다.

# 이번 코드의 컨셉

이전에 읽었던 [러스트 백 엔드 (Axum)](https://text.ibetter.kr/rust-axum)의 코드를 반영했다.
