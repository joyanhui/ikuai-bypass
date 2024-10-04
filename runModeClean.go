package main

import (
	"log"
)

// 清理旧分流规则
func clean() {
	iKuai, err := loginToIkuai()
	if err != nil {
		log.Println("登录爱快失败：", err)
		return
	}

	//删除旧的自定义运营商
	err = iKuai.DelCustomIspAll(*cleanTag)
	if err != nil {
		log.Println("移除旧的自定义运营商失败 tag:"+*cleanTag+"：", err)
	} else {
		log.Println("移除旧的自定义运营商成功 tag:" + *cleanTag)
	}
	//删除旧的域名分流
	err = iKuai.DelStreamDomainAll(*cleanTag)
	if err != nil {
		log.Println("移除旧的域名分流失败 tag:"+*cleanTag+"：", err)
	} else {
		log.Println("移除旧的域名分流成功 tag:" + *cleanTag)
	}
	//删除旧的ip组
	err = iKuai.DelIKuaiBypassIpGroup(*cleanTag)
	if err != nil {
		log.Println("移除旧的IP分组失败 tag:"+*cleanTag+"：", err)
	} else {
		log.Println("移除旧的IP分组成功 tag:" + *cleanTag)
	}
	//删除端口分流规则
	err = iKuai.DelIKuaiBypassStreamIpPort(*cleanTag)
	if err != nil {
		log.Println("移除旧的端口分流失败 tag:"+*cleanTag+"：", err)
	} else {
		log.Println("移除旧的端口分流成功 tag:" + *cleanTag)
	}
}
