package main

import (
	"crypto/rand"
	"encoding/base64"
	"flag"
	"fmt"
	"log"

	"golang.org/x/crypto/argon2"
)

type Params struct {
	m,
	t,
	p uint
	salt,
	password string
}

var params = Params{}

func init() {
	flag.UintVar(&params.m, "m", 19456, "The Minimum memory size.")
	flag.UintVar(&params.t, "t", 2, "The minimum number of iterations.")
	flag.UintVar(&params.p, "p", 1, "The degree of parallelism")
	flag.StringVar(&params.password, "password", "", "Password")
	flag.StringVar(&params.salt, "salt", "", "Salt")
	flag.CommandLine.Usage = func() {
		fmt.Fprintf(flag.CommandLine.Output(), "Usage: \n")
		flag.PrintDefaults()
	}
}

func main() {
	flag.Parse()

	if params.salt == "" {
		salt, err := generatePassword()
		if err != nil {
			log.Fatal(err)
		}
		params.salt = salt
	}

	if params.password == "" {
		password, err := generatePassword()
		if err != nil {
			log.Fatal(err)
		}
		params.password = password
	}

	passwordHash := argon2.IDKey([]byte(params.password), []byte(params.salt), uint32(params.t), uint32(params.m), uint8(params.p), 32)
	passwordHashBase64 := base64.RawStdEncoding.EncodeToString(passwordHash)
	saltBase64 := base64.RawStdEncoding.EncodeToString([]byte(params.salt))

	fmt.Println("password:", params.password)
	fmt.Println("salt    :", params.salt)
	fmt.Printf("$argon2id$v=19$m=%d,t=%d,p=%d$%s$%s\n", params.m, params.t, params.p, saltBase64, passwordHashBase64)
}

func generatePassword() (string, error) {
	passwordBtye := make([]byte, 16)
	if _, err := rand.Read(passwordBtye); err != nil {
		return "", err
	}
	return base64.RawStdEncoding.EncodeToString(passwordBtye), nil
}
