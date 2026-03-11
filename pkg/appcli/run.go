package appcli

import (
	"flag"
	"os"
	"os/signal"
	"syscall"

	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/core"
	"ikuai-bypass/pkg/ikuai_common"
	"ikuai-bypass/pkg/utils"
	"ikuai-bypass/pkg/webui"

	"github.com/robfig/cron/v3"
)

func Run() {
	flag.Parse()

	utils.SysLog.Info("START:启动程序", "Run mode: %s, Config path: '%s'", *config.RunMode, *config.ConfPath)
	err := config.Read(*config.ConfPath)
	if err != nil {
		utils.SysLog.Error("CONF:配置读取", "Failed to read configuration file: %v", err)
		return
	}
	switch *config.RunMode {
	case "web":
		utils.SysLog.Info("MODE:运行模式", "WebUI mode - starting web service")
		config.GlobalConfig.WebUI.Enable = true
		webui.OnDemandStartUpWebUI()
		return
	case "cron":
		utils.SysLog.Info("MODE:运行模式", "Cron mode - executing once then entering scheduled mode")
		go webui.OnDemandStartUpWebUI()
		core.RunUpdateByModule(*config.IsAcIpgroup)
	case "cronAft":
		utils.SysLog.Info("MODE:运行模式", "CronAft mode - scheduled execution only")
		go webui.OnDemandStartUpWebUI()
	case "nocron", "once", "1":
		core.RunUpdateByModule(*config.IsAcIpgroup)
		utils.SysLog.Success("END:运行完毕", "Once mode execution completed, exiting...")
		return
	case "clean":
		utils.SysLog.Info("MODE:运行模式", "Clean mode")
		if *config.CleanTag == ikuai_common.CleanModeAll {
			utils.SysLog.Info("CLEAN:清理范围", "Clearing all rules with prefix IKB (includes legacy notes)")
		} else {
			utils.SysLog.Info("CLEAN:清理范围", "Clearing rules with TagName or Name: %s", *config.CleanTag)
		}
		core.MainClean()
		return
	default:
		utils.SysLog.Error("ERR:参数错误", "Invalid -r parameter: %s", *config.RunMode)
		return
	}

	if config.GlobalConfig.Cron != "" {
		c := cron.New()
		_, err = c.AddFunc(config.GlobalConfig.Cron, func() { core.RunUpdateByModule(*config.IsAcIpgroup) })
		if err != nil {
			utils.SysLog.Error("CRON:定时任务", "Failed to start scheduled task: %v", err)
			return
		} else {
			utils.SysLog.Success("CRON:定时任务", "Scheduled task started: %s", config.GlobalConfig.Cron)
		}
		c.Start()
	} else if *config.RunMode != "web" {
		utils.SysLog.Info("CRON:定时任务", "Cron configuration is empty, exiting...")
		return
	}

	{
		osSignals := make(chan os.Signal, 1)
		signal.Notify(osSignals, os.Interrupt, syscall.SIGTERM)
		<-osSignals
	}
}
