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

// StartServer 启动 WebUI 服务
func StartServer() {
	port := config.GlobalConfig.WebPort
	if port == "" {
		port = "8080" // 默认端口
	}

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
		// 使用 Strict decoding 或者是直接 unmarshal
		if err := json.NewDecoder(r.Body).Decode(&newConfig); err != nil {
			http.Error(w, "Invalid JSON: "+err.Error(), http.StatusBadRequest)
			return
		}

		// 可以在这里增加额外的业务逻辑校验

		// 保存到当前使用的配置文件路径
		// 注意：这里我们使用 flag 中的 ConfPath，确保是写入启动时指定的文件
		savePath := *config.ConfPath
		if savePath == "" {
			savePath = "config.yml"
		}

		// 使用 config.Save 进行安全保存（已包含后缀名检查和 YAML 序列化）
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

			// 改为动态读取 GlobalConfig，以便在运行时修改配置后生效（如果需要支持热更新密码）

			// 注意：虽然 StartServer 时端口已定，但用户可能在界面修改了密码并保存，下次请求应使用新密码

			handler := basicAuth(mux)

		

			server := &http.Server{

				Addr:         ":" + port,

				Handler:      handler,

				ReadTimeout:  10 * time.Second,

				WriteTimeout: 10 * time.Second,

			}

		

			log.Printf("WebUI Server started on http://0.0.0.0:%s", port)

			if config.GlobalConfig.WebUser != "" {

				log.Println("Basic Auth enabled")

			} else {

				log.Println("Warning: Basic Auth is disabled (web-user is empty)")

			}

			

			if err := server.ListenAndServe(); err != nil {

				log.Fatalf("WebUI Server failed: %v", err)

			}

		}

		

		// basicAuth 简单认证中间件，动态读取配置

		func basicAuth(next http.Handler) http.Handler {

			return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {

				username := config.GlobalConfig.WebUser

				password := config.GlobalConfig.WebPass

		

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

		
