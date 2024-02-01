package main

import (
	"flag"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/robfig/cron/v3"
)

var confPath = flag.String("c", "./config.yml", "配置文件路径")
var runMode = flag.String("r", "cron", "运行模式")
var ruleName = flag.String("rn", "all", "规则名称") //COMMENT_IKUAI_BYPASS
var conf struct {
	IkuaiURL  string `yaml:"ikuai-url"`
	Username  string `yaml:"username"`
	Password  string `yaml:"password"`
	Cron      string `yaml:"cron"`
	CustomIsp []struct {
		Name string `yaml:"name"`
		URL  string `yaml:"url"`
	} `yaml:"custom-isp"`
	StreamDomain []struct {
		Interface string `yaml:"interface"`
		SrcAddr   string `yaml:"src-addr"`
		URL       string `yaml:"url"`
	} `yaml:"stream-domain"`
}

func main() {
	flag.Parse()

	log.Println("运行模式", *runMode, "配置文件", *confPath)
	err := readConf(*confPath)
	if err != nil {
		log.Println("读取配置文件失败：", err)
		return
	}
	switch *runMode { //运行模式选择
	case "cron":
		log.Println("cronA 模式,执行一次，然后进入定时执行模式")
		update()
	case "cronAft":
		log.Println("cronAft 模式稍后定时执行")
	case "nocron":
		update()
		log.Println("nocron 自动推出")
		return
	case "clean":
		log.Println("清理模式")
		clean()
		return
	default:
		log.Println("-r 参数错误")
		return
	}

	if conf.Cron == "" {
		log.Println("Cron配为空 自动推出")
		return
	}

	c := cron.New()
	_, err = c.AddFunc(conf.Cron, update)
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

}
