package ui

import (
	"context"
	"fmt"
	"image/color"
	"net/url"
	"strings"
	"sync"
	"time"

	"ikuai-bypass/pkg/service"

	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/canvas"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/data/binding"
	"fyne.io/fyne/v2/dialog"
	"fyne.io/fyne/v2/layout"
	"fyne.io/fyne/v2/theme"
	"fyne.io/fyne/v2/widget"
)

// ModuleConfig 定义分流模块配置
type ModuleConfig struct {
	Label       string
	Module      string
	Description string
}

var modules = []ModuleConfig{
	{Label: "运营商/域名分流", Module: "ispdomain", Description: "ISP + Domain"},
	{Label: "IPv4 分组/端口分流", Module: "ipgroup", Description: "IPv4 + Port"},
	{Label: "IPv6 分组", Module: "ipv6group", Description: "IPv6 Group"},
	{Label: "混合模式(运营商+IPv4)", Module: "ii", Description: "ISP + IPv4"},
	{Label: "IP混合(v4+v6)", Module: "ip", Description: "IPv4 + IPv6"},
	{Label: "全能模式", Module: "iip", Description: "All In One"},
}

// RunModeConfig 定义运行模式配置
type RunModeConfig struct {
	Label       string
	Mode        string
	Description string
	NeedCron    bool
}

var runModes = []RunModeConfig{
	{Label: "只执行一次", Mode: "once", Description: "立即执行后退出", NeedCron: false},
	{Label: "计划任务(先执行)", Mode: "cron", Description: "先执行一次后进入定时", NeedCron: true},
	{Label: "计划任务(延迟)", Mode: "cronAft", Description: "等待定时后再执行", NeedCron: true},
	{Label: "清理模式", Mode: "clean", Description: "清理所有 IKB 规则", NeedCron: false},
}

var (
	brandBlue        = color.NRGBA{R: 45, G: 122, B: 235, A: 255}
	brandBlueSoft    = color.NRGBA{R: 84, G: 162, B: 245, A: 255}
	brandBlueDeep    = color.NRGBA{R: 31, G: 88, B: 182, A: 255}
	pageBlueTop      = color.NRGBA{R: 36, G: 108, B: 224, A: 255}
	pageBlueBottom   = color.NRGBA{R: 83, G: 184, B: 247, A: 255}
	panelWhite       = color.NRGBA{R: 255, G: 255, B: 255, A: 248}
	panelBorder      = color.NRGBA{R: 220, G: 231, B: 246, A: 255}
	panelText        = color.NRGBA{R: 43, G: 58, B: 84, A: 255}
	panelMuted       = color.NRGBA{R: 111, G: 131, B: 162, A: 255}
	panelMutedSoft   = color.NRGBA{R: 227, G: 235, B: 246, A: 255}
	startRingOuter   = color.NRGBA{R: 255, G: 255, B: 255, A: 255}
	startRingInner   = color.NRGBA{R: 245, G: 250, B: 255, A: 255}
	stopRingOuter    = color.NRGBA{R: 255, G: 236, B: 236, A: 255}
	stopRingInner    = color.NRGBA{R: 255, G: 248, B: 248, A: 255}
	criticalStopText = color.NRGBA{R: 196, G: 70, B: 70, A: 255}
)

const (
	preferenceModuleKey  = "gui_selected_module"
	preferenceRunModeKey = "gui_selected_run_mode"
)

// BuildMainWindow 构建主窗口
func BuildMainWindow(app fyne.App, runtime *service.RuntimeService, webURL string, cronExpr string) fyne.Window {
	win := app.NewWindow("iKuai Bypass")
	prefs := app.Preferences()

	// ========== 状态区域 ==========
	statusBinding := binding.NewString()
	cronBinding := binding.NewString()

	cronLabel := widget.NewLabelWithData(cronBinding)
	cronLabel.Wrapping = fyne.TextWrapWord

	// ========== 日志区域 ==========
	logBinding := binding.NewString()
	logLabel := widget.NewLabelWithData(logBinding)
	logLabel.Wrapping = fyne.TextWrapWord

	var logScroll *container.Scroll

	// 日志管理
	logMutex := &sync.Mutex{}
	logLines := make([]string, 0, 2000)
	appendLog := func(line string) {
		logMutex.Lock()
		defer logMutex.Unlock()
		if line != "" {
			logLines = append(logLines, line)
		}
		if len(logLines) > 2000 {
			logLines = logLines[len(logLines)-2000:]
		}
		text := strings.Join(logLines, "\n")
		fyne.Do(func() {
			_ = logBinding.Set(text)
			if logScroll != nil {
				go func() {
					time.Sleep(50 * time.Millisecond)
					fyne.Do(func() {
						logScroll.ScrollToBottom()
					})
				}()
			}
		})
	}

	// 加载历史日志
	for _, entry := range runtime.TailLogs(100) {
		prefix := ""
		if entry.Time != (time.Time{}) {
			prefix = "[" + entry.Time.Format("15:04:05") + "] "
		}
		appendLog(prefix + entry.Line)
	}

	// 日志订阅
	ctx, cancel := context.WithCancel(context.Background())
	win.SetOnClosed(func() {
		cancel()
	})
	go func() {
		ch := runtime.SubscribeLogs(ctx, 100)
		for entry := range ch {
			prefix := ""
			if entry.Time != (time.Time{}) {
				prefix = "[" + entry.Time.Format("15:04:05") + "] "
			}
			appendLog(prefix + entry.Line)
		}
	}()

	selectedModule := findModuleByValue(prefs.StringWithFallback(preferenceModuleKey, modules[0].Module))
	selectedRunMode := findRunModeByValue(prefs.StringWithFallback(preferenceRunModeKey, runModes[0].Mode))

	moduleTagButtons := make([]*chipButton, len(modules))
	runModeTagButtons := make([]*chipButton, len(runModes))
	cronBox := container.NewVBox()
	cronBox.Hide()
	hintText := ""
	hintUntil := time.Time{}
	pendingStop := false

	setHint := func(text string, duration time.Duration) {
		hintText = text
		hintUntil = time.Now().Add(duration)
	}

	updateCronInfo := func() {
		if selectedRunMode.NeedCron {
			expr := strings.TrimSpace(cronExpr)
			if expr == "" {
				_ = cronBinding.Set("未配置，请先在配置文件或 WebUI 中设置 Cron 表达式。")
			} else {
				_ = cronBinding.Set(expr)
			}
			cronBox.Show()
		} else {
			cronBox.Hide()
		}
	}

	updateTagButtons := func(buttons []*chipButton, current string, getter func(int) string) {
		for i, btn := range buttons {
			if btn == nil {
				continue
			}
			btn.SetSelected(getter(i) == current)
		}
	}

	updateCronInfo()

	var refreshStatus func()

	startAction := func() {
		switch selectedRunMode.Mode {
		case "once":
			started, err := runtime.StartRunOnce(selectedModule.Module)
			if err != nil {
				dialog.ShowError(err, win)
				return
			}
			if !started {
				dialog.ShowInformation("提示", "任务已在运行中", win)
				return
			}
			setHint("已启动", 3*time.Second)
			refreshStatus()
		case "cron":
			expr := strings.TrimSpace(cronExpr)
			if expr == "" {
				dialog.ShowInformation("提示", "当前未配置 Cron 表达式，请先打开 WebUI 配置。", win)
				return
			}
			_, _ = runtime.StartRunOnce(selectedModule.Module)
			if err := runtime.StartCron(expr, selectedModule.Module); err != nil {
				dialog.ShowError(err, win)
				return
			}
			setHint("计划任务已启动", 3*time.Second)
			refreshStatus()
		case "cronAft":
			expr := strings.TrimSpace(cronExpr)
			if expr == "" {
				dialog.ShowInformation("提示", "当前未配置 Cron 表达式，请先打开 WebUI 配置。", win)
				return
			}
			if err := runtime.StartCron(expr, selectedModule.Module); err != nil {
				dialog.ShowError(err, win)
				return
			}
			setHint("计划任务已启动", 3*time.Second)
			refreshStatus()
		case "clean":
			dialog.ShowConfirm("确认清理", "确定要清理所有 IKB 规则吗？", func(confirm bool) {
				if !confirm {
					return
				}
				started, err := runtime.StartRunOnce("clean")
				if err != nil {
					dialog.ShowError(err, win)
					return
				}
				if !started {
					dialog.ShowInformation("提示", "任务已在运行中", win)
					return
				}
				setHint("清理任务已启动", 3*time.Second)
				refreshStatus()
			}, win)
		}
	}

	stopAction := func() {
		status := runtime.Status()
		if !status.Running && !status.CronRunning {
			return
		}

		if status.CronRunning {
			if err := runtime.StopCron(); err != nil {
				dialog.ShowError(err, win)
				return
			}
			pendingStop = false
			setHint("计划任务已停止", 4*time.Second)
			appendLog("[GUI] cron stop requested")
			refreshStatus()
			return
		}

		pendingStop = true
		setHint("停止请求已发送，等待当前操作完成", 5*time.Second)
		appendLog("[GUI] run-once stop requested")
		refreshStatus()
	}

	actionButton := newRoundActionButton(func() {
		status := runtime.Status()
		if status.Running || status.CronRunning {
			stopAction()
			return
		}
		startAction()
	})

	refreshStatus = func() {
		status := runtime.Status()
		if !status.Running && !status.CronRunning {
			pendingStop = false
		}

		// 更新状态文本
		var subText string
		if hintText != "" && time.Now().Before(hintUntil) {
			subText = hintText
		} else if status.Running || status.CronRunning {
			if pendingStop && status.Running && !status.CronRunning {
				subText = "等待当前操作结束"
			} else if status.CronRunning {
				subText = "计划任务运行中"
			} else {
				subText = fmt.Sprintf("运行中: %s", getModuleLabel(status.Module))
			}
		} else {
			subText = "未启动"
		}

		// Set to the status label
		_ = statusBinding.Set("运行状态: " + subText)

		// 只有 启动/停止/停止中 三个字
		if status.Running || status.CronRunning {
			if pendingStop && status.Running && !status.CronRunning {
				actionButton.SetState(true, "停止中", "")
			} else {
				actionButton.SetState(true, "停 止", "")
			}
		} else {
			actionButton.SetState(false, "启 动", "")
		}
	}
	refreshStatus()

	go func() {
		ticker := time.NewTicker(1 * time.Second)
		defer ticker.Stop()
		for {
			select {
			case <-ctx.Done():
				return
			case <-ticker.C:
				fyne.Do(refreshStatus)
			}
		}
	}()

	// ========== 快捷操作区域 (齿轮菜单) ==========
	settingsMenu := widget.NewButtonWithIcon("", theme.SettingsIcon(), nil)
	settingsMenu.Importance = widget.LowImportance
	settingsMenu.OnTapped = func() {
		menu := fyne.NewMenu("",
			fyne.NewMenuItem("打开 WebUI 配置", func() {
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
			}),
			fyne.NewMenuItem("关于", func() {
				dialog.ShowInformation("关于", "iKuai Bypass v4.4.10\n智能分流管理工具", win)
			}),
		)
		// Pop it up under the button

		// Use canvas to show popup
		pos := fyne.CurrentApp().Driver().AbsolutePositionForObject(settingsMenu)
		pos.Y += settingsMenu.Size().Height
		widget.ShowPopUpMenuAtPosition(menu, win.Canvas(), pos)

	}

	// ========== 标签选择区域 ==========
	moduleTags := container.NewGridWithColumns(2)
	for i, module := range modules {
		idx := i
		btn := newChipButton(module.Label, func() {
			selectedModule = modules[idx]
			prefs.SetString(preferenceModuleKey, selectedModule.Module)
			updateTagButtons(moduleTagButtons, selectedModule.Module, func(i int) string {
				return modules[i].Module
			})
			refreshStatus()
		})
		moduleTagButtons[i] = btn
		moduleTags.Add(btn)
	}
	updateTagButtons(moduleTagButtons, selectedModule.Module, func(i int) string {
		return modules[i].Module
	})

	runModeTags := container.NewGridWithColumns(2)
	for i, mode := range runModes {
		idx := i
		btn := newChipButton(mode.Label, func() {
			selectedRunMode = runModes[idx]
			prefs.SetString(preferenceRunModeKey, selectedRunMode.Mode)
			updateTagButtons(runModeTagButtons, selectedRunMode.Mode, func(i int) string {
				return runModes[i].Mode
			})
			updateCronInfo()
			refreshStatus()
		})
		runModeTagButtons[i] = btn
		runModeTags.Add(btn)
	}
	updateTagButtons(runModeTagButtons, selectedRunMode.Mode, func(i int) string {
		return runModes[i].Mode
	})

	cronTitle := newSectionTitle("Cron 表达式 (仅在计划任务模式下显示)")
	cronBox.Add(container.NewVBox(
		cronTitle,
		cronLabel,
	))

	// ========== 日志区域 ==========
	// 使用 NewVScroll 确保只有垂直滚动，并且横向自动填充
	logOverride := container.NewThemeOverride(logLabel, &customLogTheme{Theme: app.Settings().Theme()})
	logScroll = container.NewVScroll(logOverride)
	logScroll.SetMinSize(fyne.NewSize(0, 100))

	clearLogBtn := widget.NewButtonWithIcon("", theme.DeleteIcon(), func() {
		logMutex.Lock()
		logLines = logLines[:0]
		logMutex.Unlock()
		_ = logBinding.Set("")
	})
	clearLogBtn.Importance = widget.LowImportance

	// ========== 头部区域 ==========
	// 合并主标题和副标题
	headerTitle := widget.NewLabelWithStyle("iKuai Bypass - 分流同步控制台", fyne.TextAlignLeading, fyne.TextStyle{Bold: true})

	statusLabel := widget.NewLabelWithData(statusBinding)
	statusLabel.TextStyle = fyne.TextStyle{Italic: true}
	statusLabel.Wrapping = fyne.TextWrapWord

	modeCard := newInfoCard(container.NewVBox(
		newSectionTitle("分流模式"),
		moduleTags,
		newSpacer(2),
		newSectionTitle("运行模式"),
		runModeTags,
		cronBox,
	))

	// 去掉运行日志几个字和外层 border
	logBg := canvas.NewRectangle(color.NRGBA{R: 255, G: 255, B: 255, A: 120})
	logBg.CornerRadius = 8

	// 删除按钮设为左下角悬浮
	floatBtnBox := container.NewVBox(
		layout.NewSpacer(),
		container.NewHBox(clearLogBtn, layout.NewSpacer()),
	)
	// 日志外层只用 Stack 叠起来即可。使用 Max 布局让日志可以全宽，不用 padded 避免 100px 空白
	logCard := container.NewStack(logBg, logScroll, floatBtnBox)

	// 顶部：标题和设置按钮
	topBar := container.NewBorder(nil, nil, headerTitle, settingsMenu)

	// 状态文字居中
	statusLabel.Alignment = fyne.TextAlignCenter

	// 按钮和状态文字区域（垂直堆叠并水平居中）
	actionArea := container.NewVBox(
		container.NewCenter(actionButton),
		newSpacer(4),
		statusLabel,
	)

	heroInfo := container.NewVBox(
		topBar,
		newSpacer(8),
		actionArea,
		newSpacer(4),
	)

	heroCard := newHeroCard(heroInfo)

	topPanel := container.NewVBox(heroCard, newSpacer(4), modeCard)

	body := container.NewBorder(
		topPanel,
		nil,
		nil,
		nil,
		logCard,
	)

	content := container.NewStack(
		newGradientBackground(),
		container.NewPadded(body),
	)

	win.SetContent(content)
	win.Resize(fyne.NewSize(360, 560))

	return win
}

// getModuleLabel 根据模块名获取显示标签
func getModuleLabel(module string) string {
	for _, m := range modules {
		if m.Module == module {
			return m.Label
		}
	}
	return module
}

func findModuleByValue(value string) ModuleConfig {
	for _, module := range modules {
		if module.Module == value {
			return module
		}
	}
	return modules[0]
}

func findRunModeByValue(value string) RunModeConfig {
	for _, runMode := range runModes {
		if runMode.Mode == value {
			return runMode
		}
	}
	return runModes[0]
}

// newGradientBackground 创建简洁的蓝色背景，不使用复杂装饰，保持界面直接清爽。
// Create a restrained blue background so the GUI feels cleaner and less abstract.
func newGradientBackground() fyne.CanvasObject {
	bg := canvas.NewLinearGradient(
		pageBlueTop,
		pageBlueBottom,
		0,
	)
	return bg
}

func newHeroCard(content fyne.CanvasObject) fyne.CanvasObject {
	bg := canvas.NewRectangle(color.NRGBA{R: 255, G: 255, B: 255, A: 18})
	bg.CornerRadius = 22
	border := canvas.NewRectangle(color.Transparent)
	border.StrokeColor = color.NRGBA{R: 255, G: 255, B: 255, A: 70}
	border.StrokeWidth = 1
	border.CornerRadius = 22
	return container.NewStack(bg, border, container.NewPadded(content))
}

// newInfoCard 使用统一卡片容器收敛视觉层级，避免零散控件导致界面发散。
// Wrap related controls inside a single white card for a calmer and more consistent layout.
func newInfoCard(content fyne.CanvasObject) fyne.CanvasObject {
	bg := canvas.NewRectangle(panelWhite)
	bg.CornerRadius = 16
	border := canvas.NewRectangle(color.Transparent)
	border.StrokeColor = panelBorder
	border.StrokeWidth = 1
	border.CornerRadius = 16
	return container.NewStack(bg, border, container.NewPadded(content))
}

func newSectionTitle(text string) fyne.CanvasObject {
	title := canvas.NewText(text, panelMuted)
	title.TextStyle = fyne.TextStyle{Bold: true}
	title.TextSize = 11
	return title
}

type roundActionButton struct {
	widget.BaseWidget
	onTapped func()
	active   bool
	title    string
	subtitle string
}

func newRoundActionButton(onTapped func()) *roundActionButton {
	btn := &roundActionButton{
		onTapped: onTapped,
		title:    "启 动",
		subtitle: "点击开始同步",
	}
	btn.ExtendBaseWidget(btn)
	return btn
}

func (b *roundActionButton) SetState(active bool, title string, subtitle string) {
	b.active = active
	b.title = title
	b.subtitle = subtitle
	b.Refresh()
}

func (b *roundActionButton) Tapped(*fyne.PointEvent) {
	if b.onTapped == nil {
		return
	}
	b.onTapped()
}

func (b *roundActionButton) TappedSecondary(*fyne.PointEvent) {}

func (b *roundActionButton) CreateRenderer() fyne.WidgetRenderer {
	outer := canvas.NewCircle(startRingOuter)
	inner := canvas.NewCircle(startRingInner)
	outer.StrokeWidth = 0
	inner.StrokeWidth = 4
	inner.StrokeColor = brandBlueSoft

	title := canvas.NewText(b.title, brandBlue)
	title.Alignment = fyne.TextAlignCenter
	title.TextStyle = fyne.TextStyle{Bold: true}
	title.TextSize = 14

	r := &roundActionButtonRenderer{
		button:  b,
		outer:   outer,
		inner:   inner,
		title:   title,
		objects: []fyne.CanvasObject{outer, inner, title},
	}
	r.Refresh()
	return r
}

type roundActionButtonRenderer struct {
	button  *roundActionButton
	outer   *canvas.Circle
	inner   *canvas.Circle
	title   *canvas.Text
	objects []fyne.CanvasObject
}

func (r *roundActionButtonRenderer) Layout(size fyne.Size) {
	side := float32(64) // Fixed size regardless of text length

	centerX := (size.Width - side) / 2
	centerY := (size.Height - side) / 2

	r.outer.Move(fyne.NewPos(centerX, centerY))
	r.outer.Resize(fyne.NewSize(side, side))

	innerInset := float32(4)
	r.inner.Move(fyne.NewPos(centerX+innerInset, centerY+innerInset))
	r.inner.Resize(fyne.NewSize(side-innerInset*2, side-innerInset*2))

	titleSize := r.title.MinSize()
	r.title.Move(fyne.NewPos(
		centerX+(side-titleSize.Width)/2,
		centerY+(side-titleSize.Height)/2,
	))
	r.title.Resize(titleSize)
}

func (r *roundActionButtonRenderer) MinSize() fyne.Size {
	return fyne.NewSize(64, 64)
}

func (r *roundActionButtonRenderer) Refresh() {
	if r.button.active {
		r.outer.FillColor = stopRingOuter
		r.inner.FillColor = stopRingInner
		r.inner.StrokeColor = color.NRGBA{R: 231, G: 109, B: 109, A: 255}
		r.title.Color = criticalStopText
	} else {
		r.outer.FillColor = startRingOuter
		r.inner.FillColor = startRingInner
		r.inner.StrokeColor = brandBlueSoft
		r.title.Color = brandBlue
	}

	r.title.Text = r.button.title
	canvas.Refresh(r.outer)
	canvas.Refresh(r.inner)
	canvas.Refresh(r.title)
}

func (r *roundActionButtonRenderer) Destroy() {}

func (r *roundActionButtonRenderer) Objects() []fyne.CanvasObject {
	return r.objects
}

func (r *roundActionButtonRenderer) BackgroundColor() color.Color {
	return color.Transparent
}

type chipButton struct {
	widget.BaseWidget
	label    string
	selected bool
	onTapped func()
}

func newChipButton(label string, onTapped func()) *chipButton {
	btn := &chipButton{
		label:    label,
		onTapped: onTapped,
	}
	btn.ExtendBaseWidget(btn)
	return btn
}

func (b *chipButton) SetSelected(selected bool) {
	b.selected = selected
	b.Refresh()
}

func (b *chipButton) Tapped(*fyne.PointEvent) {
	if b.onTapped != nil {
		b.onTapped()
	}
}

func (b *chipButton) TappedSecondary(*fyne.PointEvent) {}

func (b *chipButton) CreateRenderer() fyne.WidgetRenderer {
	bg := canvas.NewRectangle(panelMutedSoft)
	bg.CornerRadius = 14
	border := canvas.NewRectangle(color.Transparent)
	border.CornerRadius = 14
	border.StrokeWidth = 1
	border.StrokeColor = panelBorder
	text := canvas.NewText(b.label, panelMuted)
	text.Alignment = fyne.TextAlignCenter
	text.TextSize = 10
	text.TextStyle = fyne.TextStyle{Bold: true}

	r := &chipButtonRenderer{
		button:  b,
		bg:      bg,
		border:  border,
		text:    text,
		objects: []fyne.CanvasObject{bg, border, text},
	}
	r.Refresh()
	return r
}

type chipButtonRenderer struct {
	button  *chipButton
	bg      *canvas.Rectangle
	border  *canvas.Rectangle
	text    *canvas.Text
	objects []fyne.CanvasObject
}

func (r *chipButtonRenderer) Layout(size fyne.Size) {
	r.bg.Resize(size)
	r.border.Resize(size)
	textSize := r.text.MinSize()
	r.text.Move(fyne.NewPos((size.Width-textSize.Width)/2, (size.Height-textSize.Height)/2))
	r.text.Resize(textSize)
}

func (r *chipButtonRenderer) MinSize() fyne.Size {
	return fyne.NewSize(72, 22)
}

func (r *chipButtonRenderer) Refresh() {
	if r.button.selected {
		r.bg.FillColor = brandBlue
		r.border.StrokeColor = brandBlueDeep
		r.text.Color = color.White
	} else {
		r.bg.FillColor = color.NRGBA{R: 240, G: 245, B: 252, A: 255}
		r.border.StrokeColor = color.NRGBA{R: 186, G: 203, B: 228, A: 255}
		r.text.Color = panelText
	}
	r.text.Text = r.button.label
	canvas.Refresh(r.bg)
	canvas.Refresh(r.border)
	canvas.Refresh(r.text)
}

func (r *chipButtonRenderer) Destroy() {}

func (r *chipButtonRenderer) Objects() []fyne.CanvasObject {
	return r.objects
}

func (r *chipButtonRenderer) BackgroundColor() color.Color {
	return color.Transparent
}

func newSpacer(height float32) fyne.CanvasObject {
	spacer := canvas.NewRectangle(color.Transparent)
	spacer.SetMinSize(fyne.NewSize(1, height))
	return spacer
}

type customLogTheme struct {
	fyne.Theme
}

func (c *customLogTheme) Size(n fyne.ThemeSizeName) float32 {
	if n == theme.SizeNameText {
		return 10
	}
	return c.Theme.Size(n)
}
