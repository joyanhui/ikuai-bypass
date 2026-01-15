package core

import (
	"errors"
	"io"
	"log"
	"net/http"
	"os"
	"strconv"
	"strings"
	"time"

	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/utils"
)

func MainExportDomainSteamToTxt() {
	line := ""

	for _, streamDomain := range config.GlobalConfig.StreamDomain {
		log.Println("域名分流== ", streamDomain.Interface, streamDomain.Tag, "正在获取....")
		log.Println("域名分流==  http.get ...", streamDomain.URL)
		resp, err := http.Get(streamDomain.URL)
		if err != nil {
			log.Println("域名分流== ", streamDomain.Interface, streamDomain.Tag, "获取失败 Get", err)
			return
		}
		if resp.StatusCode != 200 {
			err = errors.New(resp.Status)
			log.Println("域名分流== ", streamDomain.Interface, streamDomain.Tag, "获取失败 Status", err)
			return
		}
		body, err := io.ReadAll(resp.Body)
		if err != nil {
			log.Println("域名分流== ", streamDomain.Interface, streamDomain.Tag, "获取失败 ReadAll", err)
			return
		}
		domains := strings.Split(string(body), "\n")
		log.Println("域名分流== ", streamDomain.Interface, streamDomain.Tag, "获取到", len(domains), "个域名")
		domainGroup := utils.Group(domains, 1000) //1000条
		var countFor int = 0
		for _, d := range domainGroup {
			countFor++
			log.Println("域名分流== ", countFor, "/", len(domainGroup), streamDomain.Interface, streamDomain.Tag, " 正在导出整理 .... ")
			domain := strings.Join(d, ",")
			line = line + "id=" + strconv.Itoa(countFor) + " enabled=yes comment=IKUAI_BYPASS_" + streamDomain.Tag + " domain=" + domain + " interface=" + streamDomain.Interface + " src_addr=" + streamDomain.SrcAddr + " week=1234567 time=00:00-23:59"
			line = line + "\n"
		}

	}
	//write line to file /tmp/domain.txt
	fileName := "/stream_domain_" + time.Now().Format("20060102150405") + ".text"
	err := WriteFile(*config.ExportPath+fileName, line)
	if err != nil {
		log.Println("WriteFile== ", err)
		return
	}
	log.Println("===============================================================")
	log.Println("域名分流== 导出成功", *config.ExportPath+fileName)
	log.Println("可能会有id冲突覆盖你的原记录，请注意备份")
	log.Println("最好删除你的旧记录后再导入")
	log.Println("爱快内：留空分流>域名分流>导入 ")
	log.Println("===============================================================")

}

func WriteFile(fileName string, content string) (err error) {
	f, err := os.OpenFile(fileName, os.O_WRONLY|os.O_CREATE|os.O_TRUNC, 0666)
	if err != nil {
		log.Println("WriteFile== ", err)
		return
	}
	defer func(f *os.File) {
		_ = f.Close()
	}(f)
	_, err = f.WriteString(content)
	if err != nil {
		log.Println("WriteFile== ", err)
		return
	}
	return nil
}
