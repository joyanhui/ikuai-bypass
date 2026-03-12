package core

import (
	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/logger"
	"ikuai-bypass/pkg/utils"
)

func MainUpdateIspRule() {
	iKuai, err := utils.LoginToIkuai()
	if err != nil {
		utils.SysLog.Error("LOGIN:登录失败", "Failed to login to iKuai: %v", err)
		return
	}

	ispLogger := logger.NewLogger("ISP:运营商分流")
	domainLogger := logger.NewLogger("DOMAIN:域名分流")

	for _, customIsp := range config.GlobalConfig.CustomIsp {
		ispLogger.Info("UPDATE:开始更新", "Updating %s...", customIsp.Tag)
		err = utils.UpdateCustomIsp(ispLogger, iKuai, customIsp.Tag, customIsp.URL)
		if err != nil {
			ispLogger.Error("UPDATE:更新失败", "Failed to update custom ISP '%s': %v", customIsp.Tag, err)
			// Continue to next ISP rule even if one fails
		} else {
			ispLogger.Success("UPDATE:更新成功", "Successfully updated custom ISP '%s'", customIsp.Tag)
		}
	}

	for _, streamDomain := range config.GlobalConfig.StreamDomain {
		domainLogger.Info("UPDATE:开始更新", "Updating %s (Interface: %s, Tag: %s)...", streamDomain.URL, streamDomain.Interface, streamDomain.Tag)
		err = utils.UpdateStreamDomain(domainLogger, iKuai, streamDomain.Interface, streamDomain.Tag, streamDomain.SrcAddrOptIpGroup, streamDomain.SrcAddr, streamDomain.URL)
		if err != nil {
			domainLogger.Error("UPDATE:更新失败", "Failed to update domain streaming for tag %s: %v", streamDomain.Tag, err)
		} else {
			domainLogger.Success("UPDATE:更新成功", "Successfully updated domain streaming for tag %s", streamDomain.Tag)
		}
	}

	utils.SysLog.Success("DONE:任务完成", "ISP and Domain streaming update tasks completed")
}
