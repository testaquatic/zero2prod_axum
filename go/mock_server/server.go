package main

import (
	"encoding/json"
	"io"
	"log"
	"net"
	"net/http"
	"strings"
	"sync"
	"time"

	"github.com/google/uuid"
)

type PMMockServerState struct {
	// 키는 UUID이다.
	Requests []Request
	// 키는 METHOD이다.
	MethodHandler map[string]http.Handler
}

func (state *PMMockServerState) FindRequest(uuid string) ([]Request, bool) {
	requests := []Request{}
	for _, request := range state.Requests {
		if request.Uuid == uuid {
			requests = append(requests, request)
		}
	}
	return requests, len(requests) > 0
}

type Request struct {
	Uuid     string    `json:"uuid,omitempty"`
	Requests PMRequest `json:"requests,omitempty"`
}

// PMMockServer를 시작한다.
// 실질적인 서버의 역할을 한다.
func StartPMMockServer(timeOut time.Duration, signalRecieved chan struct{}, wg *sync.WaitGroup) (string, error) {
	log.Println("Starting PMMockServer")
	wg.Add(1)
	mux := http.NewServeMux()

	emailHandler := new(PMMockServerState)
	emailHandler.AddHander(http.MethodPost, emailHandler.PMMockServerPostHandler())
	mux.Handle("/email", emailHandler)
	mux.HandleFunc("/500/email", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
		io.Copy(w, r.Body)
		r.Body.Close()
	})
	mux.Handle("/delay/email", &DelayMiddleware{next: emailHandler})
	mux.Handle("/debug", emailHandler.PMMockServerDebugHandler())

	listener, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		return "", err
	}

	server := &http.Server{
		Handler:           mux,
		ReadHeaderTimeout: time.Second * 10,
		IdleTimeout:       time.Minute,
	}

	go func(listener net.Listener) {
		defer listener.Close()
		err = server.Serve(listener)
		if err != nil && err != http.ErrServerClosed {
			log.Fatal(err)
		}
	}(listener)

	go ServerShutdown(server, signalRecieved, make(<-chan struct{}), wg)

	_, port, err := net.SplitHostPort(listener.Addr().String())

	return port, err
}

func (pmMockServer *PMMockServerState) AddHander(method string, handler http.Handler) {
	if pmMockServer.MethodHandler == nil {
		pmMockServer.MethodHandler = make(map[string]http.Handler)
	}
	pmMockServer.MethodHandler[method] = handler
}

func (pmMockServer *PMMockServerState) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	h, ok := pmMockServer.MethodHandler[r.Method]
	if !ok {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	h.ServeHTTP(w, r)
}

func (pmMockServer *PMMockServerState) PMMockServerPostHandler() http.Handler {
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

		pmMockServer.Requests = append(pmMockServer.Requests, Request{
			Uuid:     messageID,
			Requests: request,
		})

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

// /debug의 처리를 담당한다.
func (pmMockServer *PMMockServerState) PMMockServerDebugHandler() http.Handler {
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
			request, ok := pmMockServer.FindRequest(strings.TrimSpace(command.Uuid))
			if !ok {
				log.Printf("Invalid UUID: %#v\n", command.Uuid)
				http.Error(w, "Not found", http.StatusNotFound)

				return
			}
			log.Printf("PMRequest: %#v\n", request)
			json.NewEncoder(w).Encode(request)

			return
		case "get_all":
			json.NewEncoder(w).Encode(pmMockServer.Requests)

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

type DelayMiddleware struct {
	next http.Handler
}

func (h *DelayMiddleware) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	log.Println("DelayHandler!")
	time.Sleep(100 * time.Second)
	h.next.ServeHTTP(w, r)
}
