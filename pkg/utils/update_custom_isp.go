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

// UpdateCustomIsp 更新运营商分流规则
// preDelIds: 如果非空，则在下载成功后、添加新规则前进行删除（Safe-Before 模式）
func UpdateCustomIsp(logger *logger.Logger, iKuai ikuai_common.IKuaiClient, name string, tag string, url string, preDelIds string) (err error) {
	logger.Info("HTTP:数据获取", "Downloading rules from URL: %s", url)
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
	ips = RemoveIpv6AndRemoveEmptyLine(logger, ips)
	logger.Info("STAT:规则统计", "Fetched %d IPs for %s (%s)", len(ips), name, tag)

	// 如果提供了预删除 ID，则在开始添加前进行清理（确保下载成功后才删除）
	if preDelIds != "" {
		count := len(strings.Split(preDelIds, ","))
		err = iKuai.DelCustomIspFromPreIds(preDelIds)
		if err != nil {
			logger.Error("CLEAN:清理旧规则", "Failed to clear old rules, skipping update: %v", err)
			return
		}
		logger.Success("CLEAN:清理旧规则", "Successfully cleared %d old custom ISP rules", count)
	}

	ipGroups := Group(ips, 5000) //5000条

	for i, ig := range ipGroups {
		logger.Info("ADD:正在添加", "[%d/%d] %s: adding...", i+1, len(ipGroups), name)
		ipGroup := strings.Join(ig, ",")
		err = iKuai.AddCustomIsp(name, tag, ipGroup)
		if err != nil {
			logger.Error("ADD:添加失败", "[%d/%d] %s: failed, retrying after %v seconds. error: %v", i+1, len(ipGroups), name, config.GlobalConfig.AddErrRetryWait, err)
			time.Sleep(config.GlobalConfig.AddErrRetryWait)
			err = iKuai.AddCustomIsp(name, tag, ipGroup)
			if err != nil {
				logger.Error("ADD:重试失败", "[%d/%d] %s: retry failed, skipping this operation", i+1, len(ipGroups), name)
				break
			}
		} else {
			logger.Success("ADD:添加成功", "%s (%s): added %d IPs. Waiting %v seconds...", name, tag, len(ig), config.GlobalConfig.AddWait)
			time.Sleep(config.GlobalConfig.AddWait)
		}
	}
	return nil
}
