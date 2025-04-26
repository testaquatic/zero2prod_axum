package main

import (
	"flag"
	"fmt"
	"log"
	"os"
	"os/exec"

	"github.com/testaquatic/zero2prod_axum/scripts/zcmd"
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
	var dbPort uint16
	dbPortString := os.Getenv("POSTGRES_PORT")
	if dbPortString == "" {
		dbPort = 5432
	} else {
		_, err := fmt.Sscanf(dbPortString, "%d", &dbPort)
		if err != nil {
			log.Fatal(err)
		}
	}

	postgres := zcmd.Postgres{
		User: dbUser,
		Pass: dbPass,
		Path: "localhost",
		Name: dbName,
		Port: dbPort,
	}

	log.Println("dbUser:", postgres.User)
	// log.Println("dbPass:", dbPass)
	log.Println("dbName:", postgres.Name)
	log.Println("dbPort:", postgres.Port)

	// 도커를 실행한다.
	var cmd *exec.Cmd
	if !skipDocker {
		cmd = zcmd.MakeCmd(
			"docker", "run",
			"-e", "POSTGRES_USER="+postgres.User,
			"-e", "POSTGRES_PASSWORD="+postgres.Pass,
			"-e", "POSTGRES_DB="+postgres.Name,
			"-p", fmt.Sprintf("%d:5432", postgres.Port),
			"-d", "postgres",
			"postgres", "-N", "1000",
		)
		err := zcmd.RunCmd(false, cmd)
		if err != nil {
			log.Fatal(err)
		}
	}

	// `Postgres`가 초기화되기를 기다린다.
	postgres.Wait()
	log.Println("Running migrations now!")

	// Postgres를 준비한다.
	err := postgres.Prepare()
	if err != nil {
		log.Fatal(err)
	}
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

// Postgres를 초기화한다.
