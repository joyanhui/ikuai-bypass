package main

import (
	"errors"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"strings"

	"github.com/joyanhui/ikuai-bypass/api"
	"gopkg.in/yaml.v3"
)

// 读取配置文件 到 conf
func readConf(filename string) error {
	buf, err := os.ReadFile(filename)
	if err != nil {
		return err
	}
	err = yaml.Unmarshal(buf, &conf)
	if err != nil {
		return fmt.Errorf("in file %q: %v", filename, err)
	}

	// 检查每个 CustomIsp 的 Tag，如果不存在，则使用 Name
	for i := range conf.CustomIsp {
		if conf.CustomIsp[i].Tag == "" {
			log.Println("运营商分流规则中配置中Tag为空,采用:", conf.CustomIsp[i].Name)
			conf.CustomIsp[i].Tag = conf.CustomIsp[i].Name
		}
	}

	// 检查每个 StreamDomain 的 Tag，如果不存在，则使用 Interface
	for i := range conf.StreamDomain {
		if conf.StreamDomain[i].Tag == "" {
			log.Println("域名分流规则中中Tag为空,采用:", conf.StreamDomain[i].Interface)
			conf.StreamDomain[i].Tag = conf.StreamDomain[i].Interface
		}
	}

	return nil
	/*
		buf, err := os.ReadFile(filename)
		if err != nil {
			return err
		}
		err = yaml.Unmarshal(buf, &conf)
		if err != nil {
			return fmt.Errorf("in file %q: %v", filename, err)
		}
		return nil
	*/
}

// updateCustomIsp 更新运营商分流规则
func updateCustomIsp(iKuai *api.IKuai, name string, tag string, url string) (err error) {
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
	ips = removeIpv6(ips)
	ipGroups := group(ips, 5000) //5000条
	for _, ig := range ipGroups {
		ipGroup := strings.Join(ig, ",")
		err = iKuai.AddCustomIsp(name, tag, ipGroup)
	}
	return
}

// updateStreamDomain 更新域名分流规则
func updateStreamDomain(iKuai *api.IKuai, iface, tag, srcAddr, url string) (err error) {
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
	domains := strings.Split(string(body), "\n")
	domainGroup := group(domains, 1000) //1000条
	for _, d := range domainGroup {
		domain := strings.Join(d, ",")
		err = iKuai.AddStreamDomain(iface, tag, srcAddr, domain)
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
	max := int64(len(arr))
	var segmens = make([][]string, 0)
	quantity := max / subGroupLength
	remainder := max % subGroupLength
	i := int64(0)
	for i = int64(0); i < quantity; i++ {
		segmens = append(segmens, arr[i*subGroupLength:(i+1)*subGroupLength])
	}
	if quantity == 0 || remainder != 0 {
		segmens = append(segmens, arr[i*subGroupLength:i*subGroupLength+remainder])
	}
	return segmens
}
