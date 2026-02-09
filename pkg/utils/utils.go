package utils

import (
	"errors"
	"log"
	"strings"

	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/ikuai_api3"
	"ikuai-bypass/pkg/ikuai_api4"
	"ikuai-bypass/pkg/ikuai_common"
	"ikuai-bypass/pkg/ikuai_router"
)

// GetFullUrl 根据配置的 GithubProxy 转换 URL
func GetFullUrl(originalURL string) string {
	proxy := config.GlobalConfig.GithubProxy
	// 如果代理配置为空，或者原始 URL 不是以 raw.githubusercontent.com 开头，直接返回原始 URL
	if proxy == "" || !strings.HasPrefix(originalURL, "https://raw.githubusercontent.com/") {
		return originalURL
	}

	// 确保代理地址以 / 结尾
	if !strings.HasSuffix(proxy, "/") {
		proxy += "/"
	}

	return proxy + originalURL
}

func RemoveIpv6AndRemoveEmptyLine(ips []string) []string {
	log.Println("移除ipv6地址 和删除空行、注释....")
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

func RemoveIpv4AndRemoveEmptyLine(ips []string) []string {
	log.Println("移除ipv4地址 和删除空行、注释....")
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

func FilterDomains(domains []string) []string {
	log.Println("域名分流== 清理无效域名 (如包含下划线、注释等)...")
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
			log.Printf("域名分流== 排除无效域名 (包含下划线): %s\n", d)
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
		log.Println("读取配置文件失败：", err)
		return nil, err
	}

	version := "3"
	if config.GlobalConfig.IkuaiVersion != "" {
		version = config.GlobalConfig.IkuaiVersion
	}

	var iKuai ikuai_common.IKuaiClient

	if *config.IkuaiLoginInfo != "" {
		log.Println("使用命令行参数登陆爱快")
		ikuaiLoginInfoArr := strings.Split(*config.IkuaiLoginInfo, ",")
		if len(ikuaiLoginInfoArr) != 3 {
			log.Println(*config.IkuaiLoginInfo)
			log.Println("命令行参数格式错误，请使用 -login http://ip,username,password ")
			return nil, errors.New("命令行参数格式错误，请使用 -login=\"ip,username,password\"")
		}

		if version == "4" {
			iKuai = ikuai_api4.NewIKuai(ikuaiLoginInfoArr[0])
		} else {
			iKuai = ikuai_api3.NewIKuai(ikuaiLoginInfoArr[0])
		}

		err = iKuai.Login(ikuaiLoginInfoArr[1], ikuaiLoginInfoArr[2])
		if err != nil {
			log.Println("ikuai 登陆失败：", *config.IkuaiLoginInfo, err)
			return nil, err
		} else {
			log.Println("ikuai 登录成功", ikuaiLoginInfoArr[0])
			return iKuai, nil
		}
	} else {
		baseurl := config.GlobalConfig.IkuaiURL
		if baseurl == "" {
			gateway, err := ikuai_router.GetGateway()
			if err != nil {
				log.Println("获取默认网关失败：", err)
				return nil, err
			}
			baseurl = "http://" + gateway
			log.Println("使用默认网关地址：", baseurl)
		}

		if version == "4" {
			iKuai = ikuai_api4.NewIKuai(baseurl)
		} else {
			iKuai = ikuai_api3.NewIKuai(baseurl)
		}

		err = iKuai.Login(config.GlobalConfig.Username, config.GlobalConfig.Password)
		if err != nil {
			log.Println("ikuai 登陆失败：", baseurl, err)
			return iKuai, err
		} else {
			log.Println("ikuai 登录成功", baseurl)
			return iKuai, nil
		}
	}
}