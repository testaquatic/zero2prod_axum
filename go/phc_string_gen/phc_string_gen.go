package main

import (
	"crypto/rand"
	"encoding/base64"
	"flag"
	"fmt"

	"github.com/google/uuid"
	"golang.org/x/crypto/argon2"
)

// PHC string 생성을 위한 구조체
type Argon2idGen struct {
	salt,
	password []byte
	time,
	keyLen,
	memory uint32
	threads uint8
}

// Argon2id 해시를 생성한다.
func (gen *Argon2idGen) Generate() []byte {
	return argon2.IDKey(gen.password, gen.salt, gen.time, gen.memory, gen.threads, gen.keyLen)
}

// PHC string은
// $argon2id$v=19$m=65536,t=2,p=1$gZiV/M1gPc22ElAH/Jh1Hw$CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno
// 이런 형식이다.
// https://github.com/P-H-C/phc-string-format/blob/master/phc-sf-spec.md 이 문서를 참고로 했다.
func (gen *Argon2idGen) GetPHCString() string {
	id := gen.Generate()
	return fmt.Sprintf("$argon2id$v=19$m=%d,t=%d,p=%d$%s$%s",
		gen.memory, gen.time, gen.threads, base64.RawStdEncoding.EncodeToString(gen.salt), base64.RawStdEncoding.EncodeToString(id))
}

// 생성에 필요한 필드를 담고 있는 전역 변수
var Gen Argon2idGen

func init() {
	password := flag.String("p", "", "[p]assword\nAutomatically generated if not entered")
	salt := flag.String("s", "", "[s]alt\nAutomatically generated if not entered")
	time := flag.Uint("t", 2, "[t]ime")
	memory := flag.Uint("m", 19*1024, "[m]emory")
	threads := flag.Uint("th", 1, "[th]reads")
	keyLen := flag.Uint("l", 32, "[l]ength")
	flag.Parse()

	// 비밀번호를 입력하지 않으면 자동으로 생성한다.
	if *password == "" {
		*password = generateString(32)[:32]
	}
	// 후추를 입력하지 않으면 자동으로 생성한다.
	if *salt == "" {
		*salt = generateString(8)[:8]
	}

	Gen.salt = []byte(*salt)
	Gen.password = []byte(*password)
	Gen.time = uint32(*time)
	Gen.memory = uint32(*memory)
	Gen.threads = uint8(*threads)
	Gen.keyLen = uint32(*keyLen)
}

// 지정한 길이의 문자열을 생성한다.
// 사실 정확한 길이는 아니다.
func generateString(length int) string {
	buffer := make([]byte, length)
	if _, err := rand.Read(buffer); err != nil {
		panic(err)
	}
	return base64.RawStdEncoding.EncodeToString(buffer)
}

func main() {

	var print_uuid uuid.UUID
	for {
		new_uuid, err := uuid.NewRandom()
		if err == nil {
			print_uuid = new_uuid
			break
		}
	}

	fmt.Printf("Password   : %s\n", Gen.password)
	fmt.Printf("Salt       : %s\n", Gen.salt)
	fmt.Printf("PHC string : %s\n", Gen.GetPHCString())
	fmt.Printf("UUID       : %s\n", print_uuid.String())
}
