package core

import (
	"fmt"
	"log"
	"time"

	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/utils"
)

func MainUpdateIspRule() {
	iKuai, err := utils.LoginToIkuai()
	if err != nil {
		log.Println("登录爱快失败：", err)
		return
	}

	var GoroutineEnd1 bool = false
	var GoroutineEnd2 bool = false
	go func() {
		for _, customIsp := range config.GlobalConfig.CustomIsp {
			//记录旧的自定义运营商
			preIds, err := iKuai.GetCustomIspAll(customIsp.Tag)
			if err != nil {
				log.Println("运营商/IP分流== 获取准备更新的自定义运营商列表失败：", customIsp.Name, customIsp.Tag, err)
				break
			} else {
				log.Println("运营商/IP分流== 获取准备更新的自定义运营商列表成功", customIsp.Name, customIsp.Tag)
			}

			// 强制执行 "Safe-Before" 模式：先成功获取远程数据，再清理旧规则，后添加新分片
			// 这种模式最安全且符合 iKuai 4.0 规范
			log.Println("运营商/IP分流==  正在更新", customIsp.Name, customIsp.Tag)
			err = utils.UpdateCustomIsp(iKuai, customIsp.Name, customIsp.Tag, customIsp.URL, preIds)
			if err != nil {
				log.Printf("运营商/IP分流== 添加自定义运营商'%s'失败：%s\n", customIsp.Name, err)
			} else {
				log.Printf("运营商/IP分流== 添加自定义运营商'%s'成功\n", customIsp.Name)
			}
		}
		GoroutineEnd1 = true
	}()

	go func() {
		for _, streamDomain := range config.GlobalConfig.StreamDomain {
			//记录旧的域名分流
			preIds, err := iKuai.GetStreamDomainAll(streamDomain.Tag)
			if err != nil {
				log.Println("域名分流== 获取准备更新的域名列表失败：", streamDomain.Tag, err)
				break
			} else {
				log.Println("域名分流==  获取准备更新的域名列表成功", streamDomain.Tag)
			}

			//更新域名分流 (强制 Safe-Before)
			log.Println("域名分流==  正在更新", streamDomain.Interface, streamDomain.Tag, streamDomain.SrcAddrOptIpGroup, streamDomain.SrcAddr)
			err = utils.UpdateStreamDomain(iKuai, streamDomain.Interface, streamDomain.Tag, streamDomain.SrcAddrOptIpGroup, streamDomain.SrcAddr, streamDomain.URL, preIds)
			if err != nil {
				log.Printf("域名分流== 添加域名分流 '%s' 失败：%s\n", streamDomain.Interface, err)
			} else {
				log.Printf("域名分流== 添加域名分流 '%s' 成功\n", streamDomain.Interface)
			}
		}
		GoroutineEnd2 = true
	}()

	for { //等待两个协程结束
		if GoroutineEnd1 && GoroutineEnd2 {
			break
		}
		time.Sleep(1 * time.Second)
		fmt.Printf(".")
	}
}
