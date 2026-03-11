package webui

import (
	"context"
	"crypto/subtle"
	_ "embed"
	"encoding/json"
	"errors"
	"fmt"
	"net"
	"net/http"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"ikuai-bypass/pkg/config"
	"ikuai-bypass/pkg/logger"
)

//go:embed webui.html
var htmlContent []byte

//go:embed favicon.ico
var faviconContent []byte

// ShouldStartWebUI 判断是否应该启动 WebUI 服务
func ShouldStartWebUI() bool {
	return config.GlobalConfig.WebUI.Enable
}

var webLogger = logger.NewLogger("WEB:WebUI服务")

type RunningServer struct {
	Server *http.Server
	URL    string
	Port   string
}

// StartServer 启动 WebUI 服务（阻塞）
func StartServer() {
	port := config.GlobalConfig.WebUI.Port
	if port == "" {
		port = "8080" // 默认端口
	}

	server := createServer(port)

	webLogger.Info("WEB:服务启动", "WebUI is available at http://0.0.0.0:%s", port)
	if config.GlobalConfig.WebUI.User != "" {
		webLogger.Info("AUTH:权限校验", "Basic authentication enabled")
	} else {
		webLogger.Log("AUTH:权限校验", "Warning: Basic authentication is disabled (web-user is empty)")
	}

	if err := server.ListenAndServe(); err != nil {
		if errors.Is(err, http.ErrServerClosed) {
			return
		}
		webLogger.Error("ERR:启动失败", "WebUI Server failed to start, port might be occupied: %v", err)
		os.Exit(1)
	}
}

// StartServerAsync 启动 WebUI 服务（非阻塞） / Start WebUI server non-blocking.
func StartServerAsync(allowAutoPort bool) (*RunningServer, error) {
	port := config.GlobalConfig.WebUI.Port
	if port == "" {
		port = "8080"
	}

	ln, err := net.Listen("tcp", ":"+port)
	if err != nil {
		if !allowAutoPort {
			return nil, err
		}
		ln, err = net.Listen("tcp", ":0")
		if err != nil {
			return nil, err
		}
		_, p, splitErr := net.SplitHostPort(ln.Addr().String())
		if splitErr == nil {
			port = p
		}
		webLogger.Warn("WEB:端口调整", "WebUI port is occupied, using random port: %s", port)
	}

	server := createServer(port)
	rs := &RunningServer{Server: server, Port: port, URL: "http://127.0.0.1:" + port}

	go func() {
		if serveErr := server.Serve(ln); serveErr != nil && !errors.Is(serveErr, http.ErrServerClosed) {
			webLogger.Error("ERR:服务异常", "WebUI Server stopped unexpectedly: %v", serveErr)
		}
	}()

	webLogger.Info("WEB:服务启动", "WebUI is available at %s", rs.URL)
	return rs, nil
}

func (rs *RunningServer) Shutdown(ctx context.Context) error {
	if rs == nil || rs.Server == nil {
		return nil
	}
	return rs.Server.Shutdown(ctx)
}
func OnDemandStartUpWebUI() {
	if ShouldStartWebUI() {
		StartServer()
	} else {
		webLogger.Info("WEB:服务状态", "WebUI mode is disabled")
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
			ExePath                       string            `json:"exe_path"`
			ConfPath                      string            `json:"conf_path"`
			TopLevelComments              map[string]string `json:"top_level_comments"`
			ItemComments                  map[string]string `json:"item_comments"`
			WebuiComments                 map[string]string `json:"webui_comments"`
			MaxNumberOfOneRecordsComments map[string]string `json:"max_number_of_one_records_comments"`
		}{
			Config:                        config.GlobalConfig,
			ExePath:                       exePath,
			ConfPath:                      confPath,
			TopLevelComments:              config.TopLevelComments,
			ItemComments:                  config.ItemComments,
			WebuiComments:                 config.WebuiComments,
			MaxNumberOfOneRecordsComments: config.MaxNumberOfOneRecordsComments,
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
			webLogger.Error("CONF:保存配置", "Failed to save configuration: %v", err)
			http.Error(w, "Failed to save config: "+err.Error(), http.StatusInternalServerError)
			return
		}

		// 更新内存中的全局配置
		config.GlobalConfig = req.Config
		if runtimeController != nil {
			runtimeController.SetDefaults("", config.GlobalConfig.Cron)
		}
		webLogger.Success("CONF:保存配置", "Configuration saved to %s by WebUI (with_comments: %v)", savePath, req.WithComments)

		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{"status": "success", "message": "Configuration saved successfully"}`))
	})

	mux.HandleFunc("/api/runtime/status", func(w http.ResponseWriter, r *http.Request) {
		if runtimeController == nil {
			http.NotFound(w, r)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(runtimeController.Status())
	})

	mux.HandleFunc("/api/runtime/run-once", func(w http.ResponseWriter, r *http.Request) {
		if runtimeController == nil {
			http.NotFound(w, r)
			return
		}
		if r.Method != http.MethodPost {
			http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
			return
		}
		var req struct {
			Module string `json:"module"`
		}
		_ = json.NewDecoder(r.Body).Decode(&req)
		started, err := runtimeController.StartRunOnce(req.Module)
		if err != nil {
			http.Error(w, "Failed to start run: "+err.Error(), http.StatusInternalServerError)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(map[string]any{"started": started})
	})

	mux.HandleFunc("/api/runtime/cron/start", func(w http.ResponseWriter, r *http.Request) {
		if runtimeController == nil {
			http.NotFound(w, r)
			return
		}
		if r.Method != http.MethodPost {
			http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
			return
		}
		var req struct {
			Expr   string `json:"expr"`
			Module string `json:"module"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, "Invalid JSON: "+err.Error(), http.StatusBadRequest)
			return
		}
		if err := runtimeController.StartCron(req.Expr, req.Module); err != nil {
			http.Error(w, "Failed to start cron: "+err.Error(), http.StatusBadRequest)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{"status":"success"}`))
	})

	mux.HandleFunc("/api/runtime/cron/stop", func(w http.ResponseWriter, r *http.Request) {
		if runtimeController == nil {
			http.NotFound(w, r)
			return
		}
		if r.Method != http.MethodPost {
			http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
			return
		}
		if err := runtimeController.StopCron(); err != nil {
			http.Error(w, "Failed to stop cron: "+err.Error(), http.StatusInternalServerError)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{"status":"success"}`))
	})

	mux.HandleFunc("/api/runtime/logs", func(w http.ResponseWriter, r *http.Request) {
		if runtimeController == nil {
			http.NotFound(w, r)
			return
		}
		tail := 200
		if raw := r.URL.Query().Get("tail"); raw != "" {
			if n, err := strconv.Atoi(raw); err == nil && n > 0 {
				tail = n
			}
		}
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(runtimeController.TailLogs(tail))
	})

	mux.HandleFunc("/api/runtime/logs/stream", func(w http.ResponseWriter, r *http.Request) {
		if runtimeController == nil {
			http.NotFound(w, r)
			return
		}
		flusher, ok := w.(http.Flusher)
		if !ok {
			http.Error(w, "Streaming unsupported", http.StatusInternalServerError)
			return
		}
		w.Header().Set("Content-Type", "text/event-stream")
		w.Header().Set("Cache-Control", "no-cache")
		w.Header().Set("Connection", "keep-alive")

		ch := runtimeController.SubscribeLogs(r.Context(), 200)
		fmt.Fprint(w, "retry: 1000\n\n")
		flusher.Flush()
		for {
			select {
			case <-r.Context().Done():
				return
			case entry, ok := <-ch:
				if !ok {
					return
				}
				b, _ := json.Marshal(entry)
				fmt.Fprintf(w, "data: %s\n\n", string(b))
				flusher.Flush()
			}
		}
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
