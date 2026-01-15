package main

import (
	"flag"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/robfig/cron/v3"
	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/core"
	"ikuai-bypass/pkg/webui"
)

func main() {
	flag.Parse()

	if *config.CleanTag != "cleanAll" {
		//检查规则名称中是否包含前缀 IKUAI_BYPASS，如果没有添加上
		if len(*config.CleanTag) < len("IKUAI_BYPASS") || (*config.CleanTag)[:len("IKUAI_BYPASS")] != "IKUAI_BYPASS" {
			*config.CleanTag = "IKUAI_BYPASS_" + *config.CleanTag
		}
	}

	log.Println("运行模式", *config.RunMode, "配置文件", *config.ConfPath)
	err := config.Read(*config.ConfPath)
	if err != nil {
		log.Println("读取配置文件失败：", err)
		return
	}
	switch *config.RunMode { //运行模式选择
	case "exportDomainSteamToTxt":
		log.Println("导出域名分流规则到txt,可以从爱快内导入 ")
		log.Println("导出路径:", *config.ExportPath)
		core.ExportDomainSteamToTxt()
		return
	case "web":
		log.Println("WebUI 模式 不做其他操作")
		config.GlobalConfig.WebUI.Enable = true
		webui.IsAndStartWebUI()
		return
	case "cron":
		log.Println("cron 模式,执行一次，然后进入定时执行模式")
		go webui.IsAndStartWebUI()
		updateEntrance() //马上执行依次
	case "cronAft":
		log.Println("cronAft 模式，暂时不执行，稍后定时执行")
		go webui.IsAndStartWebUI()
	case "nocron", "once", "1":
		updateEntrance()
		log.Println("once 模式 执行完毕自动退出")
		return
	case "clean":
		log.Println("清理模式")
		if *config.CleanTag == "cleanAll" {
			log.Println("清理所有备注中包含", "IKUAI_BYPASS", "的规则")
		} else {
			log.Println("清理规则备注为：", *config.CleanTag, "的规则")
		}
		core.Clean()
		return

	default:
		log.Println("-r 参数错误")
		return
	}
	// 定时任务启动和检查  ================= start
	if config.GlobalConfig.Cron != "" {
		c := cron.New()
		_, err = c.AddFunc(config.GlobalConfig.Cron, updateEntrance)
		if err != nil {
			log.Println("启动计划任务失败：", err)
			return
		} else {
			log.Println("已启动计划任务", config.GlobalConfig.Cron)
		}
		c.Start()
	} else if *config.RunMode != "web" {
		log.Println("Cron配置为空 自动退出")
		return
	}

	{
		osSignals := make(chan os.Signal, 1)
		signal.Notify(osSignals, os.Interrupt, syscall.SIGTERM)
		<-osSignals
	}
	// 定时任务启动和检查  ================= end

}

func updateEntrance() {
	switch *config.IsAcIpgroup {
	case "ispdomain":
		log.Println("启动 ... 自定义isp和域名分流模式 模式")
		core.UpdateIspRule()
	case "ipgroup":
		log.Println("启动 ... ip分组和下一条网关模式")
		core.UpdateIpgroup()
	case "ipv6group":
		log.Println("启动 ... ipv6分组")
		core.UpdateIpv6group()
	case "ii":
		log.Println("先 启动 ...  自定义isp和域名分流模式 模式")
		log.Println("再 启动 ... ip分组和下一条网关模式")
		core.UpdateIspRule()
		core.UpdateIpgroup()
	case "ip":
		log.Println("先 启动 ...  ip分组和下一条网关模式")
		log.Println("再 启动 ... ipv6分组")
		core.UpdateIpgroup()
		core.UpdateIpv6group()
	}

}
