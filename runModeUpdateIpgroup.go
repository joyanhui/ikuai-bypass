package main

import (
	"log"
)

func updateIpgroup() {
	iKuai, err := loginToIkuai()
	if err != nil {
		log.Println("登录爱快失败：", err)
		return
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
