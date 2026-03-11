package ui

import (
	"context"
	"net/url"
	"strings"
	"sync"
	"time"

	"ikuai-bypass/pkg/service"

	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/data/binding"
	"fyne.io/fyne/v2/dialog"
	"fyne.io/fyne/v2/widget"
)

type moduleOption struct {
	Label  string
	Module string
}

var moduleOptions = []moduleOption{
	{Label: "运营商/域名分流", Module: "ispdomain"},
	{Label: "IPv4 分组/端口分流", Module: "ipgroup"},
	{Label: "IPv6 分组", Module: "ipv6group"},
	{Label: "混合(ispdomain+ipgroup)", Module: "ii"},
	{Label: "混合(v4+v6分组)", Module: "ip"},
	{Label: "全能(ispdomain+v4+v6)", Module: "iip"},
}

func BuildMainWindow(app fyne.App, runtime *service.RuntimeService, webURL string, cronExpr string) fyne.Window {
	win := app.NewWindow("iKuai Bypass GUI")

	statusBinding := binding.NewString()
	statusLabel := widget.NewLabelWithData(statusBinding)

	cronLabel := widget.NewLabel("")
	if cronExpr == "" {
		cronLabel.SetText("Cron 表达式：未配置")
	} else {
		cronLabel.SetText("Cron 表达式：" + cronExpr)
	}

	logBinding := binding.NewString()
	logEntry := widget.NewEntryWithData(logBinding)
	logEntry.MultiLine = true
	logEntry.Wrapping = fyne.TextWrapWord
	logEntry.Disable()

	logMutex := &sync.Mutex{}
	logLines := make([]string, 0, 1000)
	appendLog := func(line string) {
		logMutex.Lock()
		defer logMutex.Unlock()
		if line != "" {
			logLines = append(logLines, line)
		}
		if len(logLines) > 2000 {
			logLines = logLines[len(logLines)-2000:]
		}
		_ = logBinding.Set(strings.Join(logLines, "\n"))
	}

	for _, entry := range runtime.TailLogs(200) {
		prefix := ""
		if entry.Time != (time.Time{}) {
			prefix = "[" + entry.Time.Format(time.RFC3339) + "] "
		}
		appendLog(prefix + entry.Line)
	}

	ctx, cancel := context.WithCancel(context.Background())
	win.SetOnClosed(func() {
		cancel()
	})
	go func() {
		ch := runtime.SubscribeLogs(ctx, 200)
		for entry := range ch {
			prefix := ""
			if entry.Time != (time.Time{}) {
				prefix = "[" + entry.Time.Format(time.RFC3339) + "] "
			}
			appendLog(prefix + entry.Line)
		}
	}()

	refreshStatus := func() {
		status := runtime.Status()
		text := "状态：空闲"
		if status.Running {
			text = "状态：运行中"
		}
		if status.Module != "" {
			text += " | 模块：" + status.Module
		}
		if status.CronRunning {
			text += " | 定时：已启动"
		}
		_ = statusBinding.Set(text)
	}
	refreshStatus()

	go func() {
		ticker := time.NewTicker(2 * time.Second)
		defer ticker.Stop()
		for {
			select {
			case <-ctx.Done():
				return
			case <-ticker.C:
				refreshStatus()
			}
		}
	}()

	startButtons := make([]fyne.CanvasObject, 0, len(moduleOptions))
	for _, opt := range moduleOptions {
		module := opt.Module
		label := "启动 " + opt.Label
		btn := widget.NewButton(label, func() {
			started, err := runtime.StartRunOnce(module)
			if err != nil {
				dialog.ShowError(err, win)
				return
			}
			if !started {
				dialog.ShowInformation("提示", "任务已在运行中", win)
				return
			}
			refreshStatus()
		})
		startButtons = append(startButtons, btn)
	}

	cronSelect := widget.NewSelect([]string{
		moduleOptions[0].Label,
		moduleOptions[1].Label,
		moduleOptions[2].Label,
		moduleOptions[3].Label,
		moduleOptions[4].Label,
		moduleOptions[5].Label,
	}, nil)
	cronSelect.SetSelected(moduleOptions[0].Label)
	runOnceBeforeCron := widget.NewCheck("启动前先执行一次", nil)

	resolveCronModule := func() string {
		for _, opt := range moduleOptions {
			if opt.Label == cronSelect.Selected {
				return opt.Module
			}
		}
		return moduleOptions[0].Module
	}

	startCronBtn := widget.NewButton("启动定时任务", func() {
		if cronExpr == "" {
			dialog.ShowInformation("提示", "配置文件未设置 Cron 表达式", win)
			return
		}
		if runOnceBeforeCron.Checked {
			_, _ = runtime.StartRunOnce(resolveCronModule())
		}
		if err := runtime.StartCron(cronExpr, resolveCronModule()); err != nil {
			dialog.ShowError(err, win)
			return
		}
		refreshStatus()
	})

	stopCronBtn := widget.NewButton("停止定时任务", func() {
		if err := runtime.StopCron(); err != nil {
			dialog.ShowError(err, win)
			return
		}
		refreshStatus()
	})

	openWebBtn := widget.NewButton("打开 WebUI", func() {
		if webURL == "" {
			dialog.ShowInformation("提示", "WebUI 未启动", win)
			return
		}
		parsed, err := url.Parse(webURL)
		if err != nil {
			dialog.ShowError(err, win)
			return
		}
		_ = app.OpenURL(parsed)
	})

	left := container.NewVBox(
		widget.NewLabelWithStyle("运行控制", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		container.NewGridWithColumns(2, startButtons...),
		widget.NewSeparator(),
		widget.NewLabelWithStyle("定时任务", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		cronLabel,
		widget.NewLabel("定时模块"),
		cronSelect,
		runOnceBeforeCron,
		container.NewGridWithColumns(2, startCronBtn, stopCronBtn),
		widget.NewSeparator(),
		openWebBtn,
		widget.NewSeparator(),
		widget.NewLabelWithStyle("日志", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		container.NewScroll(logEntry),
	)

	statusBar := container.NewBorder(nil, nil, nil, nil, statusLabel)
	content := container.NewBorder(statusBar, nil, nil, nil, left)
	win.SetContent(content)
	return win
}
