package webui

import (
	"crypto/subtle"
	_ "embed"
	"encoding/json"
	"log"
	"net/http"
	"time"

	"github.com/dscao/ikuai-bypass/pkg/config"
)

//go:embed webui.html
var htmlContent []byte

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

	log.Printf("WebUI Server started on http://0.0.0.0:%s", port)
	if config.GlobalConfig.WebUI.User != "" {
		log.Println("Basic Auth enabled")
	} else {
		log.Println("Warning: Basic Auth is disabled (web-user is empty)")
	}

	if err := server.ListenAndServe(); err != nil {
		log.Fatalf("WebUI Server failed: %v", err)
	}
}
func IsAndStartWebUI() {
	if ShouldStartWebUI() {
		StartServer()
	} else {
		log.Println("WebUI 模式未启用")
	}
}

// StartServerAsync 异步启动 WebUI 服务（非阻塞）
func StartServerAsync() {
	if !ShouldStartWebUI() {
		log.Println("WebUI is disabled in config, skipping startup")
		return
	}

	port := config.GlobalConfig.WebUI.Port
	if port == "" {
		port = "8080" // 默认端口
	}

	server := createServer(port)

	log.Printf("WebUI Server started on http://0.0.0.0:%s", port)
	if config.GlobalConfig.WebUI.User != "" {
		log.Println("Basic Auth enabled")
	} else {
		log.Println("Warning: Basic Auth is disabled (web-user is empty)")
	}

	go func() {
		if err := server.ListenAndServe(); err != nil {
			log.Printf("WebUI Server failed: %v", err)
		}
	}()
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
		w.Write(htmlContent)
	})

	// API: 获取配置
	mux.HandleFunc("/api/config", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(config.GlobalConfig)
	})

	// API: 保存配置
	mux.HandleFunc("/api/save", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
			return
		}

		var newConfig config.Config
		if err := json.NewDecoder(r.Body).Decode(&newConfig); err != nil {
			http.Error(w, "Invalid JSON: "+err.Error(), http.StatusBadRequest)
			return
		}

		// 保存到当前使用的配置文件路径
		savePath := *config.ConfPath
		if savePath == "" {
			savePath = "config.yml"
		}

		// 使用 config.Save 进行安全保存
		if err := config.Save(savePath, &newConfig); err != nil {
			log.Printf("Save config failed: %v", err)
			http.Error(w, "Failed to save config: "+err.Error(), http.StatusInternalServerError)
			return
		}

		// 更新内存中的全局配置
		config.GlobalConfig = newConfig
		log.Printf("Configuration saved to %s by WebUI", savePath)

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
