package utils

import (
	"errors"
	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/ikuai_common"
	"io"
	"log"
	"net/http"
	"strings"
	"time"
)

// UpdateCustomIsp 更新运营商分流规则
func UpdateCustomIsp(iKuai ikuai_common.IKuaiClient, name string, tag string, url string) (err error) {
	log.Println("运营商/IP分流==  http.get ...", url)
	resp, err := http.Get(GetFullUrl(url))
	if err != nil {
		return
	}
	if resp.StatusCode != 200 {
		err = errors.New(resp.Status)
		return
	}
	defer func(Body io.ReadCloser) {
		_ = Body.Close()
	}(resp.Body)
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return
	}
	ips := strings.Split(string(body), "\n")
	ips = RemoveIpv6AndRemoveEmptyLine(ips)
	log.Println("运营商/IP分流== ", name, tag, " 获取到", len(ips), "个ip")
	ipGroups := Group(ips, 5000) //5000条

	for _, ig := range ipGroups {
		ipGroup := strings.Join(ig, ",")
		err = iKuai.AddCustomIsp(name, tag, ipGroup)
		if err != nil {
			log.Println("运营商/IP分流==  ", name, tag, "添加失败，可能是列表太多了，添加太快,爱快没响应。", config.GlobalConfig.AddErrRetryWait, "秒后重试", err)
			time.Sleep(config.GlobalConfig.AddErrRetryWait)
			err = iKuai.AddCustomIsp(name, tag, ipGroup)
			if err != nil {
				log.Println("运营商/IP分流==  ", name, tag, "重试失败，可能是列表太多了，添加太快,爱快没响应。已经重试过一次，所以跳过此次操作")
				break
			}
		}
		log.Println("运营商/IP分流==  添加ip:", len(ig), " 个,等待", config.GlobalConfig.AddWait, "秒继续处理")
		time.Sleep(config.GlobalConfig.AddWait)
	}
	return
}
