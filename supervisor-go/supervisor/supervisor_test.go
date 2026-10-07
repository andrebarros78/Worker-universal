package supervisor

import (
	"context"
	"testing"
	"time"
)

func TestRecoversWithinRestartBudget(t *testing.T) {
	calls := 0
	report := RunWithRestart(
		context.Background(),
		Policy{MaxRestarts: 3},
		func(context.Context) error {
			calls++
			if calls < 3 {
				return ErrSynthetic
			}
			return nil
		},
	)
	if report.LastErr != nil {
		t.Fatalf("unexpected final error: %v", report.LastErr)
	}
	if report.Attempts != 3 || report.Restarts != 2 {
		t.Fatalf("unexpected report: %+v", report)
	}
}

func TestStopsAfterBudget(t *testing.T) {
	report := RunWithRestart(
		context.Background(),
		Policy{MaxRestarts: 1},
		func(context.Context) error { return ErrSynthetic },
	)
	if report.LastErr == nil {
		t.Fatal("expected error")
	}
	if report.Attempts != 2 || report.Restarts != 1 {
		t.Fatalf("unexpected report: %+v", report)
	}
}

func TestRespectsCancellationDuringBackoff(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	report := RunWithRestart(
		ctx,
		Policy{MaxRestarts: 10, Backoff: 50 * time.Millisecond},
		func(context.Context) error { return ErrSynthetic },
	)
	if report.LastErr == nil {
		t.Fatal("expected cancellation")
	}
	if report.Attempts != 0 {
		t.Fatalf("runner should not start after cancellation: %+v", report)
	}
}
