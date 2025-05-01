package main

import (
	"flag"
	"log"

	"github.com/testaquatic/zero2prod_axum/go/initializer/zcmd"
)

var (
	dbUser   string
	dbPass   string
	dbName   string
	dbPort   uint16
	postgres zcmd.Postgres
)

func init() {
	flag.StringVar(&dbUser, "user", "postgres", "Database user")
	flag.StringVar(&dbPass, "pass", "password", "Database password")
	flag.StringVar(&dbName, "name", "newsletter", "Database name")
	dbPort = uint16(*flag.Uint("port", 5432, "Database port"))
	postgres = zcmd.Postgres{
		User: dbUser,
		Pass: dbPass,
		Path: "localhost",
		Name: dbName,
		Port: dbPort,
	}
}

func main() {
	flag.Parse()

	log.Println("Staring containers.")
	cmd := zcmd.MakeCmd("docker", "compose", "up", "-d")
	err := zcmd.RunCmd(true, cmd)
	if err != nil {
		log.Fatal(err)
	}

	log.Println("Running migrations now!")
	postgres.Wait()
	err = postgres.Prepare()
	if err != nil {
		log.Fatal(err)
	}

	log.Println("Done!")
}
