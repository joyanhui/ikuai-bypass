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

// UpdateStreamDomain 更新域名分流规则
func UpdateStreamDomain(logger *logger.Logger, iKuai ikuai_common.IKuaiClient, iface, tag, srcAddrIpGroup, srcAddr, url string) (err error) {
	logger.Info("HTTP:资源下载", "http.get %s", url)
	resp, err := http.Get(GetFullUrl(url))
	if err != nil {
		return err
	}
	if resp.StatusCode != 200 {
		return errors.New(resp.Status)
	}
	defer resp.Body.Close()
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return err
	}
	domains := strings.Split(string(body), "\n")
	domains = FilterDomains(logger, domains)
	logger.Info("PARSE:解析成功", "%s %s: obtained %d valid domains", iface, tag, len(domains))

	domainMap, err := iKuai.GetStreamDomainMap(tag)
	if err != nil {
		logger.Error("QUERY:查询列表", "Failed to get existing domain streaming rules: %v", err)
		return err
	}

	domainGroup := Group(domains, config.GlobalConfig.MaxNumberOfOneRecords.Domain)
	for i, d := range domainGroup {
		index := i + 1
		name := iKuai.BuildIndexedTagName(tag, i)
		domainList := strings.Join(d, ",")
		if id, ok := domainMap[index]; ok {
			logger.Info("EDIT:正在修改", "[%d/%d] %s %s: updating %s (ID: %d)...", i+1, len(domainGroup), iface, tag, name, id)
			err = iKuai.EditStreamDomain(iface, tag, srcAddr, srcAddrIpGroup, domainList, i, id)
			delete(domainMap, index)
		} else {
			logger.Info("ADD:正在添加", "[%d/%d] %s %s: adding %s...", i+1, len(domainGroup), iface, tag, name)
			err = iKuai.AddStreamDomain(iface, tag, srcAddr, srcAddrIpGroup, domainList, i)
		}
		if err != nil {
			logger.Error("UPDATE:更新失败", "[%d/%d] %s %s: failed: %v", i+1, len(domainGroup), iface, tag, err)
			time.Sleep(config.GlobalConfig.AddErrRetryWait)
		} else {
			if config.GlobalConfig.AddWait > 0 {
				time.Sleep(config.GlobalConfig.AddWait)
			}
		}
	}

	if len(domainMap) > 0 {
		var extraIds []string
		for idx, id := range domainMap {
			logger.Info("CLEAN:冗余删除", "%s: chunk %d (ID: %d) is no longer needed, deleting...", tag, idx, id)
			extraIds = append(extraIds, strconv.Itoa(id))
		}
		err = iKuai.DelStreamDomain(strings.Join(extraIds, ","))
		if err != nil {
			logger.Error("CLEAN:删除失败", "%s: failed to delete extra domain rules: %v", tag, err)
		} else {
			logger.Success("CLEAN:清理成功", "%s: deleted %d extra domain rules", tag, len(extraIds))
		}
	}
	return nil
}
