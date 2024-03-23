package main

import (
	"fmt"
	"gopkg.in/yaml.v3"
	"log"
	"os"
	"time"
)

var conf struct {
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
}

func readConf(filename string) error {
	buf, err := os.ReadFile(filename)
	if err != nil {
		return err
	}
	err = yaml.Unmarshal(buf, &conf)
	if err != nil {
		return fmt.Errorf("in file %q: %v", filename, err)
	}

	// 检查每个 CustomIsp 的 Tag，如果不存在，则使用 Name
	for i := range conf.CustomIsp {
		if conf.CustomIsp[i].Tag == "" {
			log.Println("运营商分流规则中配置中Tag为空,采用:", conf.CustomIsp[i].Name)
			conf.CustomIsp[i].Tag = conf.CustomIsp[i].Name
		}
	}

	// 检查每个 StreamDomain 的 Tag，如果不存在，则使用 Interface
	for i := range conf.StreamDomain {
		if conf.StreamDomain[i].Tag == "" {
			log.Println("域名分流规则中中Tag为空,采用:", conf.StreamDomain[i].Interface)
			conf.StreamDomain[i].Tag = conf.StreamDomain[i].Interface
		}
	}

	return nil

}
