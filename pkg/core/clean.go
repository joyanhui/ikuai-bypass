package core

import (
	"log"

	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/utils"
)

// MainClean 清理旧分流规则
func MainClean() {
	iKuai, err := utils.LoginToIkuai()
	if err != nil {
		log.Println("登录爱快失败：", err)
		return
	}

	//删除旧的自定义运营商
	err = iKuai.DelCustomIspAll(*config.CleanTag)
	if err != nil {
		log.Println("移除旧的自定义运营商失败 tag:"+*config.CleanTag+"：", err)
	} else {
		log.Println("移除旧的自定义运营商成功 tag:" + *config.CleanTag)
	}
	//删除旧的域名分流
	err = iKuai.DelStreamDomainAll(*config.CleanTag)
	if err != nil {
		log.Println("移除旧的域名分流失败 tag:"+*config.CleanTag+"：", err)
	} else {
		log.Println("移除旧的域名分流成功 tag:" + *config.CleanTag)
	}
	//删除旧的ip组
	err = iKuai.DelIKuaiBypassIpGroup(*config.CleanTag)
	if err != nil {
		log.Println("移除旧的IP分组失败 tag:"+*config.CleanTag+"：", err)
	} else {
		log.Println("移除旧的IP分组成功 tag:" + *config.CleanTag)
	}
	//删除旧的ipv6组
	err = iKuai.DelIKuaiBypassIpv6Group(*config.CleanTag)
	if err != nil {
		log.Println("移除旧的IPV6分组失败 tag:"+*config.CleanTag+"：", err)
	} else {
		log.Println("移除旧的IPV6分组成功 tag:" + *config.CleanTag)
	}
	//删除域名分组
	err = iKuai.DelIKuaiBypassDomainGroup(*config.CleanTag)
	if err != nil {
		log.Println("移除旧的域名分组失败 tag:"+*config.CleanTag+"：", err)
	} else {
		log.Println("移除旧的域名分组成功 tag:" + *config.CleanTag)
	}
	//删除端口分流规则
	err = iKuai.DelIKuaiBypassStreamIpPort(*config.CleanTag)
	if err != nil {
		log.Println("移除旧的端口分流失败 tag:"+*config.CleanTag+"：", err)
	} else {
		log.Println("移除旧的端口分流成功 tag:" + *config.CleanTag)
	}
}
