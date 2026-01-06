package webui

import (
	"crypto/subtle"
	_ "embed"
	"encoding/json"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/dscao/ikuai-bypass/pkg/config"
)

//go:embed webui.html
var htmlContent []byte

//go:embed favicon.ico
var faviconContent []byte

// ShouldStartWebUI 判断是否应该启动 WebUI 服务
func ShouldStartWebUI() bool {
	return config.GlobalConfig.WebUI.Enable
}

// StartServer 启动 WebUI 服务（阻塞）
func StartServer() {
	port := config.GlobalConfig.WebUI.Port
	if port == "" {
		port = "8080" // 默认端口
	}

	server := createServer(port)

	log.Printf("WebUI 请访问  http://0.0.0.0:%s 注意ip要更换成实际ip地址", port)
	if config.GlobalConfig.WebUI.User != "" {
		log.Println("WebUI Basic 认证 enabled")
	} else {
		log.Println("Warning: WebUI Basic 认证 is disabled (web-user is empty)")
	}

	if err := server.ListenAndServe(); err != nil {
		log.Fatalf("WebUI Server 启动失败 是不是端口被占用了: %v", err)
	}
}
func IsAndStartWebUI() {
	if ShouldStartWebUI() {
		StartServer()
	} else {
		log.Println("WebUI 模式未启用")
	}
}

// createServer 创建并配置 HTTP 服务器
func createServer(port string) *http.Server {
	mux := http.NewServeMux()

	// 页面处理器
	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/" {
			http.NotFound(w, r)
			return
		}
		w.Header().Set("Content-Type", "text/html; charset=utf-8")

		cdnPrefix := config.GlobalConfig.WebUI.CDNPrefix
		if cdnPrefix == "" {
			cdnPrefix = "https://cdn.jsdelivr.net/npm"
		}

		content := string(htmlContent)
		// 简单的模板替换
		content = strings.ReplaceAll(content, "{{CDN_PREFIX}}", cdnPrefix)

		w.Write([]byte(content))
	})

	// Favicon 处理器
	mux.HandleFunc("/favicon.ico", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "image/x-icon")
		w.Write(faviconContent)
	})

	// API: 获取配置
	mux.HandleFunc("/api/config", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
			return
		}

		exePath, _ := os.Executable()
		confPath, _ := filepath.Abs(*config.ConfPath)
		resp := struct {
			config.Config
			ExePath          string            `json:"exe_path"`
			ConfPath         string            `json:"conf_path"`
			TopLevelComments map[string]string `json:"top_level_comments"`
			ItemComments     map[string]string `json:"item_comments"`
			WebuiComments    map[string]string `json:"webui_comments"`
		}{
			Config:           config.GlobalConfig,
			ExePath:          exePath,
			ConfPath:         confPath,
			TopLevelComments: config.TopLevelComments,
			ItemComments:     config.ItemComments,
			WebuiComments:    config.WebuiComments,
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(resp)
	})

	// API: 保存配置
	/*
	   1. 严格的文件后缀白名单：强制检查文件扩展名必须为 .yml 或 .yaml（忽略大小写）。这直接阻止了覆盖或创建系统关键文件（如 /etc/passwd, /bin/sh）或可执行脚本（.sh, .py, .bat）。
	   2. 防御符号链接攻击 (Symlink Attack)：在写入前使用 os.Lstat 检查目标路径。如果目标是一个符号链接，程序将直接拒绝写入。这防止了攻击者创建一个指向敏感文件（如
	      /root/.ssh/authorized_keys）的软链接名为 config.yml，从而诱导程序覆盖该文件的风险。
	   3. 内容格式锁定：文件内容通过 yaml.Marshal 生成。这意味着写入的数据严格遵循 YAML 语法结构。即使攻击者试图在配置值中注入 #!/bin/bash 或 import os 等代码，这些内容在 YAML
	      中也只是被视为普通的字符串值，而被引用或转义，无法被操作系统识别为可执行脚本的头部。
	*/
	mux.HandleFunc("/api/save", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
			return
		}

		var req struct {
			config.Config `json:",inline"`
			WithComments  bool `json:"with_comments"`
		}

		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, "Invalid JSON: "+err.Error(), http.StatusBadRequest)
			return
		}

		// 安全校验：是否启用了在线更新功能
		if !config.GlobalConfig.WebUI.EnableUpdate {
			http.Error(w, "Forbidden: Online update is disabled in configuration", http.StatusForbidden)
			return
		}

		// 保存到当前使用的配置文件路径
		savePath := *config.ConfPath
		if savePath == "" {
			savePath = "config.yml"
		}

		// 使用 config.Save 进行安全保存
		if err := config.Save(savePath, &req.Config, req.WithComments); err != nil {
			log.Printf("Save config failed: %v", err)
			http.Error(w, "Failed to save config: "+err.Error(), http.StatusInternalServerError)
			return
		}

		// 更新内存中的全局配置
		config.GlobalConfig = req.Config
		log.Printf("Configuration saved to %s by WebUI (with_comments: %v)", savePath, req.WithComments)

		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{"status": "success", "message": "Configuration saved successfully"}`))
	})

	// 包装 Basic Auth 中间件
	handler := basicAuth(mux)

	return &http.Server{
		Addr:         ":" + port,
		Handler:      handler,
		ReadTimeout:  10 * time.Second,
		WriteTimeout: 10 * time.Second,
	}
}

// basicAuth 简单认证中间件，动态读取配置
func basicAuth(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		username := config.GlobalConfig.WebUI.User
		password := config.GlobalConfig.WebUI.Pass

		// 如果未配置用户名，则跳过认证
		if username == "" {
			next.ServeHTTP(w, r)
			return
		}

		user, pass, ok := r.BasicAuth()
		if !ok || subtle.ConstantTimeCompare([]byte(user), []byte(username)) != 1 || subtle.ConstantTimeCompare([]byte(pass), []byte(password)) != 1 {
			w.Header().Set("WWW-Authenticate", `Basic realm="Restricted"`)
			http.Error(w, "Unauthorized", http.StatusUnauthorized)
			return
		}
		next.ServeHTTP(w, r)
	})
}
