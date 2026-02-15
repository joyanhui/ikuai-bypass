package core

import (
	"ikuai-bypass/pkg/ikuai_common"
	"strconv"
	"strings"

	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/logger"
	"ikuai-bypass/pkg/utils"
)

// MainClean 清理旧分流规则
func MainClean() {
	cleanLogger := logger.NewLogger("CLEAN:清理模式")
	cleanTag, isCleanAll := normalizeCleanTag(*config.CleanTag)
	if cleanTag == "" {
		cleanLogger.Error("VALID:参数校验", "清理模式必须指定 -tag 参数 (例如: -tag cleanAll 或 -tag 某规则名)")
		return
	}
	iKuai, err := utils.LoginToIkuai()
	if err != nil {
		utils.SysLog.Error("LOGIN:登录失败", "Failed to login to iKuai: %v", err)
		return
	}

	if isCleanAll {
		err = cleanAllByManagedMark(cleanLogger, iKuai)
		if err != nil {
			cleanLogger.Error("CLEAN:操作失败", "Failed to clear all rules: %v", err)
		} else {
			cleanLogger.Success("CLEAN:操作成功", "Cleared all rules with tag: %s", cleanTag)
		}
		return
	}

	//删除旧的自定义运营商
	err = iKuai.DelCustomIspAll(cleanTag)
	if err != nil {
		cleanLogger.Error("CLEAN:清理失败", "Failed to remove old custom ISP for tag %s: %v", cleanTag, err)
	} else {
		cleanLogger.Success("CLEAN:清理成功", "Removed old custom ISP for tag %s", cleanTag)
	}
	//删除旧的域名分流
	err = iKuai.DelStreamDomainAll(cleanTag)
	if err != nil {
		cleanLogger.Error("CLEAN:清理失败", "Failed to remove old domain streaming for tag %s: %v", cleanTag, err)
	} else {
		cleanLogger.Success("CLEAN:清理成功", "Removed old domain streaming for tag %s", cleanTag)
	}
	//删除旧的ip组
	err = iKuai.DelIKuaiBypassIpGroup(cleanTag)
	if err != nil {
		cleanLogger.Error("CLEAN:清理失败", "Failed to remove old IP group for tag %s: %v", cleanTag, err)
	} else {
		cleanLogger.Success("CLEAN:清理成功", "Removed old IP group for tag %s", cleanTag)
	}
	//删除旧的ipv6组
	err = iKuai.DelIKuaiBypassIpv6Group(cleanTag)
	if err != nil {
		cleanLogger.Error("CLEAN:清理失败", "Failed to remove old IPv6 group for tag %s: %v", cleanTag, err)
	} else {
		cleanLogger.Success("CLEAN:清理成功", "Removed old IPv6 group for tag %s", cleanTag)
	}
	//删除域名分组
	err = iKuai.DelIKuaiBypassDomainGroup(cleanTag)
	if err != nil {
		cleanLogger.Error("CLEAN:清理失败", "Failed to remove old domain group for tag %s: %v", cleanTag, err)
	} else {
		cleanLogger.Success("CLEAN:清理成功", "Removed old domain group for tag %s", cleanTag)
	}
	//删除端口分流规则
	err = iKuai.DelIKuaiBypassStreamIpPort(cleanTag)
	if err != nil {
		cleanLogger.Error("CLEAN:清理失败", "Failed to remove old port streaming for tag %s: %v", cleanTag, err)
	} else {
		cleanLogger.Success("CLEAN:清理成功", "Removed old port streaming for tag %s", cleanTag)
	}
}

func normalizeCleanTag(cleanTag string) (string, bool) {
	cleanTag = strings.TrimSpace(cleanTag)
	return cleanTag, cleanTag == ikuai_common.CleanModeAll
}

func isManagedBypassRule(comment string, name string) bool {
	// 名字以 IKB 开头
	if strings.HasPrefix(name, ikuai_common.NAME_PREFIX_IKB) {
		return true
	}
	// 备注包含 ikuai-bypass (新) 或 IKUAI_BYPASS (旧)
	if strings.Contains(comment, ikuai_common.NEW_COMMENT) || strings.Contains(comment, ikuai_common.COMMENT_IKUAI_BYPASS) {
		return true
	}
	return false
}

func cleanAllByManagedMark(l *logger.Logger, iKuai ikuai_common.IKuaiClient) (err error) {
	err = cleanAllCustomIsp(l, iKuai)
	if err != nil {
		return err
	}
	err = cleanAllStreamDomain(l, iKuai)
	if err != nil {
		return err
	}
	err = cleanAllIpGroup(l, iKuai)
	if err != nil {
		return err
	}
	err = cleanAllIpv6Group(l, iKuai)
	if err != nil {
		return err
	}
	err = cleanAllDomainGroup(l, iKuai)
	if err != nil {
		return err
	}
	err = cleanAllStreamIpPort(l, iKuai)
	if err != nil {
		return err
	}
	return nil
}

func cleanAllCustomIsp(l *logger.Logger, iKuai ikuai_common.IKuaiClient) (err error) {
	for {
		data, showErr := iKuai.ShowCustomIspByTagName("")
		if showErr != nil {
			return showErr
		}
		var ids []string
		for _, d := range data {
			if isManagedBypassRule(d.Comment, d.Name) {
				ids = append(ids, strconv.Itoa(d.ID))
			}
		}
		if len(ids) == 0 {
			return nil
		}
		err = iKuai.DelCustomIsp(strings.Join(ids, ","))
		if err != nil {
			return err
		}
		l.Success("CLEAN:清理详情", "Removed %d managed custom ISP rules", len(ids))
	}
}

func cleanAllStreamDomain(l *logger.Logger, iKuai ikuai_common.IKuaiClient) (err error) {
	for {
		data, showErr := iKuai.ShowStreamDomainByTagName("")
		if showErr != nil {
			return showErr
		}
		var ids []string
		for _, d := range data {
			if isManagedBypassRule(d.Comment, d.TagName) {
				ids = append(ids, strconv.Itoa(d.ID))
			}
		}
		if len(ids) == 0 {
			return nil
		}
		err = iKuai.DelStreamDomain(strings.Join(ids, ","))
		if err != nil {
			return err
		}
		l.Success("CLEAN:清理详情", "Removed %d managed domain streaming rules", len(ids))
	}
}

func cleanAllIpGroup(l *logger.Logger, iKuai ikuai_common.IKuaiClient) (err error) {
	for {
		data, showErr := iKuai.ShowIpGroupByTagName("")
		if showErr != nil {
			return showErr
		}
		var ids []string
		for _, d := range data {
			if isManagedBypassRule(d.Comment, d.GroupName) {
				ids = append(ids, strconv.Itoa(d.ID))
			}
		}
		if len(ids) == 0 {
			return nil
		}
		err = iKuai.DelIpGroup(strings.Join(ids, ","))
		if err != nil {
			return err
		}
		l.Success("CLEAN:清理详情", "Removed %d managed IP groups", len(ids))
	}
}

func cleanAllIpv6Group(l *logger.Logger, iKuai ikuai_common.IKuaiClient) (err error) {
	for {
		data, showErr := iKuai.ShowIpv6GroupByTagName("")
		if showErr != nil {
			return showErr
		}
		var ids []string
		for _, d := range data {
			if isManagedBypassRule(d.Comment, d.GroupName) {
				ids = append(ids, strconv.Itoa(d.ID))
			}
		}
		if len(ids) == 0 {
			return nil
		}
		err = iKuai.DelIpv6Group(strings.Join(ids, ","))
		if err != nil {
			return err
		}
		l.Success("CLEAN:清理详情", "Removed %d managed IPv6 groups", len(ids))
	}
}

func cleanAllDomainGroup(l *logger.Logger, iKuai ikuai_common.IKuaiClient) (err error) {
	for {
		data, showErr := iKuai.ShowDomainGroupByTagName("")
		if showErr != nil {
			return showErr
		}
		var ids []string
		for _, d := range data {
			if isManagedBypassRule(d.Comment, d.GroupName) {
				ids = append(ids, strconv.Itoa(d.ID))
			}
		}
		if len(ids) == 0 {
			return nil
		}
		err = iKuai.DelDomainGroup(strings.Join(ids, ","))
		if err != nil {
			return err
		}
		l.Success("CLEAN:清理详情", "Removed %d managed domain groups", len(ids))
	}
}

func cleanAllStreamIpPort(l *logger.Logger, iKuai ikuai_common.IKuaiClient) (err error) {
	for {
		data, showErr := iKuai.ShowStreamIpPortByTagName("")
		if showErr != nil {
			return showErr
		}
		var ids []string
		for _, d := range data {
			if isManagedBypassRule(d.Comment, d.TagName) {
				ids = append(ids, strconv.Itoa(d.ID))
			}
		}
		if len(ids) == 0 {
			return nil
		}
		err = iKuai.DelStreamIpPort(strings.Join(ids, ","))
		if err != nil {
			return err
		}
		l.Success("CLEAN:清理详情", "Removed %d managed port streaming rules", len(ids))
	}
}
