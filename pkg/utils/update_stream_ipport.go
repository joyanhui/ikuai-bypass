package utils

import (
	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/ikuai_common"
	"ikuai-bypass/pkg/logger"
	"strings"
	"time"
)

// UpdateStreamIpPort 更新ip端口分流
func UpdateStreamIpPort(logger *logger.Logger, iKuai ikuai_common.IKuaiClient, forwardType string, tag string, iface string, nexthop string, srcAddr string, srcAddrOptIpGroup string, ipGroupName string, mode int, ifaceband int, preDelIds string) (err error) {

	// #101 fix ip-group为空时会默认添加实际不匹配的规则
	var dstAddr string
	var dstIpGroupList []string
	if strings.TrimSpace(ipGroupName) == "" {
		logger.Info("CHECK:参数校验", "ip-group parameter is empty")
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
			logger.Info("SKIP:跳过操作", "No matching destination IP groups found, skipping port streaming rule addition. ip-group: %s", ipGroupName)
			return nil
		} else {
			dstAddr = strings.Join(dstIpGroupList, ",")
		}
	}
	if strings.TrimSpace(srcAddrOptIpGroup) != "" { // 优先使用 srcAddrOptIpGroup #99
		var srcIpGroupList []string
		for srcIpGroupItem := range strings.SplitSeq(srcAddrOptIpGroup, ",") {
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
			logger.Info("SKIP:跳过操作", "No matching source IP groups found, skipping port streaming rule addition. srcAddrOptIpGroup: %s", srcAddrOptIpGroup)
			return nil
		}
	}

	// 如果提供了预删除 ID，则在添加前清理
	if preDelIds != "" {
		count := len(strings.Split(preDelIds, ","))
		err = iKuai.DelStreamIpPort(preDelIds)
		if err != nil {
			logger.Error("CLEAN:清理旧规", "Failed to clear old rules, skipping update: %v", err)
			return
		}
		logger.Success("CLEAN:清理旧规", "Cleared %d old port streaming rules", count)
	}

	err = iKuai.AddStreamIpPort(forwardType, iface, dstAddr, srcAddr, nexthop, tag, mode, ifaceband)
	if err != nil {
		logger.Error("ADD:添加失败", "Failed to add port streaming rule, retrying after %v seconds. error: %v", config.GlobalConfig.AddErrRetryWait, err)
		time.Sleep(config.GlobalConfig.AddErrRetryWait)
	} else {
		logger.Success("ADD:添加成功", "Port streaming rule added successfully: %s", tag)
	}
	return
}
