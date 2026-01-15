package utils

import (
	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/ikuai_api"
	"log"
	"strings"
	"time"
)

// UpdateStreamIpPort 更新ip端口分流
func UpdateStreamIpPort(iKuai *ikuai_api.IKuai, forwardType string, tag string, iface string, nexthop string, srcAddr string, srcAddrOptTag string, ipGroupName string, mode int, ifaceband int) (err error) {

	// #101 fix ip-group为空时会默认添加实际不匹配的规则
	var ipGroupList []string
	if strings.TrimSpace(ipGroupName) == "" {
		log.Println("ip端口分流== ip-group 参数为空 ")
	} else {
		for ipGroupItem := range strings.SplitSeq(ipGroupName, ",") {
			var data []string
			data, err = iKuai.GetAllIKuaiBypassIpGroupNamesByName(ipGroupItem)
			if err != nil {
				return
			}
			ipGroupList = append(ipGroupList, data...)
		}
		// #101 fix ip-group为空时会默认添加
		if len(ipGroupList) == 0 {
			log.Println("ip端口分流== 未找到任何匹配的 IP 分组，跳过端口分流规则添加，配置的 ip-group:", ipGroupName)
			return nil
		}
	}

	err = iKuai.AddStreamIpPort(forwardType, iface, strings.Join(ipGroupList, ","), srcAddr, nexthop, tag, mode, ifaceband)
	if err != nil {
		log.Println("ip端口分流==  添加失败，可能是列表太多了，添加太快,爱快没响应。", config.GlobalConfig.AddErrRetryWait, "秒后重试", err)
		time.Sleep(config.GlobalConfig.AddErrRetryWait)
	}
	return
}
