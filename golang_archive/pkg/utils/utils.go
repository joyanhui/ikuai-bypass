package utils
import (
	"errors"
	"strings"

	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/ikuai_api4"
	"ikuai-bypass/pkg/ikuai_common"
	"ikuai-bypass/pkg/ikuai_router"
	"ikuai-bypass/pkg/logger"
)
var SysLog = logger.NewLogger("SYS:系统组件") // System Component Logger

func RemoveIpv6AndRemoveEmptyLine(l *logger.Logger, ips []string) []string {
	l.Info("IP:v6规则清洗", "Removing IPv6 addresses, empty lines and comments...")
	i := 0
	for _, ip := range ips {
		// 处理注释：删除 # 及其后的所有内容
		// Handle comments: remove everything after #
		if idx := strings.Index(ip, "#"); idx != -1 {
			ip = ip[:idx]
		}
		ip = strings.TrimSpace(ip)
		if ip == "" {
			continue
		}

		if !strings.Contains(ip, ":") {
			ips[i] = ip
			i++
		}
	}
	return ips[:i]
}

func RemoveIpv4AndRemoveEmptyLine(l *logger.Logger, ips []string) []string {
	l.Info("IP:v4规则清洗", "Removing IPv4 addresses, empty lines and comments...")
	i := 0
	for _, ip := range ips {
		// 处理注释：删除 # 及其后的所有内容
		// Handle comments: remove everything after #
		if idx := strings.Index(ip, "#"); idx != -1 {
			ip = ip[:idx]
		}
		ip = strings.TrimSpace(ip)
		if ip == "" {
			continue
		}

		if strings.Contains(ip, ":") { // 检查IPv6地址特征
			ips[i] = ip
			i++
		}
	}
	return ips[:i]
}

func Group(arr []string, subGroupLength int64) [][]string {
	groupMax := int64(len(arr))
	var segmens = make([][]string, 0)
	quantity := groupMax / subGroupLength
	remainder := groupMax % subGroupLength
	i := int64(0)
	for i = int64(0); i < quantity; i++ {
		segmens = append(segmens, arr[i*subGroupLength:(i+1)*subGroupLength])
	}
	if quantity == 0 || remainder != 0 {
		segmens = append(segmens, arr[i*subGroupLength:i*subGroupLength+remainder])
	}
	return segmens
}

func FilterDomains(l *logger.Logger, domains []string) []string {
	l.Info("DOMAIN:域名清洗", "Cleaning invalid domains (underscores, comments, etc.)...")
	i := 0
	for _, d := range domains {
		d = strings.TrimSpace(d)
		if d == "" {
			continue
		}

		// 处理注释：删除 # 及其后的所有内容
		// Handle comments: remove everything after #
		if idx := strings.Index(d, "#"); idx != -1 {
			d = strings.TrimSpace(d[:idx])
		}

		// 再次检查清理后的内容是否为空
		if d == "" {
			continue
		}

		// iKuai 不支持包含下划线的域名，这会导致 4.0 API 返回 "请求参数不合法"
		// iKuai does not support domains with underscores, which causes "Invalid parameters" in 4.0 API
		if strings.Contains(d, "_") {
			l.Log("DOMAIN:域名清洗", "Excluding invalid domain (contains underscore): %s", d)
			continue
		}
		domains[i] = d
		i++
	}
	return domains[:i]
}

// LoginToIkuai 登陆爱快
func LoginToIkuai() (ikuai_common.IKuaiClient, error) {
	err := config.Read(*config.ConfPath)
	if err != nil {
		SysLog.Error("CONF:配置读取", "Failed to read configuration file: %v", err)
		return nil, err
	}

	var iKuai ikuai_common.IKuaiClient

	if *config.IkuaiLoginInfo != "" {
		SysLog.Info("AUTH:登录认证", "Logging in using command line parameters")
		ikuaiLoginInfoArr := strings.Split(*config.IkuaiLoginInfo, ",")
		if len(ikuaiLoginInfoArr) != 3 {
			SysLog.Log("ARGS:参数错误", "Login info provided: %s", *config.IkuaiLoginInfo)
			SysLog.Error("AUTH:登录认证", "Command line parameter format error, please use -login http://ip,username,password")
			return nil, errors.New("command line parameter format error")
		}

		iKuai = ikuai_api4.NewIKuai(ikuaiLoginInfoArr[0])

		err = iKuai.Login(ikuaiLoginInfoArr[1], ikuaiLoginInfoArr[2])
		if err != nil {
			SysLog.Error("AUTH:登录认证", "Login failed: %s, error: %v", *config.IkuaiLoginInfo, err)
			return nil, err
		} else {
			SysLog.Success("AUTH:登录认证", "Login successful: %s", ikuaiLoginInfoArr[0])
			return iKuai, nil
		}
	} else {
		baseurl := config.GlobalConfig.IkuaiURL
		if baseurl == "" {
			gateway, err := ikuai_router.GetGateway()
			if err != nil {
				SysLog.Error("SYS:网关检测", "Failed to get default gateway: %v", err)
				return nil, err
			}
			baseurl = "http://" + gateway
			SysLog.Info("SYS:网关检测", "Using default gateway address: %s", baseurl)
		}

		iKuai = ikuai_api4.NewIKuai(baseurl)

		err = iKuai.Login(config.GlobalConfig.Username, config.GlobalConfig.Password)
		if err != nil {
			SysLog.Error("AUTH:登录认证", "Login failed: %s, error: %v", baseurl, err)
			return iKuai, err
		} else {
			SysLog.Success("AUTH:登录认证", "Login successful: %s", baseurl)
			return iKuai, nil
		}
	}
}
