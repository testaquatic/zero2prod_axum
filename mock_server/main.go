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
	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		io.Copy(w, r.Body)
		r.Body.Close()
	})
	emailHandler := new(PMMockServerState)
	emailHandler.AddHander(http.MethodPost, emailHandler.PMMockServerPostHandler())
	mux.Handle("/email", emailHandler)
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
		if contentType, ok := r.Header["Content-Type"]; !ok || contentType[0] != "application/json" {
			log.Println(contentType)
			http.Error(w, "Invalid content type", http.StatusBadRequest)
			return
		}
		if accept, ok := r.Header["Accept"]; !ok || accept[0] != "application/json" {
			log.Println(accept)
			http.Error(w, "Invalid accept type", http.StatusBadRequest)
			return
		}
		var body PMRequestBody
		err := json.NewDecoder(r.Body).Decode(&body)
		if err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		_, err = io.Copy(io.Discard, r.Body)
		if err != nil {
			log.Println(err)
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}
		defer r.Body.Close()

		request := PMRequest{
			Header: r.Header,
			Body:   body,
			Method: r.Method,
		}
		messageID := uuid.New().String()
		log.Println("messageID:", messageID)
		log.Println("request:", request)

		if server.Requests == nil {
			server.Requests = make(map[string]PMRequest)
		}
		server.Requests[messageID] = request

		response := PMResponse{
			To:          body.To,
			SubmittedAT: time.Now().Format(time.RFC3339),
			MessageID:   messageID,
			ErrorCode:   0,
			Message:     "OK",
		}

		w.WriteHeader(http.StatusOK)
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
		command := new(Command)
		err := json.NewDecoder(r.Body).Decode(command)
		if err != nil {
			log.Println(err)
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		defer r.Body.Close()

		switch command.Command {
		case "get":
			request, ok := server.Requests[command.Uuid]
			if !ok {
				http.Error(w, "Not found", http.StatusNotFound)
				return
			}
			json.NewEncoder(w).Encode(request)
			return
		default:
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
	To          string
	SubmittedAT string
	MessageID   string
	ErrorCode   uint
	Message     string
}
