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
	IkuaiURL        string        `yaml:"ikuai-url" json:"ikuai-url"`
	Username        string        `yaml:"username" json:"username"`
	Password        string        `yaml:"password" json:"password"`
	Cron            string        `yaml:"cron" json:"cron"`
	AddErrRetryWait time.Duration `yaml:"AddErrRetryWait" json:"AddErrRetryWait"`
	AddWait         time.Duration `yaml:"AddWait" json:"AddWait"`
	CustomIsp       []struct {
		Name string `yaml:"name" json:"name"`
		URL  string `yaml:"url" json:"url"`
		Tag  string `yaml:"tag" json:"tag"`
	} `yaml:"custom-isp" json:"custom-isp"`
	StreamDomain []struct {
		Interface string `yaml:"interface" json:"interface"`
		SrcAddr   string `yaml:"src-addr" json:"src-addr"`
		URL       string `yaml:"url" json:"url"`
		Tag       string `yaml:"tag" json:"tag"`
	} `yaml:"stream-domain" json:"stream-domain"`
	IpGroup []struct {
		Name string `yaml:"name" json:"name"`
		URL  string `yaml:"url" json:"url"`
	} `yaml:"ip-group" json:"ip-group"`
	Ipv6Group []struct {
		Name string `yaml:"name" json:"name"`
		URL  string `yaml:"url" json:"url"`
	} `yaml:"ipv6-group" json:"ipv6-group"`
	StreamIpPort []struct {
		Type      string `yaml:"type" json:"type"`
		Interface string `yaml:"interface" json:"interface"`
		Nexthop   string `yaml:"nexthop" json:"nexthop"`
		SrcAddr   string `yaml:"src-addr" json:"src-addr"`
		IpGroup   string `yaml:"ip-group" json:"ip-group"`
		Mode      int    `yaml:"mode" json:"mode"`
		IfaceBand int    `yaml:"ifaceband" json:"ifaceband"`
	} `yaml:"stream-ipport" json:"stream-ipport"`
	WebUI        WebUIConfig   `yaml:"webui" json:"webui"`
}

type WebUIConfig struct {
	Port   string `yaml:"port" json:"port"`     // webui 端口
	User   string `yaml:"user" json:"user"`     // webui 用户名
	Pass   string `yaml:"pass" json:"pass"`     // webui 密码
	Enable bool   `yaml:"enable" json:"enable"` // 是否启用 WebUI 服务
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

// Save 将配置保存到指定文件
// 包含安全校验：
// 1. 强制检查文件后缀名必须为 .yml 或 .yaml
// 2. 通过 yaml.Marshal 重新序列化结构体，避免注入攻击
func Save(filename string, cfg *Config) error {
	// 1. 安全校验：文件后缀
	ext := ""
	if len(filename) > 4 {
		ext = filename[len(filename)-4:]
	}
	if len(filename) > 5 && filename[len(filename)-5:] == ".yaml" {
		ext = ".yaml"
	}
	
	if ext != ".yml" && ext != ".yaml" {
		return fmt.Errorf("security violation: file extension must be .yml or .yaml")
	}

	// 2. 序列化 (清除可能的恶意脚本注入，强制转为合法的YAML格式)
	data, err := yaml.Marshal(cfg)
	if err != nil {
		return fmt.Errorf("marshal config failed: %v", err)
	}

	// 3. 写入文件
	// 使用 0644 权限，避免赋予执行权限
	err = os.WriteFile(filename, data, 0644)
	if err != nil {
		return fmt.Errorf("write file failed: %v", err)
	}

	return nil
}
