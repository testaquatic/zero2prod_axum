package main

import (
	"flag"
	"fmt"
	"log"
	"os"
	"os/exec"
	"time"
)

var skipDocker bool

func init() {
	flag.BoolFunc("skip-docker", "Skip Docker", func(_ string) error {
		skipDocker = true
		return nil
	})
	flag.BoolFunc("s", "Skip Docker(shorthand)", func(s string) error {
		skipDocker = true
		return nil
	})
}

func main() {
	flag.Parse()
	checkPrerequisite()

	// 커스텀 유저가 설정되었는지 확인한다.
	dbUser := os.Getenv("POSTGRES_USER")
	if dbUser == "" {
		dbUser = "postgres"
	}
	// 커스텀 비밀번호가 설정되었는지 확인한다.
	dbPass := os.Getenv("POSTGRES_PASSWORD")
	if dbPass == "" {
		dbPass = "password"
	}
	// 커스텀 데이터베이스명이 설정되었는지 확인한다.
	dbName := os.Getenv("POSTGRES_DB")
	if dbName == "" {
		dbName = "newsletter"
	}
	// 커스텀 포트가 설정되었는지 확인한다.
	dbPort := os.Getenv("POSTGRES_PORT")
	if dbPort == "" {
		dbPort = "5432"
	}

	log.Println("dbUser:", dbUser)
	// log.Println("dbPass:", dbPass)
	log.Println("dbName:", dbName)
	log.Println("dbPort:", dbPort)

	// 도커를 실행한다.
	var cmd *exec.Cmd
	if !skipDocker {
		cmd = makeCmd(
			"docker", "run",
			"-e", "POSTGRES_USER="+dbUser,
			"-e", "POSTGRES_PASSWORD="+dbPass,
			"-e", "POSTGRES_DB="+dbName,
			"-p", dbPort+":5432",
			"-d", "postgres",
			"postgres", "-N", "1000",
		)
		err := runCmd(false, cmd)
		if err != nil {
			log.Fatal(err)
		}
	}

	// `Postgres`가 초기화되기를 기다린다.
	waitPostgres(dbUser, dbPass, dbPort)
	log.Println("Running migrations now!")
	dataBaseURL := fmt.Sprintf("DATABASE_URL=postgres://%s:%s@localhost:%s/%s", dbUser, dbPass, dbPort, dbName)

	// sqlx 데이터 베이스를 생성한다.
	cmd = makeCmd("sqlx", "database", "create")
	cmd.Env = append(cmd.Env, dataBaseURL)
	err := runCmd(true, cmd)
	if err != nil {
		log.Fatal(err)
	}

	// 데이터베이스 마이그레이션을 한다.
	cmd = makeCmd("sqlx", "migrate", "run")
	cmd.Env = append(cmd.Env, dataBaseURL)
	err = runCmd(true, cmd)
	if err != nil {
		log.Fatal(err)
	}
	log.Println("Postgres has been migrated, ready to go!")
}

// 사전 조건을 점검한다.
func checkPrerequisite() {
	lookPaths := []string{"docker", "psql"}
	for _, lookPath := range lookPaths {
		_, err := exec.LookPath(lookPath)
		if err != nil {
			log.Fatalf("Error: %s lookPath is not installed", lookPath)
		}
	}
	_, err := exec.LookPath("sqlx")
	if err != nil {
		log.Fatal(
			`Error: sqlx is not installed
Use:
	cargo install sqlx-cli
to install it`,
		)
	}
}

// `*exe.Cmd`를 생성한다.
func makeCmd(args ...string) *exec.Cmd {
	cmd := exec.Command(args[0], args[1:]...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd
}

// `*exec.Cmd`를 실행한다.
func runCmd(isPrint bool, cmd *exec.Cmd) error {
	if isPrint {
		log.Println(cmd.String())
	}
	return cmd.Run()
}

// 사전조건을 체크한다.
func waitPostgres(dbUser, dbPass, dbPort string) {
	for {
		cmd := makeCmd("psql", "-h", "localhost", "-U", dbUser, "-p", dbPort, "-d", "postgres", "-c", `\q`)
		cmd.Env = append(cmd.Env, "PGPASSWORD="+dbPass)
		err := runCmd(true, cmd)
		if err == nil {
			log.Println("Postgres is up and running on port", dbPort)
			break
		}
		log.Println("Postgres is still unavailable - sleeping")
		time.Sleep(time.Second)
	}
}
