package core

import (
	"ikuai-bypass/pkg/utils"
)

// RunUpdateByModule 按模块执行更新任务 / Run update tasks by module.
func RunUpdateByModule(module string) {
	switch module {
	case "ispdomain":
		utils.SysLog.Info("TASK:任务启动", "Starting ISP and Domain streaming mode")
		MainUpdateIspRule()
	case "ipgroup":
		utils.SysLog.Info("TASK:任务启动", "Starting IP group and Next-hop gateway mode")
		MainUpdateIpgroup()
	case "ipv6group":
		utils.SysLog.Info("TASK:任务启动", "Starting IPv6 group mode")
		MainUpdateIpv6group()
	case "ii":
		utils.SysLog.Info("TASK:任务启动", "Starting hybrid mode: ISP/Domain + IP group")
		MainUpdateIspRule()
		MainUpdateIpgroup()
	case "ip":
		utils.SysLog.Info("TASK:任务启动", "Starting hybrid mode: IPv4 group + IPv6 group")
		MainUpdateIpgroup()
		MainUpdateIpv6group()
	case "iip":
		utils.SysLog.Info("TASK:任务启动", "Starting full hybrid mode: ISP/Domain + IPv4/v6 group")
		MainUpdateIspRule()
		MainUpdateIpgroup()
		MainUpdateIpv6group()
	default:
		utils.SysLog.Error("ERR:参数错误", "Invalid -m parameter: %s", module)
	}
}
