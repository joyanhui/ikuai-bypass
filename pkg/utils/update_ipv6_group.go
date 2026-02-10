package utils

import (
	"errors"
	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/ikuai_common"
	"ikuai-bypass/pkg/logger"
	"io"
	"net/http"
	"strings"
	"time"
)

// UpdateIpv6Group 更新ipv6分组
func UpdateIpv6Group(logger *logger.Logger, iKuai ikuai_common.IKuaiClient, tag, url string) (err error) {
	logger.Info("HTTP:资源下载", "http.get %s", url)
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
	ipGroups := Group(ips, config.GlobalConfig.MaxNumberOfOneRecords.Ipv6)
	logger.Success("PARSE:解析成功", "%s: obtained new data", tag)
	preIds, err := iKuai.GetIpv6Group(tag)
	if err != nil {
		logger.Error("QUERY:查询列表", "Failed to get IPv6 group list for update: %s, error: %v", tag, err)
		return
	} else {
		logger.Info("QUERY:查询成功", "%s: old IPv6 group IDs found: %s", tag, preIds)
	}

	if preIds != "" {
		count := len(strings.Split(preIds, ","))
		err = iKuai.DelIpv6Group(preIds)
		if err == nil {
			logger.Success("CLEAN:清理旧规", "%s: cleared %d old IPv6 groups", tag, count)
		} else {
			logger.Error("CLEAN:清理失败", "%s: error clearing old IPv6 group list: %v", tag, err)
			return
		}
	}

	preIds = ""
	for i, ig := range ipGroups {
		logger.Info("ADD:正在添加", "[%d/%d] %s: adding...", i+1, len(ipGroups), tag)
		ipGroup := strings.Join(ig, ",")
		err := iKuai.AddIpv6Group(tag, ipGroup, i)
		if err != nil {
			logger.Error("ADD:添加失败", "[%d/%d] %s: failed, retrying after %v seconds. error: %v", i+1, len(ipGroups), tag, config.GlobalConfig.AddErrRetryWait, err)
			time.Sleep(config.GlobalConfig.AddWait)
			err = iKuai.AddIpv6Group(tag, ipGroup, i)
			if err != nil {
				logger.Error("ADD:重试失败", "[%d/%d] %s: retry failed, skipping this operation", i+1, len(ipGroups), tag)
				break
			}
		}
	}
	return
}
