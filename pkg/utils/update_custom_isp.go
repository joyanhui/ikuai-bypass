package utils
import (
	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/ikuai_common"
	"ikuai-bypass/pkg/logger"
	"strconv"
	"strings"
	"time"
)
// UpdateCustomIsp 更新运营商分流规则
// UpdateCustomIsp updates custom ISP routing rules
// UpdateCustomIsp updates custom ISP routing rules
// preDelIds: 如果非空，则在下载成功后、添加新规则前进行删除（Safe-Before 模式）
// preDelIds: if not empty, old rules are deleted after successful download but before adding new ones (Safe-Before mode)
func UpdateCustomIsp(logger *logger.Logger, iKuai ikuai_common.IKuaiClient, name string, url string) (err error) {
	body, err := HttpGet(logger, url)
	if err != nil {
		return err
	}
	ips := strings.Split(string(body), "\n")
	ips = RemoveIpv6AndRemoveEmptyLine(logger, ips)
	logger.Info("STAT:规则统计", "Fetched %d IPs for %s", len(ips), name)

	ispMap, err := iKuai.GetCustomIspMap(name)
	if err != nil {
		logger.Error("QUERY:查询规则", "Failed to get existing custom ISP rules: %v", err)
		return err
	}

	ipGroups := Group(ips, config.GlobalConfig.MaxNumberOfOneRecords.Isp)
	for i, ig := range ipGroups {
		index := i + 1
		ipGroup := strings.Join(ig, ",")
		if id, ok := ispMap[index]; ok {
			logger.Info("EDIT:正在修改", "[%d/%d] %s: updating chunk %d (ID: %d)...", i+1, len(ipGroups), name, index, id)
			err = iKuai.EditCustomIsp(name, ipGroup, i, id)
			delete(ispMap, index)
		} else {
			logger.Info("ADD:正在添加", "[%d/%d] %s: adding chunk %d...", i+1, len(ipGroups), name, index)
			err = iKuai.AddCustomIsp(name, ipGroup, i)
		}
		if err != nil {
			logger.Error("UPDATE:更新失败", "[%d/%d] %s: failed: %v", i+1, len(ipGroups), name, err)
			time.Sleep(config.GlobalConfig.AddErrRetryWait)
		} else {
			if config.GlobalConfig.AddWait > 0 {
				time.Sleep(config.GlobalConfig.AddWait)
			}
		}
	}

	if len(ispMap) > 0 {
		var extraIds []string
		for idx, id := range ispMap {
			logger.Info("CLEAN:冗余删除", "%s: chunk %d (ID: %d) is no longer needed, deleting...", name, idx, id)
			extraIds = append(extraIds, strconv.Itoa(id))
		}
		err = iKuai.DelCustomIsp(strings.Join(extraIds, ","))
		if err != nil {
			logger.Error("CLEAN:删除失败", "%s: failed to delete extra rules: %v", name, err)
		} else {
			logger.Success("CLEAN:清理成功", "%s: deleted %d extra rules", name, len(extraIds))
		}
	}
	return nil
}
