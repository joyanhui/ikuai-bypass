package core

import (
	"fmt"
	"log"
	"time"

	"github.com/joyanhui/ikuai-bypass/pkg/config"
	"github.com/joyanhui/ikuai-bypass/pkg/utils"
)

func UpdateIspRule() {
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
				//return
				break
			} else {
				log.Println("运营商/IP分流== 获取准备更新的自定义运营商列表成功", customIsp.Name, customIsp.Tag)
			}
			//是否要先删除旧规则
			if *config.DelOldRule == "before" {
				//删除旧的自定义运营商
				err = iKuai.DelCustomIspFromPreIds(preIds)
				if err == nil {
					log.Println("运营商/IP分流== 删除旧的运营商列表成功", customIsp.Name, customIsp.Tag)
					log.Println("运营商/IP分流== 更新完成", customIsp.Name, customIsp.Tag)
				} else {
					log.Println("运营商/IP分流== 删除旧的运营商列表有错误", customIsp.Name, customIsp.Tag, err)
				}
			}
			//更新自定义运营商
			log.Println("运营商/IP分流==  正在更新", customIsp.Name, customIsp.Tag)
			err = utils.UpdateCustomIsp(iKuai, customIsp.Name, customIsp.Tag, customIsp.URL)
			if err != nil {
				log.Printf("运营商/IP分流== 添加自定义运营商'%s'失败：%s\n", customIsp.Name, err)
			} else {
				log.Printf("运营商/IP分流== 添加自定义运营商'%s'成功\n", customIsp.Name)
				if *config.DelOldRule == "after" {
					//删除旧的自定义运营商
					err = iKuai.DelCustomIspFromPreIds(preIds)
					if err == nil {
						log.Println("运营商/IP分流== 删除旧的运营商列表成功", customIsp.Name, customIsp.Tag)
						log.Println("运营商/IP分流== 更新完成", customIsp.Name, customIsp.Tag)
					} else {
						log.Println("运营商/IP分流== 删除旧的运营商列表有错误", customIsp.Name, customIsp.Tag, err)
					}
				}

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
			if *config.DelOldRule == "before" {
				//删除旧的域名分流
				err = iKuai.DelStreamDomainFromPreIds(preIds)
				if err == nil {
					log.Println("域名分流==  删除旧的运营商列表成功")
				} else {
					log.Println("域名分流==  删除旧的运营商列表有错误", err)
				}
			}
			//更新域名分流
			log.Println("域名分流==  正在更新", streamDomain.Interface, streamDomain.Tag, streamDomain.SrcAddr)
			err = utils.UpdateStreamDomain(iKuai, streamDomain.Interface, streamDomain.Tag, streamDomain.SrcAddr, streamDomain.URL)
			if err != nil {
				log.Printf("域名分流== 添加域名分流 '%s' 失败：%s\n", streamDomain.Interface, err)
			} else {
				log.Printf("域名分流== 添加域名分流 '%s' 成功\n", streamDomain.Interface)
				if *config.DelOldRule == "after" {
					//删除旧的域名分流
					err = iKuai.DelStreamDomainFromPreIds(preIds)
					if err == nil {
						log.Println("域名分流==  删除旧的运营商列表成功")
					} else {
						log.Println("域名分流==  删除旧的运营商列表有错误", err)
					}
				}

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
