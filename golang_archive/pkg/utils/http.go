package utils

import (
	"errors"
	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/logger"
	"io"
	"net/http"
	"strings"
)

// GetFullUrl 根据配置的 GithubProxy 转换 URL
func GetFullUrl(originalURL string) (string, string) {
	proxy := config.GlobalConfig.GithubProxy
	if proxy == "" {
		return originalURL, ""
	}

	// 只有目标地址是以 https://raw.githubusercontent.com/ 或 https://github.com/ 开头才使用代理
	if !strings.HasPrefix(originalURL, "https://raw.githubusercontent.com/") && !strings.HasPrefix(originalURL, "https://github.com/") {
		return originalURL, ""
	}

	// 确保代理地址以 / 结尾
	proxyWithSlash := proxy
	if !strings.HasSuffix(proxyWithSlash, "/") {
		proxyWithSlash += "/"
	}

	return proxyWithSlash + originalURL, proxy
}

// HttpGet 封装 HTTP GET 请求，处理 GitHub 代理和日志记录
func HttpGet(l *logger.Logger, originalURL string) ([]byte, error) {
	fullURL, proxy := GetFullUrl(originalURL)
	if proxy != "" {
		l.Info("HTTP:资源下载", "http.get '%s' proxy '%s'", originalURL, proxy)
	} else {
		l.Info("HTTP:资源下载", "http.get '%s'", originalURL)
	}

	resp, err := http.Get(fullURL)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != 200 {
		return nil, errors.New(resp.Status)
	}

	return io.ReadAll(resp.Body)
}
