package main

import (
	"errors"
	"io"
	"log"
	"net/http"
	"strings"
	"time"

	"github.com/joyanhui/ikuai-bypass/api"
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
	ips = removeIpv6(ips)
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

func removeIpv6(ips []string) []string {
	i := 0
	for _, ip := range ips {
		if !strings.Contains(ip, ":") {
			ips[i] = ip
			i++
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
