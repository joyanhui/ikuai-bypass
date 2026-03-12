package main

import (
	"context"
	"time"

	"ikuai-bypass/apps/gui/ui"
	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/logger"
	"ikuai-bypass/pkg/service"
	"ikuai-bypass/pkg/utils"
	"ikuai-bypass/pkg/webui"

	"fyne.io/fyne/v2/app"
)

const configTemplateURL = "https://raw.githubusercontent.com/joyanhui/ikuai-bypass/refs/heads/main/config.yml"

func main() {
	configPath := config.DefaultConfigPath()
	*config.ConfPath = configPath
	if err := config.EnsureConfigFromURL(configPath, configTemplateURL); err != nil {
		utils.SysLog.Error("CONF:配置读取", "Failed to ensure config: %v", err)
		return
	}
	if err := config.Read(*config.ConfPath); err != nil {
		utils.SysLog.Error("CONF:配置读取", "Failed to read configuration file: %v", err)
		return
	}

	config.GlobalConfig.WebUI.Enable = true

	broker := service.NewLogBroker(5000)
	logger.AddOutput(broker.Writer())
	rt := service.NewRuntimeService("", config.GlobalConfig.Cron, broker)
	webui.SetRuntimeController(rt)

	web, err := webui.StartServerAsync(true)
	if err != nil {
		utils.SysLog.Error("WEB:启动失败", "Failed to start WebUI: %v", err)
		return
	}
	defer func() {
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		_ = web.Shutdown(ctx)
	}()

	fontRes, err := loadEmbeddedFont()
	if err != nil {
		utils.SysLog.Error("GUI:字体", "Failed to load font: %v", err)
		return
	}

	gui := app.New()
	gui.Settings().SetTheme(ui.NewChineseTheme(fontRes))

	win := ui.BuildMainWindow(gui, rt, web.URL, config.GlobalConfig.Cron)
	// 窗口大小在 BuildMainWindow 中设置，使用移动端尺寸
	win.ShowAndRun()
}
