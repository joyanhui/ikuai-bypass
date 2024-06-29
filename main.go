package main

import (
	"flag"
	"github.com/robfig/cron/v3"
	"log"
	"os"
	"os/signal"
	"syscall"
)

var confPath = flag.String("c", "./config.yml", "配置文件路径")
var runMode = flag.String("r", "cron", "运行模式")
var isAcIpgroup = flag.String("m", "ispdomain", "启用ip分组和下一条网关模式")
var cleanTag = flag.String("tag", "cleanAll", "规则名称") //COMMENT_IKUAI_BYPASS
var exportPath = flag.String("exportPath", "/tmp", "导出文件路径")
var ikuaiLoginInfo = flag.String("login", "", "爱快登陆地址,用户名,密码")

func main() {
	flag.Parse()

	if *cleanTag != "cleanAll" {
		//检查规则名称中是否包含前缀 COMMENT_IKUAI_BYPASS，如果没有添加上
		if len(*cleanTag) < len("IKUAI_BYPASS") || (*cleanTag)[:len("IKUAI_BYPASS")] != "IKUAI_BYPASS" {
			*cleanTag = "IKUAI_BYPASS_" + *cleanTag
		}
	}

	log.Println("运行模式", *runMode, "配置文件", *confPath)
	err := readConf(*confPath)
	//log.Println(conf)
	if err != nil {
		log.Println("读取配置文件失败：", err)
		return
	}
	switch *runMode { //运行模式选择
	case "exportDomainSteamToTxt":
		log.Println("导出域名分流规则到txt,可以从爱快内导入 ")
		log.Println("导出路径:", *exportPath)
		exportDomainSteamToTxt()
		return
	case "cron":
		log.Println("cron 模式,执行一次，然后进入定时执行模式")
		updateEntrance()
	case "cronAft":
		log.Println("cronAft 模式，暂时不执行，稍后定时执行")
	case "nocron", "once", "1":
		updateEntrance()
		log.Println("once 模式 执行完毕自动退出")
		return
	case "clean":
		log.Println("清理模式")
		if *cleanTag == "cleanAll" {
			log.Println("清理所有备注中包含", "IKUAI_BYPASS", "的规则")
		} else {
			log.Println("清理规则备注为：", *cleanTag, "的规则")
		}
		clean()
		return
	default:
		log.Println("-r 参数错误")
		return
	}
	// 定时任务启动和检查  ================= start
	if conf.Cron == "" {
		log.Println("Cron配为空 自动推出")
		return
	}

	c := cron.New()
	_, err = c.AddFunc(conf.Cron, updateEntrance)
	if err != nil {
		log.Println("启动计划任务失败：", err)
		return
	} else {
		log.Println("已启动计划任务", conf.Cron)
	}
	c.Start()

	{
		osSignals := make(chan os.Signal, 1)
		signal.Notify(osSignals, os.Interrupt, os.Kill, syscall.SIGTERM)
		<-osSignals
	}
	// 定时任务启动和检查  ================= end

}

func updateEntrance() {
	switch *isAcIpgroup {
	case "ispdomain":
		log.Println("启动 ... 自定义isp和域名分流模式 模式")
		updateIspRule()
	case "ipgroup":
		log.Println("启动 ... ip分组和下一条网关模式")
		updateIpgroup()
	case "ii":
		log.Println("启动 ...  自定义isp和域名分流模式 模式")
		log.Println("启动 ... ip分组和下一条网关模式")
		updateIspRule()
		updateIpgroup()
	}

}
