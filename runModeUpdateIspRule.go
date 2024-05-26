package main

import (
	"fmt"
	"log"
	"time"

	"github.com/joyanhui/ikuai-bypass/api"
	"github.com/joyanhui/ikuai-bypass/router"
)

func updateIspRule() {
	err := readConf(*confPath)
	if err != nil {
		log.Println("读取配置文件失败：", err)
		return
	}
	baseurl := conf.IkuaiURL
	if baseurl == "" {
		gateway, err := router.GetGateway()
		if err != nil {
			log.Println("获取默认网关失败：", err)
			return
		}
		baseurl = "http://" + gateway
		log.Println("使用默认网关地址：", baseurl)
	}
	iKuai := api.NewIKuai(baseurl)
	err = iKuai.Login(conf.Username, conf.Password)
	if err != nil {
		log.Println("ikuai 登陆失败：", baseurl, err)
		return
	} else {
		log.Println("ikuai 登录成功", baseurl)
	}

	var GoroutineEnd1 bool = false
	var GoroutineEnd2 bool = false
	go func() {
		for _, customIsp := range conf.CustomIsp {
			//记录旧的自定义运营商
			preIds, err := iKuai.PrepareDelCustomIspAll(customIsp.Tag)
			if err != nil {
				log.Println("运营商/IP分流== 获取准备更新的自定义运营商列表失败：", customIsp.Name, customIsp.Tag, err)
				//return
				break
			} else {
				log.Println("运营商/IP分流== 获取准备更新的自定义运营商列表成功", customIsp.Name, customIsp.Tag)
			}
			//更新自定义运营商
			log.Println("运营商/IP分流==  正在更新", customIsp.Name, customIsp.Tag)
			err = updateCustomIsp(iKuai, customIsp.Name, customIsp.Tag, customIsp.URL)
			if err != nil {
				log.Printf("运营商/IP分流== 添加自定义运营商'%s'失败：%s\n", customIsp.Name, err)
			} else {
				log.Printf("运营商/IP分流== 添加自定义运营商'%s'成功\n", customIsp.Name)
				if err == nil {
					//删除旧的自定义运营商
					err = iKuai.DelCustomIspFromPreIds(preIds)
					if err == nil {
						log.Println("运营商/IP分流== 删除旧的运营商列表成功", customIsp.Name, customIsp.Tag)
						log.Println("运营商/IP分流== 更新完成", customIsp.Name, customIsp.Tag)
					} else {
						log.Println("运营商/IP分流== 删除旧的运营商列表有错误", customIsp.Name, customIsp.Tag, err)
					}
				} else {
					log.Println("运营商/IP分流== 添加运营商的时候有错误", customIsp.Name, customIsp.Tag, err)
				}
			}

		}

		GoroutineEnd1 = true
	}()

	go func() {

		for _, streamDomain := range conf.StreamDomain {
			//记录旧的域名分流
			preIds, err := iKuai.PrepareDelStreamDomainAll(streamDomain.Tag)
			if err != nil {
				log.Println("域名分流== 获取准备更新的域名列表失败：", streamDomain.Tag, err)
				break
			} else {
				log.Println("域名分流==  获取准备更新的域名列表成功", streamDomain.Tag)
			}
			//更新域名分流

			log.Println("域名分流==  正在更新", streamDomain.Interface, streamDomain.Tag, streamDomain.SrcAddr)
			err = updateStreamDomain(iKuai, streamDomain.Interface, streamDomain.Tag, streamDomain.SrcAddr, streamDomain.URL)
			if err != nil {
				log.Printf("域名分流== 添加域名分流 '%s' 失败：%s\n", streamDomain.Interface, err)
			} else {
				log.Printf("域名分流== 添加域名分流 '%s' 成功\n", streamDomain.Interface)
				if err == nil {
					//删除旧的域名分流
					err = iKuai.DelStreamDomainFromPreIds(preIds)
					if err == nil {
						log.Println("域名分流==  删除旧的运营商列表成功")
					} else {
						log.Println("域名分流==  删除旧的运营商列表有错误", err)
					}
				} else {
					log.Println("域名分流==  添加运营商的时候有错误", err)
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
