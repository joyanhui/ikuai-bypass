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

// UpdateIpv6Group 更新ipv6分组
func UpdateIpv6Group(logger *logger.Logger, iKuai ikuai_common.IKuaiClient, tag, url string) (err error) {
	logger.Info("HTTP:资源下载", "http.get %s", url)
	resp, err := http.Get(GetFullUrl(url))
	if err != nil {
		return err
	}
	if resp.StatusCode != 200 {
		_ = resp.Body.Close()
		return errors.New(resp.Status)
	}
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
	ipGroupMap, err := iKuai.GetIpv6GroupMap(tag)
	if err != nil {
		logger.Error("QUERY:查询列表", "Failed to get IPv6 group map for update: %s, error: %v", tag, err)
		return
	} else {
		logger.Info("QUERY:查询成功", "%s: found %d existing IPv6 groups", tag, len(ipGroupMap))
	}

	for i, ig := range ipGroups {
		index := i + 1
		name := iKuai.BuildIndexedTagName(tag, i)
		ipGroup := strings.Join(ig, ",")
		if id, ok := ipGroupMap[index]; ok {
			logger.Info("EDIT:正在修改", "[%d/%d] %s: updating %s (ID: %d)...", i+1, len(ipGroups), tag, name, id)
			err = iKuai.EditIpv6Group(tag, ipGroup, i, id)
			delete(ipGroupMap, index)
		} else {
			logger.Info("ADD:正在添加", "[%d/%d] %s: adding %s...", i+1, len(ipGroups), tag, name)
			err = iKuai.AddIpv6Group(tag, ipGroup, i)
		}

		if err != nil {
			logger.Error("UPDATE:更新失败", "[%d/%d] %s: failed, error: %v", i+1, len(ipGroups), tag, err)
			time.Sleep(config.GlobalConfig.AddErrRetryWait)
		}
	}

	if len(ipGroupMap) > 0 {
		var extraIds []string
		for _, id := range ipGroupMap {
			extraIds = append(extraIds, strconv.Itoa(id))
		}
		logger.Info("CLEAN:冗余删除", "%s: %d IPv6 groups are no longer needed, deleting IDs: %s", tag, len(ipGroupMap), strings.Join(extraIds, ","))
		err = iKuai.DelIpv6Group(strings.Join(extraIds, ","))
		if err != nil {
			logger.Error("CLEAN:删除失败", "%s: failed to delete extra IPv6 groups: %v", tag, err)
		} else {
			logger.Success("CLEAN:清理成功", "%s: deleted %d extra IPv6 groups", tag, len(extraIds))
		}
	}
	return
}
