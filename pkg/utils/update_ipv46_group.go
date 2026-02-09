package utils

import (
	"errors"
	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/ikuai_common"
	"io"
	"log"
	"net/http"
	"strconv"
	"strings"
	"time"
)

// UpdateIpGroup 更新ip分组
func UpdateIpGroup(iKuai ikuai_common.IKuaiClient, name, url string) (err error) {
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
			log.Println("ip分组== ", index, "添加失败，", config.GlobalConfig.AddErrRetryWait, "秒后重试 err:", err)
			time.Sleep(config.GlobalConfig.AddWait)
		}

	}
	return
}

// UpdateIpv6Group 更新ipv6分组
func UpdateIpv6Group(iKuai ikuai_common.IKuaiClient, name, url string) (err error) {
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
			log.Println("ipv6分组== ", index, "添加失败，", config.GlobalConfig.AddErrRetryWait, "秒后重试 err:", err)
			time.Sleep(config.GlobalConfig.AddWait)
		}
	}
	return
}
