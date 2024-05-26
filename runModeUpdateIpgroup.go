package main

import (
	"github.com/joyanhui/ikuai-bypass/api"
	"github.com/joyanhui/ikuai-bypass/router"
	"log"
)

func updateIpgroup() {
	err := readConf(*confPath)
	if err != nil {
		log.Println("读取配置文件失败：", err)
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

	err = iKuai.DelIKuaiBypassIpGroup()
	if err != nil {
		log.Println("ip分组== 移除旧的IP分组失败：", err)
	} else {
		log.Println("ip分组== 移除旧的IP分组成功")
	}
	for _, ipGroup := range conf.IpGroup {
		err = updateIpGroup(iKuai, ipGroup.Name, ipGroup.URL)
		if err != nil {
			log.Printf("ip分组== 添加IP分组'%s@%s'失败：%s\n", ipGroup.Name, ipGroup.URL, err)
		} else {
			log.Printf("ip分组== 添加IP分组'%s@%s'成功\n", ipGroup.Name, ipGroup.URL)
		}
	}

	err = iKuai.DelIKuaiBypassStreamIpPort()
	if err != nil {
		log.Println("端口分流== 移除旧的端口分流失败：", err)
	} else {
		log.Println("端口分流== 移除旧的端口分流成功")
	}
	for _, streamIpPort := range conf.StreamIpPort {
		err = updateStreamIpPort(iKuai, streamIpPort.Type, streamIpPort.Interface, streamIpPort.Nexthop, streamIpPort.SrcAddr, streamIpPort.IpGroup)
		if err != nil {
			log.Printf("端口分流== 添加端口分流 '%s@%s' 失败：%s\n", streamIpPort.Interface+streamIpPort.Nexthop, streamIpPort.IpGroup, err)
		} else {
			log.Printf("端口分流== 添加端口分流 '%s@%s' 成功\n", streamIpPort.Interface+streamIpPort.Nexthop, streamIpPort.IpGroup)
		}
	}

}
