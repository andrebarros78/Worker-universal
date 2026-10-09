package supervisor

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"net/http/httptest"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

func TestF11ConcurrencyMatrix(t *testing.T) {
	for _, workers := range []int{1, 4, 8, 16} {
		t.Run(string(rune('A'+workers)), func(t *testing.T) {
			p, err := NewPool(PoolConfig{Workers: workers, QueueCapacity: 200,
				WatchdogTimeout: time.Second * 5, RestartPolicy: Policy{MaxRestarts: 1}})
			if err != nil {
				t.Fatal(err)
			}
			p.Start()
			const total = 128
			var count atomic.Int32
			var current atomic.Int32
			var peak atomic.Int32
			for i := 0; i < total; i++ {
				name := i
				err := p.Submit(Task{ID: fmt.Sprintf("f11-job-%03d", name), Run: func(ctx context.Context) error {
					c := current.Add(1)
					for {
						old := peak.Load()
						if c <= old || peak.CompareAndSwap(old, c) {
							break
						}
					}
					time.Sleep(time.Millisecond)
					count.Add(1)
					current.Add(-1)
					return nil
				}})
				if err != nil {
					t.Fatal(err)
				}
			}
			seen := 0
			timer := time.NewTimer(20 * time.Second)
			defer timer.Stop()
			for seen < total {
				select {
				case result := <-p.Results():
					if result.Err != nil {
						t.Fatal(result.Err)
					}
					seen++
				case <-timer.C:
					t.Fatal("deadlock in worker pool")
				}
			}
			p.Stop()
			if count.Load() != total {
				t.Fatal("lost task")
			}
			if int(peak.Load()) > workers {
				t.Fatal("concurrency limit exceeded")
			}
			if h := p.Health(); h.Completed != total || h.Failed != 0 {
				t.Fatalf("bad counters: %+v", h)
			}
		})
	}
}
func TestF11ResourceLockSerializesSameKey(t *testing.T) {
	p, _ := NewPool(PoolConfig{Workers: 16, QueueCapacity: 64, WatchdogTimeout: time.Second * 2})
	p.Start()
	var inKey atomic.Int32
	var overlaps atomic.Int32
	for i := 0; i < 32; i++ {
		if err := p.Submit(Task{ID: fmt.Sprintf("locked-%03d", i), Resource: "single-browser",
			Run: func(context.Context) error {
				if inKey.Add(1) != 1 {
					overlaps.Add(1)
				}
				time.Sleep(time.Millisecond)
				inKey.Add(-1)
				return nil
			}}); err != nil {
			t.Fatal(err)
		}
	}
	for i := 0; i < 32; i++ {
		<-p.Results()
	}
	p.Stop()
	if overlaps.Load() != 0 {
		t.Fatalf("resource collision %d", overlaps.Load())
	}
}
func TestF11BackpressureAndHealthEndpoints(t *testing.T) {
	p, _ := NewPool(PoolConfig{Workers: 1, QueueCapacity: 1, WatchdogTimeout: time.Millisecond * 10})
	if p.Submit(Task{ID: "x", Run: func(context.Context) error { return nil }}) != ErrUnavailable {
		t.Fatal("must fail before starting")
	}
	p.Start()
	release := make(chan struct{})
	if err := p.Submit(Task{ID: "first", Run: func(ctx context.Context) error {
		select {
		case <-release:
			return nil
		case <-ctx.Done():
			return ctx.Err()
		}
	}}); err != nil {
		t.Fatal(err)
	}
	// The first may or may not yet be in flight. Fill until capacity is reached.
	for i := 0; i < 2; i++ {
		e := p.Submit(Task{ID: "extra", Run: func(context.Context) error { return nil }})
		if e == ErrBackpressure {
			break
		}
	}
	if h := p.Health(); h.Status != "healthy" {
		t.Fatalf("unexpected health: %+v", h)
	}
	w := httptest.NewRecorder()
	p.Handler().ServeHTTP(w, httptest.NewRequest(http.MethodGet, "/healthz", nil))
	if w.Code != 200 {
		t.Fatal("health endpoint down")
	}
	close(release)
	p.Stop()
	w = httptest.NewRecorder()
	p.Handler().ServeHTTP(w, httptest.NewRequest(http.MethodGet, "/readyz", nil))
	if w.Code != 503 {
		t.Fatal("readiness must fail after stop")
	}
}
func TestF11WatchdogStallAndCancellation(t *testing.T) {
	p, _ := NewPool(PoolConfig{Workers: 1, QueueCapacity: 2, WatchdogTimeout: time.Millisecond * 5})
	p.Start()
	entered := make(chan struct{})
	if err := p.Submit(Task{ID: "stall", Run: func(ctx context.Context) error {
		close(entered)
		<-ctx.Done()
		return ctx.Err()
	}}); err != nil {
		t.Fatal(err)
	}
	<-entered
	time.Sleep(20 * time.Millisecond)
	if got := p.Health().Status; got != "watchdog_stalled" {
		t.Fatalf("expected watchdog stall got %s", got)
	}
	p.Stop()
}
func TestF11BoundedRestart(t *testing.T) {
	p, _ := NewPool(PoolConfig{Workers: 1, QueueCapacity: 2, WatchdogTimeout: time.Second, RestartPolicy: Policy{MaxRestarts: 2}})
	p.Start()
	var attempts atomic.Int32
	err := p.Submit(Task{ID: "crash", Run: func(context.Context) error {
		if attempts.Add(1) < 3 {
			return errors.New("synthetic crash")
		}
		return nil
	}})
	if err != nil {
		t.Fatal(err)
	}
	result := <-p.Results()
	p.Stop()
	if result.Err != nil || result.Restarts != 2 || attempts.Load() != 3 {
		t.Fatalf("bad retry report %+v", result)
	}
}
func TestF11ConcurrentSubmissionsWithoutDataRace(t *testing.T) {
	p, _ := NewPool(PoolConfig{Workers: 16, QueueCapacity: 1024, WatchdogTimeout: time.Second})
	p.Start()
	var wg sync.WaitGroup
	var accepted atomic.Int32
	for i := 0; i < 16; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for j := 0; j < 32; j++ {
				if err := p.Submit(Task{ID: fmt.Sprintf("parallel-%d-%d", i, j), Run: func(context.Context) error { return nil }}); err == nil {
					accepted.Add(1)
				}
			}
		}()
	}
	wg.Wait()
	for i := int32(0); i < accepted.Load(); i++ {
		<-p.Results()
	}
	p.Stop()
	if accepted.Load() != 512 {
		t.Fatalf("lost submissions %d", accepted.Load())
	}
}

func TestF11DuplicateSubmissionRejected(t *testing.T) {
	p, _ := NewPool(PoolConfig{Workers: 1, QueueCapacity: 2, WatchdogTimeout: time.Second})
	p.Start()
	task := Task{ID: "idempotent-1", Run: func(context.Context) error { return nil }}
	if err := p.Submit(task); err != nil {
		t.Fatal(err)
	}
	if err := p.Submit(task); err != ErrDuplicateTask {
		t.Fatalf("expected duplicate rejection got %v", err)
	}
	<-p.Results()
	p.Stop()
	if p.Health().Completed != 1 {
		t.Fatal("duplicate side effect")
	}
}
