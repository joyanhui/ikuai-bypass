package core

import (
	"time"

	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/logger"
	"ikuai-bypass/pkg/utils"
)

func MainUpdateIspRule() {
	iKuai, err := utils.LoginToIkuai()
	if err != nil {
		utils.SysLog.Error("登录失败", "Failed to login to iKuai: %v", err)
		return
	}

	ispLogger := logger.NewLogger("运营商分流")
	domainLogger := logger.NewLogger("域名分流")

	var GoroutineEnd1 bool = false
	var GoroutineEnd2 bool = false
	go func() {
		for _, customIsp := range config.GlobalConfig.CustomIsp {
			//记录旧的自定义运营商
			preIds, err := iKuai.GetCustomIspAll(customIsp.Tag)
			if err != nil {
				ispLogger.Error("查询列表", "Failed to get old custom ISP list for %s (%s): %v", customIsp.Name, customIsp.Tag, err)
				break
			} else {
				ispLogger.Info("查询成功", "Obtained old custom ISP list for %s (%s)", customIsp.Name, customIsp.Tag)
			}

			// 强制执行 "Safe-Before" 模式：先成功获取远程数据，再清理旧规则，后添加新分片
			ispLogger.Info("开始更新", "Updating %s (%s)...", customIsp.Name, customIsp.Tag)
			err = utils.UpdateCustomIsp(ispLogger, iKuai, customIsp.Name, customIsp.Tag, customIsp.URL, preIds)
			if err != nil {
				ispLogger.Error("更新失败", "Failed to update custom ISP '%s': %v", customIsp.Name, err)
			} else {
				ispLogger.Success("更新成功", "Successfully updated custom ISP '%s'", customIsp.Name)
			}
		}
		GoroutineEnd1 = true
	}()

	go func() {
		for _, streamDomain := range config.GlobalConfig.StreamDomain {
			//记录旧的域名分流
			preIds, err := iKuai.GetStreamDomainAll(streamDomain.Tag)
			if err != nil {
				domainLogger.Error("查询列表", "Failed to get old domain list for tag %s: %v", streamDomain.Tag, err)
				break
			} else {
				domainLogger.Info("查询成功", "Obtained old domain list for tag %s", streamDomain.Tag)
			}

			//更新域名分流 (强制 Safe-Before)
			domainLogger.Info("开始更新", "Updating %s (Interface: %s, Tag: %s)...", streamDomain.URL, streamDomain.Interface, streamDomain.Tag)
			err = utils.UpdateStreamDomain(domainLogger, iKuai, streamDomain.Interface, streamDomain.Tag, streamDomain.SrcAddrOptIpGroup, streamDomain.SrcAddr, streamDomain.URL, preIds)
			if err != nil {
				domainLogger.Error("更新失败", "Failed to update domain streaming '%s': %v", streamDomain.Interface, err)
			} else {
				domainLogger.Success("更新成功", "Successfully updated domain streaming '%s'", streamDomain.Interface)
			}
		}
		GoroutineEnd2 = true
	}()

	for { //等待两个协程结束
		if GoroutineEnd1 && GoroutineEnd2 {
			break
		}
		time.Sleep(1 * time.Second)
	}
	utils.SysLog.Success("任务完成", "ISP and Domain streaming update tasks completed")
}
