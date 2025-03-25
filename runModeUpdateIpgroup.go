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
	log.Println("Tips: 在添加之前会强制删除所有备注包含 IKUAI_BYPASS 字符的ip分组和端口分流，不受delOldRule参数影响,后续版本会尝试完善 2024-10-04 by joyanhui")

	err = iKuai.DelIKuaiBypassIpGroup("cleanAll")
	if err != nil {
		log.Println("ip分组== 删除旧的IP分组失败,退出：", err)
		return
	} else {
		log.Println("ip分组== 删除旧的IP分组成功")
	}
	for _, ipGroup := range conf.IpGroup {
		err = updateIpGroup(iKuai, ipGroup.Name, ipGroup.URL)
		if err != nil {
			log.Printf("ip分组== 添加IP分组'%s@%s'失败：%s\n", ipGroup.Name, ipGroup.URL, err)
		} else {
			log.Printf("ip分组== 添加IP分组'%s@%s'成功\n", ipGroup.Name, ipGroup.URL)

		}
	}

	err = iKuai.DelIKuaiBypassStreamIpPort("cleanAll")
	if err != nil {
		log.Println("端口分流== 删除旧的端口分流失败,退出：", err)
		return
	} else {
		log.Println("端口分流== 删除旧的端口分流成功")
	}
	mode = SetValue(streamIpPort.mode)
	ifaceband = SetValue(streamIpPort.ifaceband)
	for _, streamIpPort := range conf.StreamIpPort {
		err = updateStreamIpPort(iKuai, streamIpPort.Type, streamIpPort.Interface, streamIpPort.Nexthop, streamIpPort.SrcAddr, streamIpPort.IpGroup, mode, ifaceband)
		if err != nil {
			log.Printf("端口分流== 添加端口分流 '%s@%s' 失败：%s\n", streamIpPort.Interface+streamIpPort.Nexthop, streamIpPort.IpGroup, err)
		} else {
			log.Printf("端口分流== 添加端口分流 '%s@%s' 成功\n", streamIpPort.Interface+streamIpPort.Nexthop, streamIpPort.IpGroup)
		}
	}

}

func SetValue(values ...int) int {
    if len(values) == 0 {
        return 0  // 未传参数时返回 0
    }
    return values[0]
}

func updateIpv6group() {
	iKuai, err := loginToIkuai()
	if err != nil {
		log.Println("登录爱快失败：", err)
		return
	}
	log.Println("Tips: 在添加之前会强制删除所有备注包含 IKUAI_BYPASS 字符的ipv6分组")

	err = iKuai.DelIKuaiBypassIpv6Group("cleanAll")
	if err != nil {
		log.Println("ipv6分组== 删除旧的IPV6分组失败,退出：", err)
		return
	} else {
		log.Println("ipv6分组== 删除旧的IPV6分组成功")
	}
	for _, ipv6Group := range conf.Ipv6Group {
		err = updateIpv6Group(iKuai, ipv6Group.Name, ipv6Group.URL)
		if err != nil {
			log.Printf("ipv6分组== 添加IPV6分组'%s@%s'失败：%s\n", ipv6Group.Name, ipv6Group.URL, err)
		} else {
			log.Printf("ipv6分组== 添加IPV6分组'%s@%s'成功\n", ipv6Group.Name, ipv6Group.URL)

		}
	}
}
