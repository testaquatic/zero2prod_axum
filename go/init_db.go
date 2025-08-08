package main

import (
	"flag"
	"fmt"
	"os"
	"os/exec"
	"time"
)

// 주어진 명령어와 인자를 실행한다.
// 표준 출력과 표준 오류를 현재 프로세스의 출력으로 리디렉션한다.
// 명령어가 실패하면 오류를 반환한다.
func runCommand(command string, args ...string) error {
	cmd := exec.Command(command, args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

// PostgreSQL 데이터베이스를 도커 컨테이너로 실행한다.
func runDocker(dbUser, dbPassword, dbName, dbPort string) error {
	return runCommand(
		"docker", "run",
		"-e", "POSTGRES_USER="+dbUser,
		"-e", "POSTGRES_PASSWORD="+dbPassword,
		"-e", "POSTGRES_DB="+dbName,
		"-p", dbPort+":5432",
		"-d", "postgres", "-N", "1000",
	)

}

// 주어진 명령어가 설치되어 있는지 확인한다.
func checkRequiredCommand(command string) error {
	_, err := exec.LookPath(command)
	if err != nil {
		return fmt.Errorf("%s command not found.\n", command)
	}

	return nil
}

// 도커 설정을 건너뛸지 여부를 나타내는 플래그이다.
var SKIP_DOKER bool

func init() {
	flag.BoolVar(&SKIP_DOKER, "skip-docker", false, "Skip Docker setup")
}

func main() {
	flag.Parse()

	// 필수 명령어가 설치되어 있는지 확인한다.
	// docker, psql, sea-orm-cli가 필요하다.
	if err := checkRequiredCommand("docker"); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %s\n", err)
		os.Exit(1)
	}

	if err := checkRequiredCommand("psql"); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %s\n", err)
		os.Exit(1)
	}

	if err := checkRequiredCommand("sea-orm-cli"); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %s\n", err)
		os.Exit(1)
	}

	// 환경 변수를 읽는다.
	DB_USER := os.Getenv("POSTGRES_USER")
	DB_PASSWORD := os.Getenv("POSTGRES_PASSWORD")
	DB_NAME := os.Getenv("POSTGRES_DB")
	DB_PORT := os.Getenv("POSTGRES_PORT")

	// 환경 변수가 설정되지 않은 경우 기본값을 설정한다.
	if DB_USER == "" {
		DB_USER = "postgres"
	}
	if DB_PASSWORD == "" {
		DB_PASSWORD = "password"
	}
	if DB_NAME == "" {
		DB_NAME = "newsletter"
	}
	if DB_PORT == "" {
		DB_PORT = "5432"
	}

	// 도커 설정을 건너뛰지 않는 경우 PostgreSQL 컨테이너를 실행한다.
	if !SKIP_DOKER {
		if err := runDocker(DB_USER, DB_PASSWORD, DB_NAME, DB_PORT); err != nil {
			os.Stderr.WriteString("Failed to start PostgreSQL container: " + err.Error() + "\n")
			os.Exit(1)
		}
	}

	if err := os.Setenv("PGPASSWORD", DB_PASSWORD); err != nil {
		os.Stderr.WriteString("Failed to set PGPASSWORD: " + err.Error() + "\n")
		os.Exit(1)
	}

	// 데이터 베이스가 시작했는지 확인한다.
	for {
		if err := runCommand("psql", "-h", "localhost", "-p", DB_PORT, "-U", DB_USER); err == nil {
			break
		}
		// 잠시 대기 후 재시도
		time.Sleep(100 * time.Millisecond)
		os.Stderr.WriteString("Waiting for PostgreSQL to start...\n")
	}

	fmt.Println("PostgreSQL is ready on port", DB_PORT)

	// DATABASE_URL 환경 변수를 설정한다.
	DATABASE_URL := "postgres://" + DB_USER + ":" + DB_PASSWORD + "@localhost:" + DB_PORT + "/" + DB_NAME
	if err := os.Setenv("DATABASE_URL", DATABASE_URL); err != nil {
		os.Stderr.WriteString("Failed to set DATABASE_URL: " + err.Error() + "\n")
		os.Exit(1)
	}

	// 데이터베이스 마이그레이션을 실행한다.
	if err := runCommand("sea-orm-cli", "migrate", "up", "-d", "./migration"); err != nil {
		os.Stderr.WriteString("Failed to run migrations: " + err.Error() + "\n")
		os.Exit(1)
	}

	// 엔티티를 생성한다.
	// 엔티티는 ./src/entities 디렉토리에 생성된다.
	if err := runCommand("sea-orm-cli", "generate", "entity", "-o", "./src/entities"); err != nil {
		os.Stderr.WriteString("Failed to generate entities: " + err.Error() + "\n")
		os.Exit(1)
	}

	fmt.Println("Database migrations completed successfully.")
}
