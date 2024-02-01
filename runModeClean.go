package main

import (
	"log"

	"github.com/joyanhui/ikuai-bypass/api"
	"github.com/joyanhui/ikuai-bypass/router"
)

// 清理旧分流规则
func clean() {
	err := readConf(*confPath)
	if err != nil {
		log.Println("更新配置文件失败：", err)
		return
	}

	baseurl := conf.IkuaiURL
	if baseurl == "" {
		gateway, err := router.GetGateway()
		if err != nil {
			log.Println("获取默认网关失败：", err)
			return
		}
		baseurl = "http://" + gateway
		log.Println("使用默认网关地址：", baseurl)
	}

	iKuai := api.NewIKuai(baseurl)

	err = iKuai.Login(conf.Username, conf.Password)
	if err != nil {
		log.Println("ikuai 登陆失败：", baseurl, err)
		return
	} else {
		log.Println("ikuai 登录成功", baseurl)
	}
	//删除旧的自定义运营商
	err = iKuai.DelCustomIspAll(*cleanTag)
	if err != nil {
		log.Println("移除旧的自定义运营商失败 tag:"+*cleanTag+"：", err)
	} else {
		log.Println("移除旧的自定义运营商成功 tag:" + *cleanTag)
	}
	//删除旧的域名分流
	err = iKuai.DelStreamDomainAll(*cleanTag)
	if err != nil {
		log.Println("移除旧的域名分流失败 tag:"+*cleanTag+"：", err)
	} else {
		log.Println("移除旧的域名分流成功 tag:" + *cleanTag)
	}

}
