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

// UpdateStreamDomain 更新域名分流规则
// preDelIds: 如果非空，则在下载成功后、添加新规则前进行删除（Safe-Before 模式）
func UpdateStreamDomain(logger *logger.Logger, iKuai ikuai_common.IKuaiClient, iface, tag, srcAddrIpGroup, srcAddr, url string, preDelIds string) (err error) {
	logger.Info("资源下载", "http.get %s", url)
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
	// 清理无效域名
	domains = FilterDomains(logger, domains)

	logger.Info("解析成功", "%s %s: obtained %d valid domains", iface, tag, len(domains))

	// 如果提供了预删除 ID，则在开始添加前进行清理（确保下载成功后才删除）
	if preDelIds != "" {
		count := len(strings.Split(preDelIds, ","))
		err = iKuai.DelStreamDomainFromPreIds(preDelIds)
		if err != nil {
			logger.Error("清理旧规", "Failed to clear old rules, skipping update: %v", err)
			return
		}
		logger.Success("清理旧规", "Cleared %d old domain streaming rules", count)
	}

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
			logger.Info("跳过操作", "No matching source IP groups found, skipping rule addition. srcAddrIpGroup: %s", srcAddrIpGroup)
			return nil
		}
	}

	for index, d := range domainGroup {
		logger.Info("正在添加", "[%d/%d] %s %s: adding...", index+1, len(domainGroup), iface, tag)
		domain := strings.Join(d, ",")
		err = iKuai.AddStreamDomain(iface, tag, srcAddr, domain, index)
		if err != nil {
			logger.Error("添加失败", "[%d/%d] %s %s: failed, retrying after %v seconds. error: %v", index+1, len(domainGroup), iface, tag, config.GlobalConfig.AddErrRetryWait, err)
			time.Sleep(config.GlobalConfig.AddErrRetryWait)
			err = iKuai.AddStreamDomain(iface, tag, srcAddr, domain, index)
			if err != nil {
				logger.Error("重试失败", "[%d/%d] %s %s: retry failed, skipping this operation", index+1, len(domainGroup), iface, tag)
				break
			}
		} else {
			logger.Success("添加成功", "%s %s: added %d domains. Waiting %v seconds...", iface, tag, len(d), config.GlobalConfig.AddWait)
			time.Sleep(config.GlobalConfig.AddWait)
		}
	}
	return
}
