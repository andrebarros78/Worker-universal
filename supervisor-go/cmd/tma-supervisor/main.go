package main

import (
	"context"
	"encoding/json"
	"fmt"
	"net"
	"net/http"
	"os"
	"os/signal"
	"strconv"
	"syscall"
	"time"

	"timedmissionagent/supervisor/supervisor"
)

func selfTest() int {
	calls := 0
	report := supervisor.RunWithRestart(
		context.Background(),
		supervisor.Policy{MaxRestarts: 2},
		func(context.Context) error {
			calls++
			if calls == 1 {
				return supervisor.ErrSynthetic
			}
			return nil
		},
	)
	payload := map[string]any{
		"component": "tma-supervisor",
		"attempts":  report.Attempts,
		"restarts":  report.Restarts,
		"status":    "ok",
	}
	if report.LastErr != nil {
		payload["status"] = "failed"
		payload["error"] = report.LastErr.Error()
	}
	encoded, _ := json.Marshal(payload)
	fmt.Println(string(encoded))
	if report.LastErr != nil {
		return 1
	}
	return 0
}

func serveHealth(port string) int {
	n, err := strconv.Atoi(port)
	if err != nil || n < 0 || n > 65535 {
		fmt.Fprintln(os.Stderr, "invalid port")
		return 2
	}
	pool, err := supervisor.NewPool(supervisor.PoolConfig{
		Workers: 4, QueueCapacity: 128, WatchdogTimeout: 30 * time.Second,
		RestartPolicy: supervisor.Policy{MaxRestarts: 2, Backoff: 100 * time.Millisecond},
	})
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		return 1
	}
	pool.Start()
	defer pool.Stop()
	listener, err := net.Listen("tcp", fmt.Sprintf("127.0.0.1:%d", n))
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		return 1
	}
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()
	server := &http.Server{Handler: pool.Handler(), ReadHeaderTimeout: 5 * time.Second}
	fmt.Printf("F11_HEALTH_LISTEN=http://%s\n", listener.Addr().String())
	finished := make(chan error, 1)
	go func() { finished <- server.Serve(listener) }()
	select {
	case <-ctx.Done():
		shutdown, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		_ = server.Shutdown(shutdown)
		return 0
	case err := <-finished:
		if err != nil && err != http.ErrServerClosed {
			fmt.Fprintln(os.Stderr, err)
			return 1
		}
		return 0
	}
}

func main() {
	command := "self-test"
	if len(os.Args) > 1 {
		command = os.Args[1]
	}
	switch command {
	case "self-test":
		os.Exit(selfTest())
	case "serve-health":
		if len(os.Args) != 3 {
			fmt.Fprintln(os.Stderr, "usage: serve-health <port>")
			os.Exit(2)
		}
		os.Exit(serveHealth(os.Args[2]))
	default:
		fmt.Fprintln(os.Stderr, "unknown command:", command)
		os.Exit(2)
	}
}
