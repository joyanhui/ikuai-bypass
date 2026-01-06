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
// 2. 通过 yaml.Node 树操作注入注释说明，解决 WebUI 保存无注释的问题
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

	// 2. 使用 Node 树注入注释
	var node yaml.Node
	if err := node.Encode(cfg); err != nil {
		return fmt.Errorf("marshal config failed: %v", err)
	}

	if node.Kind == yaml.DocumentNode && len(node.Content) > 0 {
		doc := node.Content[0]
		doc.HeadComment = " iKuai Bypass 配置文件\n 详情参考: https://github.com/joyanhui/ikuai-bypass"
		addCommentsToNode(doc)
	}

	// 序列化
	data, err := yaml.Marshal(&node)
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

// addCommentsToNode 为 YAML 节点递归添加说明注释
func addCommentsToNode(node *yaml.Node) {
	if node.Kind != yaml.MappingNode {
		return
	}

	// 顶级字段注释映射 (对应 yaml tag)
	topLevelComments := map[string]string{
		"ikuai-url":       "爱快控制台地址，结尾不要加 \"/\"",
		"username":        "爱快登陆用户名",
		"password":        "爱快登陆密码",
		"cron":            "更新周期，例如 0 7 * * *",
		"AddErrRetryWait": "自动重试时间间隔 (10s, 120s)",
		"AddWait":         "规则添加后的反应等待时间",
		"webui":           "WebUI 管理服务设置",
		"custom-isp":      "自定义运营商分流 (IP分流)",
		"stream-domain":   "域名分流 (优先级高于IP分流)",
		"ip-group":        "IP分组 (与端口分流配合使用)",
		"ipv6-group":      "IPv6分组 (与端口分流配合使用)",
		"stream-ipport":   "端口分流 (下一跳网关/外网线路)",
	}

	// WebUI 子项注释
	webuiComments := map[string]string{
		"port":   "WebUI 服务端口",
		"user":   "WebUI 用户名 (留空禁用认证)",
		"pass":   "WebUI 密码",
		"enable": "是否启用 WebUI 服务",
	}

	for i := 0; i < len(node.Content); i += 2 {
		keyNode := node.Content[i]
		if comment, ok := topLevelComments[keyNode.Value]; ok {
			keyNode.LineComment = " " + comment
		}

		// 特殊处理 WebUI 内部
		if keyNode.Value == "webui" && i+1 < len(node.Content) {
			valNode := node.Content[i+1]
			if valNode.Kind == yaml.MappingNode {
				for j := 0; j < len(valNode.Content); j += 2 {
					subKeyNode := valNode.Content[j]
					if subComment, ok := webuiComments[subKeyNode.Value]; ok {
						subKeyNode.LineComment = " " + subComment
					}
				}
			}
		}
	}
}
