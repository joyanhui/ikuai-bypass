package config

import (
	"flag"
	"fmt"
	"gopkg.in/yaml.v3"
	"log"
	"os"
	"time"
)

var (
	ConfPath                   = flag.String("c", "./config.yml", "配置文件路径")
	RunMode                    = flag.String("r", "cron", "运行模式，马上执行 或者定时执行 或者执行一次")
	IsAcIpgroup                = flag.String("m", "ispdomain", "ipgroup(启用ip分组和下一条网关模式) 或者 ispdomain(isp和域名分流模式)")
	CleanTag                   = flag.String("tag", "cleanAll", "要清理的分流规则备注名或关键词")
	ExportPath                 = flag.String("exportPath", "/tmp", "域名分流规则导出文件路径")
	IkuaiLoginInfo             = flag.String("login", "", "爱快登陆地址,用户名,密码。优先级比配置文件内的高")
	DelOldRule                 = flag.String("delOldRule", "after", "删除旧规则顺序 after before ")
	IsIpGroupNameAddRandomSuff = flag.String("isIpGroupNameAddRandomSuff", "1", "ip分组名称是否增加随机数后缀(仅ip分组模式有效) 1为添加 0不添加")
)

type Config struct {
	IkuaiURL        string        `yaml:"ikuai-url"`
	Username        string        `yaml:"username"`
	Password        string        `yaml:"password"`
	Cron            string        `yaml:"cron"`
	AddErrRetryWait time.Duration `yaml:"AddErrRetryWait"`
	AddWait         time.Duration `yaml:"AddWait"`
	CustomIsp       []struct {
		Name string `yaml:"name"`
		URL  string `yaml:"url"`
		Tag  string `yaml:"tag"`
	} `yaml:"custom-isp"`
	StreamDomain []struct {
		Interface string `yaml:"interface"`
		SrcAddr   string `yaml:"src-addr"`
		URL       string `yaml:"url"`
		Tag       string `yaml:"tag"`
	} `yaml:"stream-domain"`
	IpGroup []struct {
		Name string `yaml:"name"`
		URL  string `yaml:"url"`
	} `yaml:"ip-group"`
	Ipv6Group []struct {
		Name string `yaml:"name"`
		URL  string `yaml:"url"`
	} `yaml:"ipv6-group"`
	StreamIpPort []struct {
		Type      string `yaml:"type"`
		Interface string `yaml:"interface"`
		Nexthop   string `yaml:"nexthop"`
		SrcAddr   string `yaml:"src-addr"`
		IpGroup   string `yaml:"ip-group"`
		Mode      int    `yaml:"mode"`
		IfaceBand int    `yaml:"ifaceband"`
	} `yaml:"stream-ipport"`
}

var GlobalConfig Config

func Read(filename string) error {
	buf, err := os.ReadFile(filename)
	if err != nil {
		return err
	}
	err = yaml.Unmarshal(buf, &GlobalConfig)
	if err != nil {
		return fmt.Errorf("in file %q: %v", filename, err)
	}

	// 检查每个 CustomIsp 的 Tag，如果不存在，则使用 Name
	for i := range GlobalConfig.CustomIsp {
		if GlobalConfig.CustomIsp[i].Tag == "" {
			log.Println("运营商分流规则中配置中Tag为空,采用:", GlobalConfig.CustomIsp[i].Name)
			GlobalConfig.CustomIsp[i].Tag = GlobalConfig.CustomIsp[i].Name
		}
	}

	// 检查每个 StreamDomain 的 Tag，如果不存在，则使用 Interface
	for i := range GlobalConfig.StreamDomain {
		if GlobalConfig.StreamDomain[i].Tag == "" {
			log.Println("域名分流规则中中Tag为空,采用:", GlobalConfig.StreamDomain[i].Interface)
			GlobalConfig.StreamDomain[i].Tag = GlobalConfig.StreamDomain[i].Interface
		}
	}

	return nil
}
