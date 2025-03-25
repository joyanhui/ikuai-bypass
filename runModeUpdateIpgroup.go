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

	//添加之前会强制删除所有备注包含 IKUAI_BYPASS 字符的ip分组和端口分流，效率更高，但如果添加失败原来的分组数据会丢失。
	if *delOldRule == "before" {
		log.Println("Tips: 在添加之前会强制删除所有备注包含 IKUAI_BYPASS 字符的ip分组和端口分流")
		err = iKuai.DelIKuaiBypassIpGroup("cleanAll")
		if err != nil {
			log.Println("ip分组== 删除旧的IP分组失败,退出：", err)
			return
		} else {
			log.Println("ip分组== 删除旧的IP分组成功")
		}
	}

	for _, ipGroup := range conf.IpGroup {
		if *delOldRule == "after" {
			//记录旧的ip分组
			var preIds []string
			preIds, err = iKuai.GetIpGroup(ipGroup.Name)
			if err != nil {
				log.Println("ip分组== 获取准备更新的IP分组列表失败：", ipGroup.Name, err)
				//return
				break
			} else {
				log.Println("ip分组== 获取准备更新的IP分组列表成功", ipGroup.Name)
			}
		}
		err = updateIpGroup(iKuai, ipGroup.Name, ipGroup.URL)
		if err != nil {
			log.Printf("ip分组== 添加IP分组'%s@%s'失败：%s\n", ipGroup.Name, ipGroup.URL, err)
		} else {
			log.Printf("ip分组== 添加IP分组'%s@%s'成功\n", ipGroup.Name, ipGroup.URL)
			if *delOldRule == "after" {
				//删除旧的ip分组
				err = iKuai.DelIpGroup(preIds)
				if err == nil {
					log.Println("ip分组== 删除旧的IP分组列表成功", ipGroup.Name)
					log.Println("ip分组== 更新完成", ipGroup.Name)
				} else {
					log.Println("ip分组== 删除旧的IP分组列表有错误", ipGroup.Name, err)
				}
			}
		}
	}

	if *delOldRule == "before" {
		//在添加之前会强制删除所有备注包含 IKUAI_BYPASS 字符的端口分流
		err = iKuai.DelIKuaiBypassStreamIpPort("cleanAll")
		if err != nil {
			log.Println("端口分流== 删除旧的端口分流失败,退出：", err)
			return
		} else {
			log.Println("端口分流== 删除旧的端口分流成功")
		}
	}
	for _, streamIpPort := range conf.StreamIpPort {
		if *delOldRule == "after" {
			//记录旧的ip分组
			var preIds []string
			preIds, err = iKuai.GetStreamIpPortIds(streamIpPort.IpGroup)
			if err != nil {
				log.Println("端口分流== 获取准备更新的端口分流列表失败：", streamIpPort.IpGroup, err)
				//return
				break
			} else {
				log.Println("端口分流== 获取准备更新的端口分流列表成功", streamIpPort.IpGroup)
			}
		}
	
		err = updateStreamIpPort(
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
			if *delOldRule == "after" {
				//删除旧的端口分流
				err = iKuai.DelStreamIpPort(preIds)
				if err == nil {
					log.Println("端口分流== 删除旧的端口分流列表成功", streamIpPort.IpGroup)
					log.Println("端口分流== 更新完成", streamIpPort.IpGroup)
				} else {
					log.Println("端口分流== 删除旧的端口分流列表有错误", streamIpPort.IpGroup, err)
				}
			}
		}
	}

}


func updateIpv6group() {
	iKuai, err := loginToIkuai()
	if err != nil {
		log.Println("登录爱快失败：", err)
		return
	}
	if *delOldRule == "before" {
		log.Println("Tips: 在添加之前会强制删除所有备注包含 IKUAI_BYPASS 字符的ipv6分组")
		err = iKuai.DelIKuaiBypassIpv6Group("cleanAll")
		if err != nil {
			log.Println("ipv6分组== 删除旧的IPV6分组失败,退出：", err)
			return
		} else {
			log.Println("ipv6分组== 删除旧的IPV6分组成功")
		}
	}
	for _, ipv6Group := range conf.Ipv6Group {
		if *delOldRule == "after" {
			//记录旧的ipv6分组
			var preIds []string
			preIds, err = iKuai.GetIpv6Group(ipv6Group.Name)
			if err != nil {
				log.Println("ipv6分组== 获取准备更新的IPv6分组列表失败：", ipv6Group.Name, err)
				//return
				break
			} else {
				log.Println("ipv6分组== 获取准备更新的IPv6分组列表成功", ipv6Group.Name)
			}
		}
		err = updateIpv6Group(iKuai, ipv6Group.Name, ipv6Group.URL)
		if err != nil {
			log.Printf("ipv6分组== 添加IPV6分组'%s@%s'失败：%s\n", ipv6Group.Name, ipv6Group.URL, err)
		} else {
			log.Printf("ipv6分组== 添加IPV6分组'%s@%s'成功\n", ipv6Group.Name, ipv6Group.URL)
			if *delOldRule == "after" {
				//删除旧的ipv6分组
				err = iKuai.DelIpv6Group(preIds)
				if err == nil {
					log.Println("ipv6分组== 删除旧的IPv6分组列表成功", ipv6Group.Name)
					log.Println("ipv6分组== 更新完成", ipv6Group.Name)
				} else {
					log.Println("ipv6分组== 删除旧的IPv6分组列表有错误", ipv6Group.Name, err)
				}
			}
		}
	}
}
