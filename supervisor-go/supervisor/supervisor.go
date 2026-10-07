package supervisor

import (
	"context"
	"errors"
	"time"
)

type Runner func(context.Context) error

type Policy struct {
	MaxRestarts int
	Backoff     time.Duration
}

type Report struct {
	Attempts int
	Restarts int
	LastErr  error
}

func RunWithRestart(ctx context.Context, policy Policy, runner Runner) Report {
	if policy.MaxRestarts < 0 {
		policy.MaxRestarts = 0
	}
	report := Report{}
	for {
		if err := ctx.Err(); err != nil {
			report.LastErr = err
			return report
		}
		report.Attempts++
		err := runner(ctx)
		if err == nil {
			report.LastErr = nil
			return report
		}
		report.LastErr = err
		if report.Restarts >= policy.MaxRestarts {
			return report
		}
		report.Restarts++
		if policy.Backoff > 0 {
			timer := time.NewTimer(policy.Backoff)
			select {
			case <-ctx.Done():
				if !timer.Stop() {
					<-timer.C
				}
				report.LastErr = ctx.Err()
				return report
			case <-timer.C:
			}
		}
	}
}

var ErrSynthetic = errors.New("synthetic worker failure")
