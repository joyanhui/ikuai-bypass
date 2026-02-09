package utils

import (
	"errors"
	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/ikuai_common"
	"ikuai-bypass/pkg/logger"
	"io"
	"net/http"
	"strconv"
	"strings"
	"time"
)

// UpdateIpGroup 更新ip分组
func UpdateIpGroup(logger *logger.Logger, iKuai ikuai_common.IKuaiClient, name, url string) (err error) {
	logger.Info("资源下载", "http.get %s", url)
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
	ips = RemoveIpv6AndRemoveEmptyLine(logger, ips)
	ipGroups := Group(ips, 1000)
	logger.Success("解析成功", "%s: obtained new data", name)
	preIds, err := iKuai.GetIpGroup(name)
	if err != nil {
		logger.Error("查询列表", "Failed to get IP group list for update: %s, error: %v", name, err)
		return
	} else {
		logger.Info("查询成功", "%s: old group IDs found: %s", name, preIds)
	}

	if preIds != "" {
		count := len(strings.Split(preIds, ","))
		err = iKuai.DelIpGroup(preIds)
		if err == nil {
			logger.Success("清理旧规", "%s: cleared %d old IP groups", name, count)
		} else {
			logger.Error("清理失败", "%s: error clearing old IP group list: %v", name, err)
			return
		}
	}

	preIds = ""
	for index, ig := range ipGroups {
		logger.Info("正在添加", "[%d/%d] %s: adding...", index+1, len(ipGroups), name)
		ipGroup := strings.Join(ig, ",")
		err := iKuai.AddIpGroup(name+"_"+strconv.Itoa(index), ipGroup)
		if err != nil {
			logger.Error("添加失败", "[%d/%d] %s: failed, retrying after %v seconds. error: %v", index+1, len(ipGroups), name, config.GlobalConfig.AddErrRetryWait, err)
			time.Sleep(config.GlobalConfig.AddWait)
		}
	}
	return
}

// UpdateIpv6Group 更新ipv6分组
func UpdateIpv6Group(logger *logger.Logger, iKuai ikuai_common.IKuaiClient, name, url string) (err error) {
	logger.Info("资源下载", "http.get %s", url)
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
	ips = RemoveIpv4AndRemoveEmptyLine(logger, ips)
	ipGroups := Group(ips, 1000)
	logger.Success("解析成功", "%s: obtained new data", name)
	preIds, err := iKuai.GetIpv6Group(name)
	if err != nil {
		logger.Error("查询列表", "Failed to get IPv6 group list for update: %s, error: %v", name, err)
		return
	} else {
		logger.Info("查询成功", "%s: old IPv6 group IDs found: %s", name, preIds)
	}

	if preIds != "" {
		count := len(strings.Split(preIds, ","))
		err = iKuai.DelIpv6Group(preIds)
		if err == nil {
			logger.Success("清理旧规", "%s: cleared %d old IPv6 groups", name, count)
		} else {
			logger.Error("清理失败", "%s: error clearing old IPv6 group list: %v", name, err)
			return
		}
	}

	preIds = ""
	for index, ig := range ipGroups {
		logger.Info("正在添加", "[%d/%d] %s: adding...", index+1, len(ipGroups), name)
		ipGroup := strings.Join(ig, ",")
		err := iKuai.AddIpv6Group(name+"_"+strconv.Itoa(index), ipGroup)
		if err != nil {
			logger.Error("添加失败", "[%d/%d] %s: failed, retrying after %v seconds. error: %v", index+1, len(ipGroups), name, config.GlobalConfig.AddErrRetryWait, err)
			time.Sleep(config.GlobalConfig.AddWait)
		}
	}
	return
}
