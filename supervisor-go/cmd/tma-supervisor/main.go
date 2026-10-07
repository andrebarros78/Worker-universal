package main

import (
	"context"
	"encoding/json"
	"fmt"
	"os"

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

func main() {
	command := "self-test"
	if len(os.Args) > 1 {
		command = os.Args[1]
	}
	switch command {
	case "self-test":
		os.Exit(selfTest())
	default:
		fmt.Fprintln(os.Stderr, "unknown command:", command)
		os.Exit(2)
	}
}
