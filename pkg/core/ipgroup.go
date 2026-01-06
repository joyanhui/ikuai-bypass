package core

import (
	"log"

	"github.com/dscao/ikuai-bypass/pkg/config"
	"github.com/dscao/ikuai-bypass/pkg/utils"
)

func UpdateIpgroup() {
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

	if *config.DelOldRule == "before" {
		err = iKuai.DelIKuaiBypassStreamIpPort("cleanAll")
		if err != nil {
			log.Println("端口分流== 删除旧的端口分流失败,退出：", err)
			return
		} else {
			log.Println("端口分流== 删除旧的端口分流成功")
		}
	}
	for _, streamIpPort := range config.GlobalConfig.StreamIpPort {
		if *config.DelOldRule == "after" {
			preIds, err := iKuai.GetStreamIpPortIds(streamIpPort.IpGroup)
			if err != nil {
				log.Println("端口分流== 获取准备更新的端口分流列表失败：", streamIpPort.IpGroup, err)
				continue
			} else {
				log.Println("端口分流== 获取准备更新的端口分流列表成功", streamIpPort.IpGroup, preIds)
			}

			err = utils.UpdateStreamIpPort(
				iKuai,
				streamIpPort.Type,
				streamIpPort.Interface,
				streamIpPort.Nexthop,
				streamIpPort.SrcAddr,
				streamIpPort.IpGroup,
				streamIpPort.Mode,
				streamIpPort.IfaceBand,
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
				err = iKuai.DelStreamIpPort(preIds)
				if err == nil {
					log.Println("端口分流== 删除旧的端口分流列表成功", streamIpPort.IpGroup, preIds)
					log.Println("端口分流== 更新完成", streamIpPort.IpGroup)
				} else {
					log.Println("端口分流== 删除旧的端口分流列表有错误", streamIpPort.IpGroup, err)
				}
			}
		} else {
			err := utils.UpdateStreamIpPort(
				iKuai,
				streamIpPort.Type,
				streamIpPort.Interface,
				streamIpPort.Nexthop,
				streamIpPort.SrcAddr,
				streamIpPort.IpGroup,
				streamIpPort.Mode,
				streamIpPort.IfaceBand,
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
			}
		}
	}
}

func UpdateIpv6group() {
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
