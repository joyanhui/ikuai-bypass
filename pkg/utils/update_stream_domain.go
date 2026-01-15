package utils

import (
	"errors"
	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/ikuai_api"
	"io"
	"log"
	"net/http"
	"strings"
	"time"
)

// UpdateStreamDomain 更新域名分流规则
func UpdateStreamDomain(iKuai *ikuai_api.IKuai, iface, tag, srcAddrIpGroup, srcAddr, url string) (err error) {
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
	// #99 fix srcAddr 优先使用 srcAddrIpGroup
	var srcAddrList []string
	if strings.TrimSpace(srcAddrIpGroup) != "" {
		for srcIpGroupItem := range strings.SplitSeq(srcAddrIpGroup, ",") {
			var data []string
			data, err = iKuai.GetAllIKuaiBypassIpGroupNamesByName(srcIpGroupItem)
			if err != nil {
				return
			}
			srcAddrList = append(srcAddrList, data...)
		}
		if len(srcAddrList) > 0 {
			srcAddr = strings.Join(srcAddrList, ",") // #99
		} else {
			log.Println("域名分流== 未找到任何匹配的 源地址 IP 分组，跳过域名分流规则添加，配置的 srcAddrIpGroup:", srcAddrIpGroup)
			return nil
		}
	}
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
