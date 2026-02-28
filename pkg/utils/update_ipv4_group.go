package utils
import (
	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/ikuai_common"
	"ikuai-bypass/pkg/logger"
	"strconv"
	"strings"
	"time"
)

// UpdateIpGroup 更新ip分组
func UpdateIpGroup(logger *logger.Logger, iKuai ikuai_common.IKuaiClient, tag, url string) (err error) {
	body, err := HttpGet(logger, url)
	if err != nil {
		return err
	}
	ips := strings.Split(string(body), "\n")
	ips = RemoveIpv6AndRemoveEmptyLine(logger, ips)
	ipGroups := Group(ips, config.GlobalConfig.MaxNumberOfOneRecords.Ipv4)
	logger.Success("PARSE:解析成功", "%s: obtained new data", tag)
	ipGroupMap, err := iKuai.GetIpGroupMap(tag)
	if err != nil {
		logger.Error("QUERY:查询列表", "Failed to get IP group map for update: %s, error: %v", tag, err)
		return
	} else {
		logger.Info("QUERY:查询成功", "%s: found %d existing groups", tag, len(ipGroupMap))
	}

	for i, ig := range ipGroups {
		index := i + 1
		name := iKuai.BuildIndexedTagName(tag, i)
		ipGroup := strings.Join(ig, ",")
		if id, ok := ipGroupMap[index]; ok {
			logger.Info("EDIT:正在修改", "[%d/%d] %s: updating %s (ID: %d)...", i+1, len(ipGroups), tag, name, id)
			err = iKuai.EditIpGroup(tag, ipGroup, i, id)
			delete(ipGroupMap, index) // Mark as updated
		} else {
			logger.Info("ADD:正在添加", "[%d/%d] %s: adding %s...", i+1, len(ipGroups), tag, name)
			err = iKuai.AddIpGroup(tag, ipGroup, i)
		}

		if err != nil {
			logger.Error("UPDATE:更新失败", "[%d/%d] %s: failed, error: %v", i+1, len(ipGroups), tag, err)
			time.Sleep(config.GlobalConfig.AddErrRetryWait)
		}
	}

	// Delete extra groups that were not updated
	if len(ipGroupMap) > 0 {
		var extraIds []string
		for _, id := range ipGroupMap {
			extraIds = append(extraIds, strconv.Itoa(id))
		}
		logger.Info("CLEAN:冗余删除", "%s: %d groups are no longer needed, deleting IDs: %s", tag, len(ipGroupMap), strings.Join(extraIds, ","))
		err = iKuai.DelIpGroup(strings.Join(extraIds, ","))
		if err != nil {
			logger.Error("CLEAN:删除失败", "%s: failed to delete extra groups: %v", tag, err)
		} else {
			logger.Success("CLEAN:清理成功", "%s: deleted %d extra groups", tag, len(extraIds))
		}
	}
	return
}
