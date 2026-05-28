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

## 키 생성 방법

JWT를 활성화하는데 필요하다.

```bash
openssl genpkey -algorithm ed25519 -out private.pem
openssl pkey -in private.pem -pubout -out public.pem
```

# 환경변수

- TEST_LOG=true: 테스트에 로그를 표시한다.
- SKIP_DOCKER=true : `./scripts/init_db.sh`를 실행할 때 도커 컨테이너 생성을 건너뛴다.

# 이번 코드의 컨셉

1. 이전에 읽었던 [러스트 백 엔드 (Axum)](https://text.ibetter.kr/rust-axum)의 코드를 반영했다.  
1. HTML을 직접 다루지 않고 JSON을 통해서 통신한다.
1. JWT 인증을 사용한다. 

# phc_string_gen

PHC 문자열과 UUID를 생성한다.
편의를 위해서 GO로 작성한 작은 프로그램이다.  
이전에 작성한 코드가 있어서 가져왔다.

```bash
go run ./phc_string_gen.go --help
Usage of phc_string_gen:
  -l uint
        [l]ength (default 32)
  -m uint
        [m]emory (default 19456)
  -p string
        [p]assword
        Automatically generated if not entered
  -s string
        [s]alt
        Automatically generated if not entered
  -t uint
        [t]ime (default 2)
  -th uint
        [th]reads (default 1)
```
