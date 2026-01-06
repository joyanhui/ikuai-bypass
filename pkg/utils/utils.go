package utils

import (
	"errors"
	"io"
	"log"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/dscao/ikuai-bypass/pkg/ikuai-api"
	"github.com/dscao/ikuai-bypass/pkg/config"
	"github.com/dscao/ikuai-bypass/pkg/ikuai-router"
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

// UpdateCustomIsp 更新运营商分流规则
func UpdateCustomIsp(iKuai *ikuaiapi.IKuai, name string, tag string, url string) (err error) {
	log.Println("运营商/IP分流==  http.get ...", url)
	resp, err := http.Get(GetFullUrl(url))
	if err != nil {
		return
	}
	if resp.StatusCode != 200 {
		err = errors.New(resp.Status)
		return
	}
	defer func(Body io.ReadCloser) {
		_ = Body.Close()
	}(resp.Body)
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return
	}
	ips := strings.Split(string(body), "\n")
	ips = RemoveIpv6AndRemoveEmptyLine(ips)
	log.Println("运营商/IP分流== ", name, tag, " 获取到", len(ips), "个ip")
	ipGroups := Group(ips, 5000) //5000条

	for _, ig := range ipGroups {
		ipGroup := strings.Join(ig, ",")
		err = iKuai.AddCustomIsp(name, tag, ipGroup)
		if err != nil {
			log.Println("运营商/IP分流==  ", name, tag, "添加失败，可能是列表太多了，添加太快,爱快没响应。", config.GlobalConfig.AddErrRetryWait, "秒后重试", err)
			time.Sleep(config.GlobalConfig.AddErrRetryWait)
			err = iKuai.AddCustomIsp(name, tag, ipGroup)
			if err != nil {
				log.Println("运营商/IP分流==  ", name, tag, "重试失败，可能是列表太多了，添加太快,爱快没响应。已经重试过一次，所以跳过此次操作")
				break
			}
		}
		log.Println("运营商/IP分流==  添加ip:", len(ig), " 个,等待", config.GlobalConfig.AddWait, "秒继续处理")
		time.Sleep(config.GlobalConfig.AddWait)
	}
	return
}

// UpdateStreamDomain 更新域名分流规则
func UpdateStreamDomain(iKuai *ikuaiapi.IKuai, iface, tag, srcAddr, url string) (err error) {
	log.Println("域名分流==  http.get ...", url)
	resp, err := http.Get(GetFullUrl(url))
	if err != nil {
		return
	}
	if resp.StatusCode != 200 {
		err = errors.New(resp.Status)
		return
	}
	defer func(Body io.ReadCloser) {
		_ = Body.Close()
	}(resp.Body)
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return
	}
	domains := strings.Split(string(body), "\n")
	log.Println("域名分流== ", iface, tag, "获取到", len(domains), "个域名")
	domainGroup := Group(domains, 1000) //1000条
	var countFor int = 0
	for _, d := range domainGroup {
		countFor = countFor + 1
		log.Println("域名分流== ", countFor, "/", len(domainGroup), iface, tag, " 正在添加 .... ")
		domain := strings.Join(d, ",")
		err = iKuai.AddStreamDomain(iface, tag, srcAddr, domain)
		if err != nil {
			log.Println("域名分流==  ", countFor, "/", len(domainGroup), iface, tag, "添加失败，可能是列表太多了，添加太快,爱快没响应。", config.GlobalConfig.AddErrRetryWait, "秒后重试", err)
			time.Sleep(config.GlobalConfig.AddErrRetryWait)
			err = iKuai.AddStreamDomain(iface, tag, srcAddr, domain)
			if err != nil {
				log.Println("域名分流=  ", countFor, "/", len(domainGroup), iface, tag, "重试失败，可能是列表太多了，添加太快,爱快没响应。已经重试过一次，所以跳过此次操作")
				break
			}
		} else {
			log.Println("域名分流== ", iface, tag, " 添加域名:", len(d), " 个成功,等待", config.GlobalConfig.AddWait, "秒继续处理")
			time.Sleep(config.GlobalConfig.AddWait)
		}
	}
	return
}

func RemoveIpv6AndRemoveEmptyLine(ips []string) []string {
	log.Println("移除ipv6地址 和删除空行....")
	i := 0
	for _, ip := range ips {
		if !strings.Contains(ip, ":") {
			//删除空行
			ip = strings.Trim(strings.Trim(ip, "\n"), "\r")
			if ip != "" {
				ips[i] = ip
				i++
			}

		}
	}
	return ips[:i]
}

func RemoveIpv4AndRemoveEmptyLine(ips []string) []string {
	log.Println("移除ipv4地址 和删除空行....")
	i := 0
	for _, ip := range ips {
		if strings.Contains(ip, ":") { // 检查IPv6地址特征
			// 清理首尾的空白字符和换行符
			ip = strings.TrimSpace(ip)
			if ip != "" {
				ips[i] = ip
				i++
			}
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

// UpdateIpGroup 更新ip分组
func UpdateIpGroup(iKuai *ikuaiapi.IKuai, name, url string) (err error) {
	log.Println("ip分组==  http.get ...", url)
	resp, err := http.Get(GetFullUrl(url))
	if err != nil {
		return
	}
	if resp.StatusCode != 200 {
		err = errors.New(resp.Status)
	}
	defer resp.Body.Close()
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return
	}
	ips := strings.Split(string(body), "\n")
	ips = RemoveIpv6AndRemoveEmptyLine(ips)
	ipGroups := Group(ips, 1000)
    log.Println("ip分组获取新数据成功", name)
	preIds, err := iKuai.GetIpGroup(name)
	if err != nil {
		log.Println("ip分组== 获取准备更新的IP分组列表失败：", name, err)
		return
	} else {
		log.Println("ip分组== 获取准备更新的IP分组列表成功", name, preIds)
	}
	err = iKuai.DelIpGroup(preIds)
	if err == nil {
		log.Println("ip分组== 删除旧的IP分组列表成功", name)
	} else {
		log.Println("ip分组== 删除旧的IP分组列表有错误", name, err)
		return
	}
	preIds = ""
	for index, ig := range ipGroups {
		log.Println("ip分组== ", index, " 正在添加 .... ")
		ipGroup := strings.Join(ig, ",")
		err := iKuai.AddIpGroup(name+"_"+strconv.Itoa(index), ipGroup)
		if err != nil {
			log.Println("ip分组== ", index, "添加失败，可能是列表太多了，添加太快,爱快没响应。", config.GlobalConfig.AddErrRetryWait, "秒后重试", err)
			time.Sleep(config.GlobalConfig.AddWait)
		}

	}
	return
}

// UpdateIpv6Group 更新ipv6分组
func UpdateIpv6Group(iKuai *ikuaiapi.IKuai, name, url string) (err error) {
	log.Println("ipv6分组==  http.get ...", url)
	resp, err := http.Get(GetFullUrl(url))
	if err != nil {
		return
	}
	if resp.StatusCode != 200 {
		err = errors.New(resp.Status)
	}
	defer resp.Body.Close()
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return
	}
	ips := strings.Split(string(body), "\n")
	ips = RemoveIpv4AndRemoveEmptyLine(ips)
	ipGroups := Group(ips, 1000)
	log.Println("ipv6分组获取新数据成功", name)
	preIds, err := iKuai.GetIpv6Group(name)
	if err != nil {
		log.Println("ipv6分组== 获取准备更新的IPv6分组列表失败：", name, err)
		return
	} else {
		log.Println("ipv6分组== 获取准备更新的IPv6分组列表成功", name, preIds)
	}
	err = iKuai.DelIpv6Group(preIds)
	if err == nil {
		log.Println("ipv6分组== 删除旧的IPv6分组列表成功", name, preIds)
	} else {
		log.Println("ipv6分组== 删除旧的IPv6分组列表有错误", name, err)
		return
	}
	preIds = ""
	for index, ig := range ipGroups {
		log.Println("ipv6分组== ", index, " 正在添加 .... ")
		ipGroup := strings.Join(ig, ",")
		err := iKuai.AddIpv6Group(name+"_"+strconv.Itoa(index), ipGroup)
		if err != nil {
			log.Println("ipv6分组== ", index, "添加失败，可能是列表太多了，添加太快,爱快没响应。", config.GlobalConfig.AddErrRetryWait, "秒后重试", err)
			time.Sleep(config.GlobalConfig.AddWait)
		}
	}
	return
}

// UpdateStreamIpPort 更新ip端口分流
func UpdateStreamIpPort(iKuai *ikuaiapi.IKuai, forwardType string, iface string, nexthop string, srcAddr string, ipGroup string, mode int, ifaceband int) (err error) {

	var ipGroupList []string
	for _, ipGroupItem := range strings.Split(ipGroup, ",") {
		var data []string
		data, err = iKuai.GetAllIKuaiBypassIpGroupNamesByName(ipGroupItem)
		if err != nil {
			return
		}
		ipGroupList = append(ipGroupList, data...)
	}
	err = iKuai.AddStreamIpPort(forwardType, iface, strings.Join(ipGroupList, ","), srcAddr, nexthop, ipGroup, mode, ifaceband)
	if err != nil {
		log.Println("ip端口分流==  添加失败，可能是列表太多了，添加太快,爱快没响应。", config.GlobalConfig.AddErrRetryWait, "秒后重试", err)
		time.Sleep(config.GlobalConfig.AddErrRetryWait)
	}
	return
}

// LoginToIkuai 登陆爱快
func LoginToIkuai() (*ikuaiapi.IKuai, error) {
	err := config.Read(*config.ConfPath)
	if err != nil {
		log.Println("读取配置文件失败：", err)
		return nil, err
	}
	if *config.IkuaiLoginInfo != "" {
		log.Println("使用命令行参数登陆爱快")
		ikuaiLoginInfoArr := strings.Split(*config.IkuaiLoginInfo, ",")
		if len(ikuaiLoginInfoArr) != 3 {
			log.Println(*config.IkuaiLoginInfo)
			log.Println("命令行参数格式错误，请使用 -login http://ip,username,password ")
			return nil, errors.New("命令行参数格式错误，请使用 -login=\"ip,username,password\"")
		}
		iKuai := ikuaiapi.NewIKuai(ikuaiLoginInfoArr[0])
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
			gateway, err := ikuairouter.GetGateway()
			if err != nil {
				log.Println("获取默认网关失败：", err)
				return nil, err
			}
			baseurl = "http://" + gateway
			log.Println("使用默认网关地址：", baseurl)
		}
		iKuai := ikuaiapi.NewIKuai(baseurl)
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
