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
// preDelIds: 如果非空，则在下载成功后、添加新规则前进行删除（Safe-Before 模式）
func UpdateCustomIsp(iKuai ikuai_common.IKuaiClient, name string, tag string, url string, preDelIds string) (err error) {
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

	// 如果提供了预删除 ID，则在开始添加前进行清理（确保下载成功后才删除）
	if preDelIds != "" {
		err = iKuai.DelCustomIspFromPreIds(preDelIds)
		if err != nil {
			log.Println("运营商/IP分流== 清理旧规则失败，跳过此次更新:", err)
			return
		}
		log.Println("运营商/IP分流== 已清理旧的运营商列表规则")
	}

	ipGroups := Group(ips, 5000) //5000条

	for _, ig := range ipGroups {
		ipGroup := strings.Join(ig, ",")
		err = iKuai.AddCustomIsp(name, tag, ipGroup)
		if err != nil {
			log.Println("运营商/IP分流==  ", name, tag, "添加失败，", config.GlobalConfig.AddErrRetryWait, "秒后重试 err:", err)
			time.Sleep(config.GlobalConfig.AddErrRetryWait)
			err = iKuai.AddCustomIsp(name, tag, ipGroup)
			if err != nil {
				log.Println("运营商/IP分流==  ", name, tag, "重试失败，已经重试过一次，所以跳过此次操作")
				break
			}
		}
		log.Println("运营商/IP分流==  添加ip:", len(ig), " 个,等待", config.GlobalConfig.AddWait, "秒继续处理")
		time.Sleep(config.GlobalConfig.AddWait)
	}
	return
}
