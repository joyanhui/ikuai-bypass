package webui

import (
	"context"

	"ikuai-bypass/pkg/service"
)

type RuntimeController interface {
	Status() service.RuntimeStatus
	StartRunOnce(module string) (bool, error)
	StartCron(expr string, module string) error
	StopCron() error
	TailLogs(n int) []service.LogEntry
	SubscribeLogs(ctx context.Context, buffer int) <-chan service.LogEntry
	SetDefaults(module string, cronExpr string)
}

var runtimeController RuntimeController

func SetRuntimeController(c RuntimeController) {
	runtimeController = c
}
