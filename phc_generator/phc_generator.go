package main

import (
	"crypto/rand"
	"encoding/base64"
	"flag"
	"fmt"
	"log"
	"os"

	"golang.org/x/crypto/argon2"
)

type Params struct {
	m,
	t,
	p,
	output_len uint
	salt,
	password string
}

func initParams() Params {
	params := Params{}
	flag.UintVar(&params.m, "m", 19456, "[m]emory")
	flag.UintVar(&params.t, "t", 2, "i[t]erations")
	flag.UintVar(&params.p, "p", 1, "[p]arallelism")
	flag.UintVar(&params.output_len, "l", 32, "output [l]ength")
	flag.StringVar(&params.password, "password", "", "[password] (Optional)")
	flag.StringVar(&params.salt, "salt", "", "[salt] (Optional)")
	flag.Parse()

	if params.salt == "" {
		salt, err := generatePassword(16)
		if err != nil {
			log.Fatal(err)
		}
		params.salt = salt
	}

	if params.password == "" {
		password, err := generatePassword(32)
		if err != nil {
			log.Fatal(err)
		}
		params.password = password
	}

	return params
}

func (params *Params) GeneratePHC() string {
	passwordHash := argon2.IDKey(
		[]byte(params.password),
		[]byte(params.salt),
		uint32(params.t),
		uint32(params.m),
		uint8(params.p),
		uint32(params.output_len),
	)
	passwordHashBase64 := base64.RawStdEncoding.EncodeToString(passwordHash)
	saltBase64 := base64.RawStdEncoding.EncodeToString([]byte(params.salt))

	return fmt.Sprintf("$argon2id$v=19$m=%d,t=%d,p=%d$%s$%s",
		params.m, params.t, params.p, saltBase64, passwordHashBase64,
	)
}

func generatePassword(len int) (string, error) {
	passwordBtye := make([]byte, len)
	if _, err := rand.Read(passwordBtye); err != nil {
		return "", err
	}
	return base64.RawStdEncoding.EncodeToString(passwordBtye), nil
}

func init() {
	flag.CommandLine.Usage = func() {
		fmt.Fprintf(flag.CommandLine.Output(), "%s\n\n", os.Args[0])
		fmt.Fprintf(flag.CommandLine.Output(), "Generates a PHC string.\n\n")
		fmt.Fprintf(flag.CommandLine.Output(), "Usage: \n")
		flag.PrintDefaults()
	}
}

func main() {
	params := initParams()

	fmt.Println("password   :", params.password)
	fmt.Println("salt       :", params.salt)
	fmt.Println("PHC string :", params.GeneratePHC())
}
