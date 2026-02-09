package core

import (
	"ikuai-bypass/pkg/ikuai_common"
	"log"
	"strconv"
	"strings"

	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/utils"
)

const (
	cleanModeAll    = "cleanAll"
	defaultCleanTag = "IKUAI_BYPASS"
)

// MainClean 清理旧分流规则
func MainClean() {
	iKuai, err := utils.LoginToIkuai()
	if err != nil {
		log.Println("登录爱快失败：", err)
		return
	}

	cleanTag, isCleanAll := normalizeCleanTag(*config.CleanTag)

	if isCleanAll {
		err = cleanAllByManagedMark(iKuai)
		if err != nil {
			log.Println("清理所有规则失败：", err)
		} else {
			log.Println("清理所有规则成功 tag:" + cleanTag)
		}
		return
	}

	//删除旧的自定义运营商
	err = iKuai.DelCustomIspAll(cleanTag)
	if err != nil {
		log.Println("移除旧的自定义运营商失败 tag:"+cleanTag+"：", err)
	} else {
		log.Println("移除旧的自定义运营商成功 tag:" + cleanTag)
	}
	//删除旧的域名分流
	err = iKuai.DelStreamDomainAll(cleanTag)
	if err != nil {
		log.Println("移除旧的域名分流失败 tag:"+cleanTag+"：", err)
	} else {
		log.Println("移除旧的域名分流成功 tag:" + cleanTag)
	}
	//删除旧的ip组
	err = iKuai.DelIKuaiBypassIpGroup(cleanTag)
	if err != nil {
		log.Println("移除旧的IP分组失败 tag:"+cleanTag+"：", err)
	} else {
		log.Println("移除旧的IP分组成功 tag:" + cleanTag)
	}
	//删除旧的ipv6组
	err = iKuai.DelIKuaiBypassIpv6Group(cleanTag)
	if err != nil {
		log.Println("移除旧的IPV6分组失败 tag:"+cleanTag+"：", err)
	} else {
		log.Println("移除旧的IPV6分组成功 tag:" + cleanTag)
	}
	//删除域名分组
	err = iKuai.DelIKuaiBypassDomainGroup(cleanTag)
	if err != nil {
		log.Println("移除旧的域名分组失败 tag:"+cleanTag+"：", err)
	} else {
		log.Println("移除旧的域名分组成功 tag:" + cleanTag)
	}
	//删除端口分流规则
	err = iKuai.DelIKuaiBypassStreamIpPort(cleanTag)
	if err != nil {
		log.Println("移除旧的端口分流失败 tag:"+cleanTag+"：", err)
	} else {
		log.Println("移除旧的端口分流成功 tag:" + cleanTag)
	}
}

func normalizeCleanTag(cleanTag string) (string, bool) {
	cleanTag = strings.TrimSpace(cleanTag)
	if cleanTag == "" {
		return defaultCleanTag, false
	}
	return cleanTag, cleanTag == cleanModeAll
}

func isManagedBypassRule(comment, name string) bool {
	return strings.Contains(comment, defaultCleanTag) ||
		strings.HasPrefix(name, "IKB_") ||
		strings.Contains(name, "IKB")
}

func cleanAllByManagedMark(iKuai ikuai_common.IKuaiClient) (err error) {
	err = cleanAllCustomIsp(iKuai)
	if err != nil {
		return err
	}
	err = cleanAllStreamDomain(iKuai)
	if err != nil {
		return err
	}
	err = cleanAllIpGroup(iKuai)
	if err != nil {
		return err
	}
	err = cleanAllIpv6Group(iKuai)
	if err != nil {
		return err
	}
	err = cleanAllDomainGroup(iKuai)
	if err != nil {
		return err
	}
	err = cleanAllStreamIpPort(iKuai)
	if err != nil {
		return err
	}
	return nil
}

func cleanAllCustomIsp(iKuai ikuai_common.IKuaiClient) (err error) {
	for {
		data, showErr := iKuai.ShowCustomIspByComment()
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
	}
}

func cleanAllStreamDomain(iKuai ikuai_common.IKuaiClient) (err error) {
	for {
		data, showErr := iKuai.ShowStreamDomainByComment("")
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
	}
}

func cleanAllIpGroup(iKuai ikuai_common.IKuaiClient) (err error) {
	for {
		data, showErr := iKuai.ShowIpGroupByComment("")
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
	}
}

func cleanAllIpv6Group(iKuai ikuai_common.IKuaiClient) (err error) {
	for {
		data, showErr := iKuai.ShowIpv6GroupByComment("")
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
	}
}

func cleanAllDomainGroup(iKuai ikuai_common.IKuaiClient) (err error) {
	for {
		data, showErr := iKuai.ShowDomainGroupByComment("")
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
	}
}

func cleanAllStreamIpPort(iKuai ikuai_common.IKuaiClient) (err error) {
	for {
		data, showErr := iKuai.ShowStreamIpPortByComment("")
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
	}
}
