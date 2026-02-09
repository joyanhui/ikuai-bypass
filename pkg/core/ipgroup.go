package core

import (
	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/logger"
	"ikuai-bypass/pkg/utils"
)

func MainUpdateIpgroup() {
	iKuai, err := utils.LoginToIkuai()
	if err != nil {
		utils.SysLog.Error("登录失败", "Failed to login to iKuai: %v", err)
		return
	}

	ipLogger := logger.NewLogger("IP分组")
	streamLogger := logger.NewLogger("端口分流")

	for _, ipGroup := range config.GlobalConfig.IpGroup {
		err := utils.UpdateIpGroup(ipLogger, iKuai, ipGroup.Name, ipGroup.URL)
		if err != nil {
			ipLogger.Error("更新失败", "Failed to add IP group '%s@%s': %v", ipGroup.Name, ipGroup.URL, err)
		} else {
			ipLogger.Success("更新成功", "Successfully updated IP group '%s@%s'", ipGroup.Name, ipGroup.URL)
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
			streamLogger.Error("参数校验", "Rule name and IpGroup cannot both be empty, skipping: %+v", streamIpPort)
			continue
		}

		preIds, err := iKuai.GetStreamIpPortIdsByTag(tag)
		if err != nil {
			streamLogger.Error("查询列表", "Failed to get port streaming list for %s: %v", tag, err)
			continue
		}

		// 强制执行 Safe-Before 模式
		err = utils.UpdateStreamIpPort(
			streamLogger,
			iKuai,
			streamIpPort.Type, tag,
			streamIpPort.Interface,
			streamIpPort.Nexthop,
			streamIpPort.SrcAddr, streamIpPort.SrcAddrOptIpGroup,
			streamIpPort.IpGroup,
			streamIpPort.Mode,
			streamIpPort.IfaceBand,
			preIds,
		)

		if err != nil {
			streamLogger.Error("更新失败", "Failed to update port streaming '%s@%s': %v",
				streamIpPort.Interface+streamIpPort.Nexthop,
				streamIpPort.IpGroup,
				err,
			)
		} else {
			streamLogger.Success("更新成功", "Successfully updated port streaming '%s@%s'",
				streamIpPort.Interface+streamIpPort.Nexthop,
				streamIpPort.IpGroup,
			)
		}
	}
}

func MainUpdateIpv6group() {
	iKuai, err := utils.LoginToIkuai()
	if err != nil {
		utils.SysLog.Error("登录失败", "Failed to login to iKuai: %v", err)
		return
	}
	ipv6Logger := logger.NewLogger("IPv6分组")
	for _, ipv6Group := range config.GlobalConfig.Ipv6Group {
		err := utils.UpdateIpv6Group(ipv6Logger, iKuai, ipv6Group.Name, ipv6Group.URL)
		if err != nil {
			ipv6Logger.Error("更新失败", "Failed to add IPv6 group '%s@%s': %v", ipv6Group.Name, ipv6Group.URL, err)
		} else {
			ipv6Logger.Success("更新成功", "Successfully updated IPv6 group '%s@%s'", ipv6Group.Name, ipv6Group.URL)
		}
	}
}
