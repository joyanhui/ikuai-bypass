package service

import (
	"context"
	"errors"
	"sync"
	"time"

	"ikuai-bypass/pkg/core"

	"github.com/robfig/cron/v3"
)

type RuntimeStatus struct {
	Running     bool   `json:"running"`
	CronRunning bool   `json:"cron_running"`
	CronExpr    string `json:"cron_expr"`
	Module      string `json:"module"`
	LastRunAt   string `json:"last_run_at"`
	NextRunAt   string `json:"next_run_at"`
}

// RuntimeService 提供 run-once/cron/logs 控制 / Provides run-once/cron/logs control.
type RuntimeService struct {
	mu sync.Mutex

	module   string
	cronExpr string

	cron      *cron.Cron
	cronJobID cron.EntryID

	runSem chan struct{}

	lastRun time.Time
	logs    *LogBroker
}

func NewRuntimeService(defaultModule string, defaultCron string, logs *LogBroker) *RuntimeService {
	if defaultModule == "" {
		defaultModule = "ispdomain"
	}
	if logs == nil {
		logs = NewLogBroker(2000)
	}
	s := &RuntimeService{
		module:   defaultModule,
		cronExpr: defaultCron,
		runSem:   make(chan struct{}, 1),
		logs:     logs,
	}
	return s
}

func (s *RuntimeService) LogBroker() *LogBroker {
	return s.logs
}

func (s *RuntimeService) TailLogs(n int) []LogEntry {
	return s.logs.Tail(n)
}

func (s *RuntimeService) SubscribeLogs(ctx context.Context, buffer int) <-chan LogEntry {
	return s.logs.Subscribe(ctx, buffer)
}

func (s *RuntimeService) SetDefaults(module string, cronExpr string) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if module != "" {
		s.module = module
	}
	if cronExpr != "" {
		s.cronExpr = cronExpr
	}
}

func (s *RuntimeService) Status() RuntimeStatus {
	s.mu.Lock()
	defer s.mu.Unlock()

	st := RuntimeStatus{
		Running:     len(s.runSem) > 0,
		CronRunning: s.cron != nil,
		CronExpr:    s.cronExpr,
		Module:      s.module,
	}
	if !s.lastRun.IsZero() {
		st.LastRunAt = s.lastRun.Format(time.RFC3339)
	}
	if s.cron != nil {
		entry := s.cron.Entry(s.cronJobID)
		if !entry.Next.IsZero() {
			st.NextRunAt = entry.Next.Format(time.RFC3339)
		}
	}
	return st
}

func (s *RuntimeService) StartRunOnce(module string) (bool, error) {
	if module == "" {
		s.mu.Lock()
		module = s.module
		s.mu.Unlock()
	}

	select {
	case s.runSem <- struct{}{}:
	default:
		return false, nil
	}

	go func(mod string) {
		defer func() {
			<-s.runSem
			s.mu.Lock()
			s.lastRun = time.Now()
			s.mu.Unlock()
		}()
		core.RunUpdateByModule(mod)
	}(module)

	return true, nil
}

func (s *RuntimeService) StartCron(expr string, module string) error {
	if expr == "" {
		expr = s.cronExpr
	}
	if expr == "" {
		return errors.New("Cron expression is empty in config file")
	}
	if s.cronExpr != "" && expr != s.cronExpr {
		return errors.New("Cron expression must match config file")
	}
	if module == "" {
		s.mu.Lock()
		module = s.module
		s.mu.Unlock()
	}

	s.mu.Lock()
	defer s.mu.Unlock()
	if s.cron != nil {
		return errors.New("cron is already running")
	}

	s.cronExpr = expr
	s.module = module

	c := cron.New()
	id, err := c.AddFunc(expr, func() {
		_, _ = s.StartRunOnce(module)
	})
	if err != nil {
		return err
	}
	s.cron = c
	s.cronJobID = id
	c.Start()
	return nil
}

func (s *RuntimeService) StopCron() error {
	s.mu.Lock()
	c := s.cron
	s.cron = nil
	s.cronJobID = 0
	s.mu.Unlock()
	if c == nil {
		return nil
	}
	ctx := c.Stop()
	<-ctx.Done()
	return nil
}
