package main

import (
	"errors"
	"io"
	"log"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/dscao/ikuai-bypass/api"
	"github.com/dscao/ikuai-bypass/router"
)

// 读取配置文件 到 conf

// updateCustomIsp 更新运营商分流规则
func updateCustomIsp(iKuai *api.IKuai, name string, tag string, url string) (err error) {
	log.Println("运营商/IP分流==  http.get ...", url)
	resp, err := http.Get(url)
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
	ips = removeIpv6AndRemoveEmptyLine(ips)
	log.Println("运营商/IP分流== ", name, tag, " 获取到", len(ips), "个ip")
	ipGroups := group(ips, 5000) //5000条

	for _, ig := range ipGroups {
		ipGroup := strings.Join(ig, ",")
		err = iKuai.AddCustomIsp(name, tag, ipGroup)
		if err != nil {
			log.Println("运营商/IP分流==  ", name, tag, "添加失败，可能是列表太多了，添加太快,爱快没响应。", conf.AddErrRetryWait, "秒后重试", err)
			time.Sleep(conf.AddErrRetryWait)
			err = iKuai.AddCustomIsp(name, tag, ipGroup)
			if err != nil {
				log.Println("运营商/IP分流==  ", name, tag, "重试失败，可能是列表太多了，添加太快,爱快没响应。已经重试过一次，所以跳过此次操作")
				break
			}
		}
		log.Println("运营商/IP分流==  添加ip:", len(ig), " 个,等待", conf.AddWait, "秒继续处理")
		time.Sleep(conf.AddWait)
	}
	return
}

// updateStreamDomain 更新域名分流规则
func updateStreamDomain(iKuai *api.IKuai, iface, tag, srcAddr, url string) (err error) {
	log.Println("域名分流==  http.get ...", url)
	resp, err := http.Get(url)
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
	domainGroup := group(domains, 1000) //1000条
	var countFor int = 0
	for _, d := range domainGroup {
		countFor = countFor + 1
		log.Println("域名分流== ", countFor, "/", len(domainGroup), iface, tag, " 正在添加 .... ")
		domain := strings.Join(d, ",")
		err = iKuai.AddStreamDomain(iface, tag, srcAddr, domain)
		if err != nil {
			log.Println("域名分流==  ", countFor, "/", len(domainGroup), iface, tag, "添加失败，可能是列表太多了，添加太快,爱快没响应。", conf.AddErrRetryWait, "秒后重试", err)
			time.Sleep(conf.AddErrRetryWait)
			err = iKuai.AddStreamDomain(iface, tag, srcAddr, domain)
			if err != nil {
				log.Println("域名分流=  ", countFor, "/", len(domainGroup), iface, tag, "重试失败，可能是列表太多了，添加太快,爱快没响应。已经重试过一次，所以跳过此次操作")
				break
			}
		} else {
			log.Println("域名分流== ", iface, tag, " 添加域名:", len(d), " 个成功,等待", conf.AddWait, "秒继续处理")
			time.Sleep(conf.AddWait)
		}
	}
	return
}

func removeIpv6AndRemoveEmptyLine(ips []string) []string {
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

func removeIpv4AndRemoveEmptyLine(ips []string) []string {
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

func group(arr []string, subGroupLength int64) [][]string {
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

// 更新ip分组
func updateIpGroup(iKuai *api.IKuai, name, url string) (err error) {
	log.Println("ip分组==  http.get ...", url)
	resp, err := http.Get(url)
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
	ips = removeIpv6AndRemoveEmptyLine(ips)
	ipGroups := group(ips, 1000)
	last4 := ""
	if *isIpGroupNameAddRandomSuff == "1" { //https://github.com/joyanhui/ikuai-bypass/issues/76
		timestamp := time.Now().Unix() // 秒级时间戳（10位，如 1620000000）
		str := strconv.FormatInt(timestamp, 10)
		last4 = "_" + str[len(str)-4:] // 截取字符串最后4位，防止分组名重复导致无法先增加后删除，分组名称最多20个字符，除去 _0_1234 分组名设置中最多13个。
	}

	for index, ig := range ipGroups {
		log.Println("ip分组== ", index, " 正在添加 .... ")
		ipGroup := strings.Join(ig, ",")
		err := iKuai.AddIpGroup(name+"_"+strconv.Itoa(index)+last4, ipGroup)
		if err != nil {
			log.Println("ip分组== ", index, "添加失败，可能是列表太多了，添加太快,爱快没响应。", conf.AddErrRetryWait, "秒后重试", err)
			time.Sleep(conf.AddWait)
		}

	}
	return
}

// 更新ipv6分组
func updateIpv6Group(iKuai *api.IKuai, name, url string) (err error) {
	log.Println("ipv6分组==  http.get ...", url)
	resp, err := http.Get(url)
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
	ips = removeIpv4AndRemoveEmptyLine(ips)
	ipGroups := group(ips, 1000)
	last4 := ""
	if *isIpGroupNameAddRandomSuff == "1" { //https://github.com/joyanhui/ikuai-bypass/issues/76
		timestamp := time.Now().Unix() // 秒级时间戳（10位，如 1620000000）
		str := strconv.FormatInt(timestamp, 10)
		last4 = "_" + str[len(str)-4:] // 截取字符串最后4位，防止分组名重复导致无法先增加后删除，分组名称最多20个字符，除去 _0_1234 分组名设置中最多13个。
	}
	for index, ig := range ipGroups {
		log.Println("ipv6分组== ", index, " 正在添加 .... ")
		ipGroup := strings.Join(ig, ",")
		err := iKuai.AddIpv6Group(name+"_"+strconv.Itoa(index)+last4, ipGroup)
		if err != nil {
			log.Println("ipv6分组== ", index, "添加失败，可能是列表太多了，添加太快,爱快没响应。", conf.AddErrRetryWait, "秒后重试", err)
			time.Sleep(conf.AddWait)
		}
	}
	return
}

// 更新ip端口分流
func updateStreamIpPort(iKuai *api.IKuai, forwardType string, iface string, nexthop string, srcAddr string, ipGroup string, mode int, ifaceband int) (err error) {

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
		log.Println("ip端口分流==  添加失败，可能是列表太多了，添加太快,爱快没响应。", conf.AddErrRetryWait, "秒后重试", err)
		time.Sleep(conf.AddErrRetryWait)
	}
	return
}

// 登陆爱快
func loginToIkuai() (*api.IKuai, error) {
	err := readConf(*confPath)
	if err != nil {
		log.Println("读取配置文件失败：", err)
		return nil, err
	}
	if *ikuaiLoginInfo != "" {
		log.Println("使用命令行参数登陆爱快")
		ikuaiLoginInfoArr := strings.Split(*ikuaiLoginInfo, ",")
		if len(ikuaiLoginInfoArr) != 3 {
			log.Println(*ikuaiLoginInfo)
			log.Println("命令行参数格式错误，请使用 -login http://ip|username|password ")
			return nil, errors.New("命令行参数格式错误，请使用 -login=\"ip|username|password\"")
		}
		iKuai := api.NewIKuai(ikuaiLoginInfoArr[0])
		err = iKuai.Login(ikuaiLoginInfoArr[1], ikuaiLoginInfoArr[2])
		if err != nil {
			log.Println("ikuai 登陆失败：", *ikuaiLoginInfo, err)
			return nil, err
		} else {
			log.Println("ikuai 登录成功", ikuaiLoginInfoArr[0])
			return iKuai, nil
		}
	} else {
		baseurl := conf.IkuaiURL
		if baseurl == "" {
			gateway, err := router.GetGateway()
			if err != nil {
				log.Println("获取默认网关失败：", err)
				return nil, err
			}
			baseurl = "http://" + gateway
			log.Println("使用默认网关地址：", baseurl)
		}
		iKuai := api.NewIKuai(baseurl)
		err = iKuai.Login(conf.Username, conf.Password)
		if err != nil {
			log.Println("ikuai 登陆失败：", baseurl, err)
			return iKuai, err
		} else {
			log.Println("ikuai 登录成功", baseurl)
			return iKuai, nil
		}
	}
}
