package main

import (
	"context"
	"flag"
	"fmt"
	"io"
	"log"
	"math"
	"net"
	"net/http"
	"os"
	"os/signal"
	"sync"
	"time"
)

var Port uint
var TimeOut time.Duration

func init() {
	flag.UintVar(&Port, "port", 8800, "Port number")
	if Port > math.MaxUint16 {
		flag.PrintDefaults()
		os.Exit(1)
	}
	flag.DurationVar(&TimeOut, "timeout", 10*time.Second, "Timeout")
}

func main() {
	signalRecieved := make(chan struct{})
	go signalHandler(signalRecieved)

	flag.Parse()

	wg := new(sync.WaitGroup)
	HubServer(signalRecieved, wg)

	wg.Wait()
}

func HubServer(signalRecieved chan struct{}, wg *sync.WaitGroup) {
	log.Println("Starting HubServer")
	wg.Add(1)

	mux := http.NewServeMux()
	resetTimerChan := make(chan struct{})
	mux.HandleFunc("/health_check", func(w http.ResponseWriter, r *http.Request) {
		resetTimerChan <- struct{}{}
		w.WriteHeader(http.StatusOK)
		io.Copy(io.Discard, r.Body)
		r.Body.Close()
	})

	mux.HandleFunc("/new_server", func(w http.ResponseWriter, r *http.Request) {
		resetTimerChan <- struct{}{}
		defer func() {
			io.Copy(io.Discard, r.Body)
			r.Body.Close()
		}()
		log.Println("New Server!")
		port, err := StartPMMockServer(TimeOut, signalRecieved, wg)
		if err != nil {
			log.Fatal(err)
		}
		w.WriteHeader(http.StatusOK)
		io.WriteString(w, port)
	})

	listener, err := net.Listen("tcp", fmt.Sprintf("127.0.0.1:%d", Port))
	if err != nil {
		log.Fatal(err)
	}
	defer listener.Close()

	server := &http.Server{
		Handler:           mux,
		ReadHeaderTimeout: time.Second * 10,
		IdleTimeout:       time.Minute,
	}

	go ServerShutdown(server, signalRecieved, resetTimerChan, wg)

	err = server.Serve(listener)
	if err != nil && err != http.ErrServerClosed {
		log.Fatal(err)
	}

}

func signalHandler(signalRecieved chan struct{}) {
	sig := make(chan os.Signal, 1)
	signal.Notify(sig, os.Interrupt)
	<-sig

	close(signalRecieved)
}

func ServerShutdown(server *http.Server, signalRecieved, resetTimerChan <-chan struct{}, wg *sync.WaitGroup) {
	timer := time.NewTimer(TimeOut)

out:
	for {
		select {
		case <-timer.C:
			log.Println("Timeout")
			break out
		case <-resetTimerChan:
			log.Println("Resetting timeout")
			timer.Reset(TimeOut)
		case <-signalRecieved:
			log.Println("Signal received")
			break out
		}
	}

	log.Println("Shutting down server...")
	err := server.Shutdown(context.Background())
	if err != nil {
		log.Fatal(err)
	}
	log.Println("Server gracefully stopped")
	wg.Done()
}
