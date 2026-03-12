package config

import "os"

func DefaultConfigPath() string {
	if base, err := os.UserConfigDir(); err == nil && base != "" {
		return base + string(os.PathSeparator) + "ikuai-bypass" + string(os.PathSeparator) + "config.yml"
	}
	return "./config.yml"
}
