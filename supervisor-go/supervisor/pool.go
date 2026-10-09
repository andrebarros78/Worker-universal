package supervisor

import (
	"context"
	"errors"
	"net/http"
	"strconv"
	"sync"
	"time"
)

var ErrBackpressure = errors.New("queue capacity exhausted")
var ErrUnavailable = errors.New("worker pool unavailable")
var ErrDuplicateTask = errors.New("duplicate task id")

type Task struct {
	ID       string
	Resource string
	Run      Runner
}
type Outcome struct {
	ID       string
	Attempts int
	Restarts int
	Err      error
}
type Health struct {
	Status            string `json:"status"`
	Ready             bool   `json:"ready"`
	Workers           int    `json:"workers"`
	QueueDepth        int    `json:"queue_depth"`
	Inflight          int    `json:"inflight"`
	Completed         int    `json:"completed"`
	Failed            int    `json:"failed"`
	Restarts          int    `json:"restarts"`
	LastProgressAgoMS int64  `json:"last_progress_ago_ms"`
}
type PoolConfig struct {
	Workers         int
	QueueCapacity   int
	WatchdogTimeout time.Duration
	RestartPolicy   Policy
}
type Pool struct {
	cfg          PoolConfig
	queue        chan Task
	results      chan Outcome
	ctx          context.Context
	cancel       context.CancelFunc
	mu           sync.Mutex
	active       bool
	inflight     int
	completed    int
	failed       int
	restarts     int
	lastProgress time.Time
	locks        map[string]*sync.Mutex
	acceptedIDs  map[string]struct{}
	wg           sync.WaitGroup
}

func NewPool(cfg PoolConfig) (*Pool, error) {
	if cfg.Workers < 1 || cfg.Workers > 16 || cfg.QueueCapacity < 1 || cfg.RestartPolicy.MaxRestarts < 0 || cfg.WatchdogTimeout <= 0 {
		return nil, errors.New("invalid F11 pool config")
	}
	ctx, cancel := context.WithCancel(context.Background())
	return &Pool{cfg: cfg, queue: make(chan Task, cfg.QueueCapacity),
		results: make(chan Outcome, cfg.QueueCapacity+cfg.Workers),
		ctx:     ctx, cancel: cancel, locks: make(map[string]*sync.Mutex), acceptedIDs: make(map[string]struct{}), lastProgress: time.Now()}, nil
}
func (p *Pool) Start() {
	p.mu.Lock()
	defer p.mu.Unlock()
	if p.active {
		return
	}
	p.active = true
	for i := 0; i < p.cfg.Workers; i++ {
		p.wg.Add(1)
		go p.worker()
	}
}
func (p *Pool) Submit(task Task) error {
	if task.ID == "" || task.Run == nil {
		return errors.New("invalid task")
	}
	p.mu.Lock()
	defer p.mu.Unlock()
	if !p.active || p.ctx.Err() != nil {
		return ErrUnavailable
	}
	if _, exists := p.acceptedIDs[task.ID]; exists {
		return ErrDuplicateTask
	}
	select {
	case p.queue <- task:
		p.acceptedIDs[task.ID] = struct{}{}
		return nil
	default:
		return ErrBackpressure
	}
}
func (p *Pool) worker() {
	defer p.wg.Done()
	for {
		select {
		case <-p.ctx.Done():
			return
		case task := <-p.queue:
			p.mu.Lock()
			p.inflight++
			p.mu.Unlock()
			var lock *sync.Mutex
			if task.Resource != "" {
				p.mu.Lock()
				lock = p.locks[task.Resource]
				if lock == nil {
					lock = &sync.Mutex{}
					p.locks[task.Resource] = lock
				}
				p.mu.Unlock()
				lock.Lock()
			}
			report := RunWithRestart(p.ctx, p.cfg.RestartPolicy, task.Run)
			if lock != nil {
				lock.Unlock()
			}
			p.mu.Lock()
			p.inflight--
			p.completed++
			p.restarts += report.Restarts
			if report.LastErr != nil {
				p.failed++
			}
			p.lastProgress = time.Now()
			p.mu.Unlock()
			result := Outcome{ID: task.ID, Attempts: report.Attempts, Restarts: report.Restarts, Err: report.LastErr}
			select {
			case p.results <- result:
			case <-p.ctx.Done():
			}
		}
	}
}
func (p *Pool) Results() <-chan Outcome { return p.results }
func (p *Pool) Health() Health {
	p.mu.Lock()
	defer p.mu.Unlock()
	ago := time.Since(p.lastProgress)
	status := "healthy"
	ready := p.active && p.ctx.Err() == nil
	if !ready {
		status = "unavailable"
	} else if p.inflight > 0 && ago > p.cfg.WatchdogTimeout {
		status = "watchdog_stalled"
	}
	return Health{Status: status, Ready: ready && status == "healthy",
		Workers: p.cfg.Workers, QueueDepth: len(p.queue),
		Inflight: p.inflight, Completed: p.completed, Failed: p.failed,
		Restarts: p.restarts, LastProgressAgoMS: ago.Milliseconds()}
}
func (p *Pool) Handler() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		h := p.Health()
		if h.Status != "healthy" {
			w.WriteHeader(http.StatusServiceUnavailable)
		}
		w.Header().Set("Content-Type", "text/plain")
		_, _ = w.Write([]byte(h.Status))
	})
	mux.HandleFunc("/readyz", func(w http.ResponseWriter, r *http.Request) {
		h := p.Health()
		if !h.Ready {
			w.WriteHeader(http.StatusServiceUnavailable)
		}
		_, _ = w.Write([]byte(strconv.FormatBool(h.Ready)))
	})
	return mux
}
func (p *Pool) Stop() {
	p.cancel()
	p.wg.Wait()
	p.mu.Lock()
	p.active = false
	p.mu.Unlock()
}
