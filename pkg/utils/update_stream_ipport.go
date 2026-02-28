package utils

import (
	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/ikuai_common"
	"ikuai-bypass/pkg/logger"
	"strings"
	"time"
)

// UpdateStreamIpPort 更新ip端口分流
func UpdateStreamIpPort(logger *logger.Logger, iKuai ikuai_common.IKuaiClient, forwardType string, tag string, iface string, nexthop string, srcAddr string, srcAddrOptIpGroup string, ipGroupName string, mode int, ifaceband int) (err error) {
	var dstAddr string
	var dstIpGroupList []string
	if strings.TrimSpace(ipGroupName) == "" {
		logger.Info("CHECK:参数校验", "ip-group parameter is empty")
	} else {
		for ipGroupItem := range strings.SplitSeq(ipGroupName, ",") {
			var data []string
			data, err = iKuai.GetAllIKuaiBypassIpGroupNamesByName(ipGroupItem)
			if err != nil {
				return err
			}
			dstIpGroupList = append(dstIpGroupList, data...)
		}
		if len(dstIpGroupList) == 0 {
			logger.Info("SKIP:跳过操作", "No matching destination IP groups found, skipping port streaming rule addition. ip-group: %s", ipGroupName)
			return nil
		} else {
			dstAddr = strings.Join(dstIpGroupList, ",")
		}
	}
	if strings.TrimSpace(srcAddrOptIpGroup) != "" {
		var srcIpGroupList []string
		for srcIpGroupItem := range strings.SplitSeq(srcAddrOptIpGroup, ",") {
			var data []string
			data, err = iKuai.GetAllIKuaiBypassIpGroupNamesByName(srcIpGroupItem)
			if err != nil {
				return err
			}
			srcIpGroupList = append(srcIpGroupList, data...)
		}
		if len(srcIpGroupList) > 0 {
			srcAddr = strings.Join(srcIpGroupList, ",")
		} else {
			logger.Info("SKIP:跳过操作", "No matching source IP groups found, skipping port streaming rule addition. srcAddrOptIpGroup: %s", srcAddrOptIpGroup)
			return nil
		}
	}

	streamMap, err := iKuai.GetStreamIpPortMap(tag)
	if err != nil {
		logger.Error("QUERY:查询规则", "Failed to get existing port streaming rules: %v", err)
		return err
	}
	var foundId int
	var foundName string
	for name, id := range streamMap {
		// In iKuai API, port streaming TagName is what we care about.
		// streamMap is map[string]int where key is TagName.
		// We search for a TagName that matches our current tag.
		// Since Port streaming usually has only one rule per config item, we just pick the first match.
		foundId = id
		foundName = name
		break
	}

	if foundId > 0 {
		logger.Info("EDIT:正在修改", "[1/1] %s: updating existing rule %s (ID: %d)...", tag, foundName, foundId)
		err = iKuai.EditStreamIpPort(forwardType, iface, dstAddr, srcAddr, nexthop, tag, mode, ifaceband, foundId)
	} else {
		logger.Info("ADD:正在添加", "[1/1] %s: adding new rule...", tag)
		err = iKuai.AddStreamIpPort(forwardType, iface, dstAddr, srcAddr, nexthop, tag, mode, ifaceband)
	}

	if err != nil {
		logger.Error("UPDATE:更新失败", "[1/1] %s: failed: %v", tag, err)
		time.Sleep(config.GlobalConfig.AddErrRetryWait)
	} else {
		logger.Success("UPDATE:更新成功", "[1/1] %s: updated successfully", tag)
	}
	return nil
}
