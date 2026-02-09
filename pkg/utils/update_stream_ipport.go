package utils

import (
	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/ikuai_common"
	"log"
	"strings"
	"time"
)

// UpdateStreamIpPort 更新ip端口分流
func UpdateStreamIpPort(iKuai ikuai_common.IKuaiClient, forwardType string, tag string, iface string, nexthop string, srcAddr string, srcAddrOptTag string, ipGroupName string, mode int, ifaceband int, preDelIds string) (err error) {

	// #101 fix ip-group为空时会默认添加实际不匹配的规则
	var dstAddr string
	var dstIpGroupList []string
	if strings.TrimSpace(ipGroupName) == "" {
		log.Println("ip端口分流== ip-group 参数为空 ")
	} else {
		for ipGroupItem := range strings.SplitSeq(ipGroupName, ",") {
			var data []string
			data, err = iKuai.GetAllIKuaiBypassIpGroupNamesByName(ipGroupItem)
			if err != nil {
				return
			}
			dstIpGroupList = append(dstIpGroupList, data...)
		}
		// #101 fix ip-group为空时会默认添加
		if len(dstIpGroupList) == 0 {
			log.Println("ip端口分流== 未找到任何匹配的 IP 分组，跳过端口分流规则添加，配置的 ip-group:", ipGroupName)
			return nil
		} else {
			dstAddr = strings.Join(dstIpGroupList, ",")
		}
	}
	if strings.TrimSpace(srcAddrOptTag) != "" { // 优先使用 srcAddrOptTag #99
		var srcIpGroupList []string
		for srcIpGroupItem := range strings.SplitSeq(srcAddrOptTag, ",") {
			var data []string
			data, err = iKuai.GetAllIKuaiBypassIpGroupNamesByName(srcIpGroupItem)
			if err != nil {
				return
			}
			srcIpGroupList = append(srcIpGroupList, data...)
		}
		if len(srcIpGroupList) > 0 {
			srcAddr = strings.Join(srcIpGroupList, ",") // #99
		} else {
			log.Println("ip端口分流== 未找到任何匹配的 源地址 IP 分组，跳过端口分流规则添加，配置的 srcAddrOptTag:", srcAddrOptTag)
			return nil
		}
	}

	// 如果提供了预删除 ID，则在添加前清理
	if preDelIds != "" {
		err = iKuai.DelStreamIpPort(preDelIds)
		if err != nil {
			log.Println("ip端口分流== 清理旧规则失败，跳过此次更新:", err)
			return
		}
		log.Println("ip端口分流== 已清理旧的端口分流规则")
	}

	err = iKuai.AddStreamIpPort(forwardType, iface, dstAddr, srcAddr, nexthop, tag, mode, ifaceband)
	if err != nil {
		log.Println("ip端口分流==  添加失败，", config.GlobalConfig.AddErrRetryWait, "秒后重试  err:", err)
		time.Sleep(config.GlobalConfig.AddErrRetryWait)
	}
	return
}
