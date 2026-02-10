package config

import (
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"ikuai-bypass/pkg/logger"

	"gopkg.in/yaml.v3"
)

var configLogger = logger.NewLogger("CONF:配置中心")

var (
	ConfPath                   = flag.String("c", "./config.yml", "配置文件路径 后缀必须是yml/yaml")
	RunMode                    = flag.String("r", "cron", "运行模式: cron(立即执行并定时), cronAft(仅定时), once/1(执行一次退出), clean(清理规则), web(仅WebUI)")
	IsAcIpgroup                = flag.String("m", "ispdomain", "功能模块: ispdomain(运营商/域名分流), ipgroup(IPv4分组/端口分流), ipv6group(IPv6分组), ii(ispdomain+ipgroup), ip(v4+v6分组), iip(ii+ip)")
	CleanTag                   = flag.String("tag", "", "要清理的分流规则 TagName 或关键词 (cleanAll 清理全部)")
	ExportPath                 = flag.String("exportPath", "/tmp", "域名分流规则导出文件路径")
	IkuaiLoginInfo             = flag.String("login", "", "爱快登陆地址,用户名,密码。优先级比配置文件内的高")
	IsIpGroupNameAddRandomSuff = flag.String("isIpGroupNameAddRandomSuff", "1", "ip分组名称是否增加随机数后缀(仅ip分组模式有效) 1为添加 0不添加")
)

type Config struct {
	IkuaiURL        string        `yaml:"ikuai-url" json:"ikuai-url"`
	Username        string        `yaml:"username" json:"username"`
	Password        string        `yaml:"password" json:"password"`
	Cron            string        `yaml:"cron" json:"cron"`
	AddErrRetryWait time.Duration `yaml:"AddErrRetryWait" json:"AddErrRetryWait"`
	AddWait         time.Duration `yaml:"AddWait" json:"AddWait"`
	GithubProxy     string        `yaml:"github-proxy" json:"github-proxy"` // Github代理加速地址
	CustomIsp       []struct {
		Tag string `yaml:"tag" json:"tag"`
		URL string `yaml:"url" json:"url"`
	} `yaml:"custom-isp" json:"custom-isp"`
	StreamDomain []struct {
		Interface         string `yaml:"interface" json:"interface"`
		SrcAddr           string `yaml:"src-addr" json:"src-addr"`
		SrcAddrOptIpGroup string `yaml:"src-addr-opt-ipgroup" json:"src-addr-opt-ipgroup"`
		URL               string `yaml:"url" json:"url"`
		Tag               string `yaml:"tag" json:"tag"`
	} `yaml:"stream-domain" json:"stream-domain"`
	IpGroup []struct {
		Tag string `yaml:"tag" json:"tag"`
		URL string `yaml:"url" json:"url"`
	} `yaml:"ip-group" json:"ip-group"`
	Ipv6Group []struct {
		Tag string `yaml:"tag" json:"tag"`
		URL string `yaml:"url" json:"url"`
	} `yaml:"ipv6-group" json:"ipv6-group"`
	StreamIpPort []struct {
		OptTagName        string `yaml:"opt-tagname" json:"opt-tagname"`
		Type              string `yaml:"type" json:"type"`
		Interface         string `yaml:"interface" json:"interface"`
		Nexthop           string `yaml:"nexthop" json:"nexthop"`
		SrcAddr           string `yaml:"src-addr" json:"src-addr"`
		SrcAddrOptIpGroup string `yaml:"src-addr-opt-ipgroup" json:"src-addr-opt-ipgroup"`
		IpGroup           string `yaml:"ip-group" json:"ip-group"`
		Mode              int    `yaml:"mode" json:"mode"`
		IfaceBand         int    `yaml:"ifaceband" json:"ifaceband"`
	} `yaml:"stream-ipport" json:"stream-ipport"`
	WebUI                 WebUIConfig                 `yaml:"webui" json:"webui"`
	MaxNumberOfOneRecords MaxNumberOfOneRecordsConfig `yaml:"MaxNumberOfOneRecords" json:"MaxNumberOfOneRecords"`
}

type MaxNumberOfOneRecordsConfig struct {
	Isp    int64 `yaml:"Isp" json:"Isp"`
	Ipv4   int64 `yaml:"Ipv4" json:"Ipv4"`
	Ipv6   int64 `yaml:"Ipv6" json:"Ipv6"`
	Domain int64 `yaml:"Domain" json:"Domain"`
}

type WebUIConfig struct {
	Port         string `yaml:"port" json:"port"`
	User         string `yaml:"user" json:"user"`
	Pass         string `yaml:"pass" json:"pass"`
	Enable       bool   `yaml:"enable" json:"enable"`
	EnableUpdate bool   `yaml:"enable-update" json:"enable-update"` // 是否启用配置文件在线更新功能
	CDNPrefix    string `yaml:"cdn-prefix" json:"cdn-prefix"`
}

var GlobalConfig Config

func Read(filename string) error {
	buf, err := os.ReadFile(filename)
	if err != nil {
		return err
	}

	// 首先解析为原始 map 以便检测旧字段 'name'
	var raw map[string]interface{}
	_ = yaml.Unmarshal(buf, &raw)

	err = yaml.Unmarshal(buf, &GlobalConfig)
	if err != nil {
		return fmt.Errorf("in file %q: %v", filename, err)
	}

	// 检查并提示用户从 'name' 迁移到 'tag'
	migrationNag := func(section string, items []interface{}) {
		for _, item := range items {
			if m, ok := item.(map[string]interface{}); ok {
				if _, hasName := m["name"]; hasName {
					if _, hasTag := m["tag"]; !hasTag {
						configLogger.Warn("CONF:迁移提示", "Section [%s] 发现过时的 'name' 字段, 请修改配置文件统一使用 'tag' 字段", section)
					}
				}
			}
		}
	}

	if customIsp, ok := raw["custom-isp"].([]interface{}); ok {
		migrationNag("custom-isp", customIsp)
	}
	if ipGroup, ok := raw["ip-group"].([]interface{}); ok {
		migrationNag("ip-group", ipGroup)
	}
	if ipv6Group, ok := raw["ipv6-group"].([]interface{}); ok {
		migrationNag("ipv6-group", ipv6Group)
	}

	// 设置默认 CDN 前缀
	if GlobalConfig.WebUI.CDNPrefix == "" {
		GlobalConfig.WebUI.CDNPrefix = "https://cdn.jsdelivr.net/npm"
	}

	// 设置 MaxNumberOfOneRecords 默认值
	if GlobalConfig.MaxNumberOfOneRecords.Isp == 0 {
		GlobalConfig.MaxNumberOfOneRecords.Isp = 5000
	}
	if GlobalConfig.MaxNumberOfOneRecords.Ipv4 == 0 {
		GlobalConfig.MaxNumberOfOneRecords.Ipv4 = 1000
	}
	if GlobalConfig.MaxNumberOfOneRecords.Ipv6 == 0 {
		GlobalConfig.MaxNumberOfOneRecords.Ipv6 = 1000
	}
	if GlobalConfig.MaxNumberOfOneRecords.Domain == 0 {
		GlobalConfig.MaxNumberOfOneRecords.Domain = 1000
	}

	// 检查每个 StreamDomain 的 Tag，如果不存在，则使用 Interface
	for i := range GlobalConfig.StreamDomain {
		if GlobalConfig.StreamDomain[i].Tag == "" {
			configLogger.Info("CONF:默认参数", "Tag is empty for domain streaming, using interface: %s", GlobalConfig.StreamDomain[i].Interface)
			GlobalConfig.StreamDomain[i].Tag = GlobalConfig.StreamDomain[i].Interface
		}
	}

	return nil
}

// TopLevelComments 顶级字段注释映射
var TopLevelComments = map[string]string{
	"ikuai-url":       "爱快控制台地址，结尾不要加 \"/\"",
	"username":        "爱快登陆用户名",
	"password":        "爱快登陆密码",
	"cron":            "更新周期cron表达式，例如 0 7 * * *",
	"AddErrRetryWait": "自动重试时间间隔 (10s, 120s)",
	"AddWait":         "规则添加后的反应等待时间，部分设备性能优先可以增加这个时间",
	"github-proxy":    "Github代理加速地址，例如 https://gh-proxy.com/ (留空不使用) 可以通过bing搜索引擎搜索关键词 ghproxy 获取最新可用的，如果留空确定你的ikuai-bypass有良好的网络环境可以访问github",
	"webui":           "WebUI 管理服务设置",
	"custom-isp":      "自定义运营商分流 (IP分流)",
	"stream-domain":   "域名分流 (优先级高于IP分流)",
	"ip-group":        "IP分组 (与端口分流配合使用)",
	"ipv6-group":      "IPv6分组 (与端口分流配合使用)",
	"stream-ipport":   "端口分流 (下一跳网关/外网线路)",
	"MaxNumberOfOneRecords": "分组和分流规则单条记录最大写入数据量设置",
}

// ItemComments 列表项内部字段注释映射
var ItemComments = map[string]string{
	"type":                 "分流方式：0-外网线路，1-下一跳网关",
	"mode":                 "负载模式：0-新建连接数, 1-源IP, 2-源IP+源端口, 3-源IP+目的IP, 4-源IP+目的IP+目的端口, 5-主备模式",
	"ifaceband":            "线路绑定：0-不勾选，1-勾选",
	"interface":            "分流线路 (如 wan1)",
	"nexthop":              "下一跳网关地址",
	"tag":                  "规则标识名称 (支持中文，系统自动添加 IKB 前缀)",
	"src-addr":             "分流源地址 (IP或范围)",
	"src-addr-opt-ipgroup": "分流源地址标签 (用于匹配爱快中的IP分组) 在设置了src-addr-opt-ipgroup后，src-addr参数会被忽略。多个名字可以逗号分隔",
	"ip-group":             "关联的IP分组名称，多个名字可以逗号",
	"opt-tagname":          "该条规则的 TagName (可选，如果不填写则自动根据其他条件区分)",
}

// WebuiComments WebUI 子项注释
var WebuiComments = map[string]string{
	"port":          "WebUI 服务端口",
	"user":          "WebUI 用户名 (留空禁用认证)",
	"pass":          "WebUI 密码",
	"enable":        "是否启用 WebUI 服务",
	"enable-update": "是否启用配置文件在线更新功能",
	"cdn-prefix":    "CDN 前缀 (例如: https://cdn.jsdelivr.net/npm |  https://cdn.jsdmirror.com/npm（国内）)",
}

// MaxNumberOfOneRecordsComments MaxNumberOfOneRecords 子项注释
var MaxNumberOfOneRecordsComments = map[string]string{
	"Isp":    "自定义运营商IP最大单条写入数 (爱快限制5000，实际本工具可以写入1w+)",
	"Ipv4":   "IPv4分组最大单条写入数 (爱快限制1000，实际本工具可以写入1.5K)",
	"Ipv6":   "IPv6分组最大单条写入数 (爱快限制1000，实际本工具可以写入1.5K)",
	"Domain": "域名分流最大单条写入数 (爱快限制1000，实际本工具可以写入1w+)",
}

// Save 将配置保存到指定文件
func Save(filename string, cfg *Config, withComments bool) error {
	// 1. 安全校验：文件后缀
	ext := strings.ToLower(filepath.Ext(filename))
	if ext != ".yml" && ext != ".yaml" {
		return fmt.Errorf("security violation: file extension must be .yml or .yaml")
	}

	// 2. 安全校验：检查是否为软链接 (防止符号链接攻击)
	info, err := os.Lstat(filename)
	if err == nil {
		// 文件存在，检查是否为 Symlink
		if info.Mode()&os.ModeSymlink != 0 {
			return fmt.Errorf("security violation: cannot write to a symbolic link")
		}
	} else if !os.IsNotExist(err) {
		// 其他错误
		return err
	}

	// 3. 使用 Node 树注入注释
	var node yaml.Node
	if err := node.Encode(cfg); err != nil {
		return fmt.Errorf("marshal config failed: %v", err)
	}

	if withComments {
		var rootNode *yaml.Node
		if node.Kind == yaml.DocumentNode && len(node.Content) > 0 {
			rootNode = node.Content[0]
			node.HeadComment = " iKuai Bypass 配置文件 大部分时候请使用默认设置即可\n 详情参考: https://github.com/joyanhui/ikuai-bypass"
		} else if node.Kind == yaml.MappingNode {
			rootNode = &node
			node.HeadComment = " iKuai Bypass 配置文件 大部分时候请使用默认设置即可\n 详情参考: https://github.com/joyanhui/ikuai-bypass"
		}

		if rootNode != nil {
			addCommentsToNode(rootNode)
		}
	}

	// 序列化
	data, err := yaml.Marshal(&node)
	if err != nil {
		return fmt.Errorf("marshal config failed: %v", err)
	}

	// 4. 写入文件 (权限 0600: 仅所有者可读写)
	err = os.WriteFile(filename, data, 0600)
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

	for i := 0; i < len(node.Content); i += 2 {
		keyNode := node.Content[i]
		valNode := node.Content[i+1]

		if comment, ok := TopLevelComments[keyNode.Value]; ok {
			keyNode.LineComment = comment
		}

		// 处理 WebUI 对象
		if keyNode.Value == "webui" && valNode.Kind == yaml.MappingNode {
			for j := 0; j < len(valNode.Content); j += 2 {
				subKeyNode := valNode.Content[j]
				if subComment, ok := WebuiComments[subKeyNode.Value]; ok {
					subKeyNode.LineComment = subComment
				}
			}
		}

		// 处理 MaxNumberOfOneRecords 对象
		if keyNode.Value == "MaxNumberOfOneRecords" && valNode.Kind == yaml.MappingNode {
			for j := 0; j < len(valNode.Content); j += 2 {
				subKeyNode := valNode.Content[j]
				if subComment, ok := MaxNumberOfOneRecordsComments[subKeyNode.Value]; ok {
					subKeyNode.LineComment = subComment
				}
			}
		}

		// 处理列表项内部字段
		if valNode.Kind == yaml.SequenceNode {
			for _, itemNode := range valNode.Content {
				if itemNode.Kind == yaml.MappingNode {
					for j := 0; j < len(itemNode.Content); j += 2 {
						subKeyNode := itemNode.Content[j]
						if subComment, ok := ItemComments[subKeyNode.Value]; ok {
							subKeyNode.LineComment = subComment
						}
					}
				}
			}
		}
	}
}
