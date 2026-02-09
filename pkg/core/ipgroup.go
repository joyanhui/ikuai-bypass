package core

import (
	"log"

	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/utils"
)

func MainUpdateIpgroup() {
	iKuai, err := utils.LoginToIkuai()
	if err != nil {
		log.Println("登录爱快失败：", err)
		return
	}

	for _, ipGroup := range config.GlobalConfig.IpGroup {
		err := utils.UpdateIpGroup(iKuai, ipGroup.Name, ipGroup.URL)
		if err != nil {
			log.Printf("ip分组== 添加IP分组'%s@%s'失败：%s\n", ipGroup.Name, ipGroup.URL, err)
		} else {
			log.Printf("ip分组== 添加IP分组'%s@%s'成功\n", ipGroup.Name, ipGroup.URL)
		}
	}

	isVersion4 := config.GlobalConfig.IkuaiVersion == "4"

	if !isVersion4 && *config.DelOldRule == "before" {
		err = iKuai.DelIKuaiBypassStreamIpPort("cleanAll")
		if err != nil {
			log.Println("端口分流== 删除旧的端口分流失败,退出：", err)
			return
		} else {
			log.Println("端口分流== 删除旧的端口分流成功")
		}
	}

	// 更新端口分流规则
	for _, streamIpPort := range config.GlobalConfig.StreamIpPort {
		var tag string
		if streamIpPort.OptTagName != "" {
			tag = streamIpPort.OptTagName
		} else {
			tag = streamIpPort.Interface + streamIpPort.Nexthop
		}
		if tag == "" {
			log.Println("端口分流== err 规则名称和IpGroup不能同时为空，跳过该规则:", streamIpPort)
			continue
		}

		preIds, err := iKuai.GetStreamIpPortIdsByTag(tag)
		if err != nil {
			log.Println("端口分流== 获取准备更新的端口分流列表失败：", tag, err)
			continue
		}

		preDelIds := ""
		if isVersion4 || *config.DelOldRule == "before" {
			preDelIds = preIds
		}

		err = utils.UpdateStreamIpPort(
			iKuai,
			streamIpPort.Type, tag,
			streamIpPort.Interface,
			streamIpPort.Nexthop,
			streamIpPort.SrcAddr, streamIpPort.SrcAddrOptIpGroup,
			streamIpPort.IpGroup,
			streamIpPort.Mode,
			streamIpPort.IfaceBand,
			preDelIds,
		)

		if err != nil {
			log.Printf("端口分流== 添加端口分流 '%s@%s' 失败：%s\n",
				streamIpPort.Interface+streamIpPort.Nexthop,
				streamIpPort.IpGroup,
				err,
			)
		} else {
			log.Printf("端口分流== 添加端口分流 '%s@%s' 成功\n",
				streamIpPort.Interface+streamIpPort.Nexthop,
				streamIpPort.IpGroup,
			)
			if !isVersion4 && *config.DelOldRule == "after" {
				err = iKuai.DelStreamIpPort(preIds)
				if err == nil {
					log.Println("端口分流== 删除旧的端口分流列表成功", tag, preIds)
				} else {
					log.Println("端口分流== 删除旧的端口分流列表有错误", tag, err)
				}
			}
		}
	}
}

func MainUpdateIpv6group() {
	iKuai, err := utils.LoginToIkuai()
	if err != nil {
		log.Println("登录爱快失败：", err)
		return
	}
	for _, ipv6Group := range config.GlobalConfig.Ipv6Group {
		err := utils.UpdateIpv6Group(iKuai, ipv6Group.Name, ipv6Group.URL)
		if err != nil {
			log.Printf("ipv6分组== 添加IPV6分组'%s@%s'失败：%s\n", ipv6Group.Name, ipv6Group.URL, err)
		} else {
			log.Printf("ipv6分组== 添加IPV6分组'%s@%s'成功\n", ipv6Group.Name, ipv6Group.URL)
		}
	}
}