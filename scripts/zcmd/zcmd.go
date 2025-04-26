package zcmd

import (
	"fmt"
	"log"
	"os"
	"os/exec"
	"time"
)

type Postgres struct {
	User string
	Pass string
	Name string
	Path string
	Port uint16
}

func (p *Postgres) DatabaseURL() string {
	return fmt.Sprintf("postgres://%s:%s@%s:%d/%s", p.User, p.Pass, p.Path, p.Port, p.Name)
}

func (p *Postgres) Prepare() error {
	dataBaseURL := p.DatabaseURL()

	// sqlx 데이터 베이스를 생성한다.
	cmd := MakeCmd("sqlx", "database", "create")
	cmd.Env = append(cmd.Env, dataBaseURL)
	err := RunCmd(true, cmd)
	if err != nil {
		return err
	}

	// 데이터베이스 마이그레이션을 한다.
	cmd = MakeCmd("sqlx", "migrate", "run")
	cmd.Env = append(cmd.Env, dataBaseURL)
	err = RunCmd(true, cmd)
	if err != nil {
		return err
	}
	log.Println("Postgres has been migrated, ready to go!")

	return nil
}

func (p *Postgres) Wait() {
	for {
		cmd := MakeCmd("psql", "-h", "localhost", "-U", p.User, "-p", fmt.Sprint(p.Port), "-d", "postgres", "-c", `\q`)
		cmd.Env = append(cmd.Env, fmt.Sprint("PGPASSWORD=", p.Pass))
		err := RunCmd(true, cmd)
		if err == nil {
			log.Println("Postgres is up and running on port", p.Port)
			break
		}
		log.Println("Postgres is still unavailable - sleeping")
		time.Sleep(time.Second)
	}
}

// `*exe.Cmd`를 생성한다.
func MakeCmd(args ...string) *exec.Cmd {
	cmd := exec.Command(args[0], args[1:]...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd
}

// `*exec.Cmd`를 실행한다.
func RunCmd(isPrint bool, cmd *exec.Cmd) error {
	if isPrint {
		log.Println(cmd.String())
	}
	return cmd.Run()
}
