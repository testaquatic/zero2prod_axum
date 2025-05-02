package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"log"
	"math"
	"net"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/google/uuid"
)

var Port uint

func init() {
	flag.UintVar(&Port, "port", 8800, "Port number")
	if Port > math.MaxUint16 {
		flag.PrintDefaults()
		os.Exit(1)
	}
}

func main() {
	flag.Parse()

	mux := http.NewServeMux()
	mux.HandleFunc("/health_check", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		io.Copy(w, r.Body)
		r.Body.Close()
	})
	emailHandler := new(PMMockServerState)
	emailHandler.AddHander(http.MethodPost, emailHandler.PMMockServerPostHandler())
	mux.Handle("/email", emailHandler)
	mux.HandleFunc("/500/email", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
		io.Copy(w, r.Body)
		r.Body.Close()
	})
	mux.Handle("/delay/email", &DelayHandler{next: emailHandler})
	mux.Handle("/debug", emailHandler.PMMockServerDebugHandler())

	listener, err := net.Listen("tcp", fmt.Sprintf("localhost:%d", Port))
	if err != nil {
		log.Fatal(err)
	}
	http.Serve(listener, mux)
}

type PMMockServerState struct {
	// 키는 UUID이다.
	Requests map[string]PMRequest
	// 키는 METHOD이다.
	MethodHandler map[string]http.Handler
}

func (server *PMMockServerState) AddHander(method string, handler http.Handler) {
	if server.MethodHandler == nil {
		server.MethodHandler = make(map[string]http.Handler)
	}
	server.MethodHandler[method] = handler
}

func (server *PMMockServerState) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	h, ok := server.MethodHandler[r.Method]
	if !ok {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	h.ServeHTTP(w, r)
}

func (server *PMMockServerState) PMMockServerPostHandler() http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		defer func() {
			io.Copy(io.Discard, r.Body)
			r.Body.Close()
		}()
		log.Println("PMMockServerPostHandler!")

		if contentType, ok := r.Header["Content-Type"]; !ok || contentType[0] != "application/json" {
			log.Println(contentType)
			http.Error(w, "Invalid content type", http.StatusBadRequest)
			return
		}
		/*
			if accept, ok := r.Header["Accept"]; !ok || accept[0] != "application/json" {
				log.Println(accept)
				http.Error(w, "Invalid accept type", http.StatusBadRequest)
				return
			}
		*/
		var body PMRequestBody
		err := json.NewDecoder(r.Body).Decode(&body)
		if err != nil {
			log.Println(err)
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}

		request := PMRequest{
			Header: r.Header,
			Body:   body,
			Method: r.Method,
		}
		log.Printf("PMRequest: %#v\n", request)

		messageID := uuid.New().String()

		if server.Requests == nil {
			server.Requests = make(map[string]PMRequest)
		}
		server.Requests[messageID] = request

		response := PMResponse{
			To:          body.To,
			SubmittedAt: time.Now().Format(time.RFC3339),
			MessageID:   messageID,
			ErrorCode:   0,
			Message:     "OK",
		}

		w.WriteHeader(http.StatusOK)
		log.Printf("PMResponse: %#v\n", response)
		err = json.NewEncoder(w).Encode(response)
		if err != nil {
			log.Println(err)
		}
	})
}

type Command struct {
	Command string `json:"command"`
	Uuid    string `json:"uuid,omitempty"`
}

func (server *PMMockServerState) PMMockServerDebugHandler() http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		defer func() {
			io.Copy(io.Discard, r.Body)
			r.Body.Close()
		}()
		log.Println("PMMockServerDebugHandler!")

		command := new(Command)
		err := json.NewDecoder(r.Body).Decode(command)
		if err != nil {
			log.Println(err)
			http.Error(w, err.Error(), http.StatusBadRequest)

			return
		}
		log.Printf("Command: %#v\n", command)

		switch command.Command {
		case "get":
			request, ok := server.Requests[strings.TrimSpace(command.Uuid)]
			if !ok {
				log.Printf("Invalid UUID: %#v\n", command.Uuid)
				http.Error(w, "Not found", http.StatusNotFound)

				return
			}
			log.Printf("PMRequest: %#v\n", request)
			json.NewEncoder(w).Encode(request)

			return
		default:
			log.Printf("Invalid command: %#v\n", command.Command)
			http.Error(w, "Invalid command", http.StatusBadRequest)

			return
		}
	},
	)
}

type PMRequest struct {
	Header http.Header   `json:"header"`
	Body   PMRequestBody `json:"body"`
	Method string        `json:"method"`
}

type PMRequestBody struct {
	From     string
	To       string
	Subject  string
	TextBody string
	HtmlBody string
}

type PMResponse struct {
	To          string `json:",omitempty"`
	SubmittedAt string `json:",omitempty"`
	MessageID   string `json:",omitempty"`
	ErrorCode   uint
	Message     string `json:",omitempty"`
}

type DelayHandler struct {
	next http.Handler
}

func (h *DelayHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	log.Println("DelayHandler!")
	time.Sleep(100 * time.Second)
	h.next.ServeHTTP(w, r)
}
