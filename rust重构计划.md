# Rust 重构计划（Tauri v2 + Bun + Astro）

> 目标：在不破坏现有 Go 版本可用性的前提下，启动一个 **Rust 实现**，并最终做到：
> - **CLI 端**：完整复刻现有 Go CLI 的功能与行为（参数、默认值、安全策略、日志风格）。
> - **App 端（PC + Mobile）**：基于 **Tauri v2**，内置配置向导与运行控制，覆盖 WebUI 的全部能力。
> - **配置文件**：对现有 `config.yml` / `config_example.yml` 的字段与语义保持 **100% 兼容**（含迁移提示、默认值、保存安全策略、尽量保留注释体验）。

---

## 0. 约束与非目标

### 硬约束（必须满足）

1. **配置兼容性**：
   - YAML 字段名、含义、默认值与 Go 版本对齐（`pkg/config/config.go`）。
   - 继续支持旧字段迁移提示（如 `name -> tag` 的迁移提示）。
   - 保存配置时必须保留 Go 版本的安全约束：仅允许 `.yml/.yaml`、拒绝 symlink、权限 `0600`。

2. **核心业务不变**（来自项目约定/AGENTS.md）：
   - 规则识别：名称前缀 `IKB`、统一备注 `joyanhui/ikuai-bypass`、命名模版 `IKB + tag + 序号`。
   - 顺序执行：核心更新任务必须严格顺序执行（避免日志交织、避免 iKuai 高并发导致失败）。
   - Safe-Before：远程下载失败/HTTP 异常时，该模块必须立即停止，禁止清理/删除旧规则。
   - 清理模式：必须显式指定 `-tag`（或 `cleanAll`），严禁默认值。
   - 日志：面向用户标签中文；底层错误细节英文；Web/API 错误信息英文。

3. **接口能力对齐**：
   - 提供与现有 WebUI 类似的能力集合：读取配置、保存配置、运行一次、启动/停止 cron、日志 tail/实时流。

### 非目标（阶段性不做）

- 不追求与现有 `pkg/webui/webui.html` 前端实现细节一致（将使用 **Bun + Astro** 重写向导）。
- iOS/Android 后台 cron 保活策略不在本项目解决范围（用户自行处理保活）。

---

## 1. 总体架构（增量迁移，Go 与 Rust 并存）

为避免破坏现有 Go 工程、CI、发布流程，Rust 实现建议放在仓库子目录 `rust/`，使用 Cargo Workspace 管理：

```text
rust/
  Cargo.toml (workspace)
  core/         # 纯业务核心库：配置、iKuai API、模块更新、日志 broker、调度
  cli/          # Rust CLI：对齐 Go flags 与运行模式，支持 headless
  app/          # Tauri v2 + Astro UI（与 CLI 共享 core）
```

### crate 拆分原则

- `core`：不依赖任何 UI/平台特性，保证可复用、可测试。
- `cli`：只做参数解析、运行模式编排、可选 web 服务托管静态 UI。
- `app`：Tauri 壳 + 前端；通过调用 `core` 暴露的能力完成配置/运行控制。

---

## 2. 功能对齐清单（Rust 必须复刻的行为）

### 2.1 CLI Flags/运行模式对齐

对齐 Go 侧（`pkg/config/config.go`）定义的核心参数：

- `-c` 配置路径（校验后缀 yml/yaml）。
- `-r` 运行模式：`cron`, `cronAft`, `once/1`, `clean`, `web`。
- `-m` 功能模块：`ispdomain`, `ipgroup`, `ipv6group`, `ii`, `ip`, `iip`。
- `-tag` 清理标签（必填；`cleanAll` 特判）。
- `-login` 覆盖登录信息。
- 其他：`github-proxy`, `isIpGroupNameAddRandomSuff` 等。

验收标准：Rust CLI 在同一份 `config.yml` 下，输出日志、运行路径与 Go 版本一致（允许时间戳等差异）。

### 2.2 配置读取与默认值

必须对齐 Go 的默认值策略（示例）：

- `webui.cdn-prefix` 默认 `https://cdn.jsdelivr.net/npm`。
- `MaxNumberOfOneRecords` 默认值：Isp=5000, Ipv4=1000, Ipv6=1000, Domain=5000。
- `stream-domain` 中 tag 为空时使用 interface 作为 tag。

### 2.3 配置保存安全策略

保存必须满足：

1. 仅允许 `.yml/.yaml`；
2. 禁止写入 symlink；
3. 写入权限 `0600`；
4. 保存来源：App/Web 触发保存时，若启用了“在线更新开关”，才允许写盘（对齐 Go 的 `webui.enable-update` 语义）。

### 2.4 iKuai API 交互与模块更新

Rust 侧需要对齐 Go `pkg/ikuai_api4/` 下的能力边界（不要求逐行复刻，但行为要一致）：

- 登录与会话（Cookie）。
- 自定义运营商（custom isp）同步。
- IPv4 分组同步。
- IPv6 分组同步。
- 域名分流同步。
- 端口分流同步。
- 清理/匹配逻辑（IKB 前缀、备注、兼容旧 IKUAI_BYPASS 规则）。

### 2.5 日志与运行时控制

Rust 侧需要提供等价的运行时控制能力：

- run-once（互斥：同一时刻只能跑一个更新任务）。
- cron start/stop（同一时刻只能启一个 cron）。
- tail logs + subscribe logs（用于 UI 实时展示）。

---

## 3. 前端重构（Bun + Astro）策略

### 3.1 目标

- 用 Astro 组件化重写“设置向导/配置编辑器/参数生成器/运行控制台”。
- UI 需适配 PC + 移动端（响应式布局、表单友好）。

### 3.2 API 抽象（同一前端支持 CLI-Web 与 Tauri-App）

建议做一层 `apiClient` 适配：

- **在 Tauri 环境**：优先走 `tauri::command`（避免本地端口、减少安全面）。
- **在 CLI Web 环境**：走 HTTP JSON API（与 Go WebUI 的 `/api/*` 对齐或提供兼容层）。

验收标准：同一套前端代码可在两种运行形态下工作：

1) `ikuai-bypass-rs -r web`（浏览器访问）；2) Tauri App 内嵌窗口。

---

## 4. YAML 注释“保真”策略（分阶段交付）

现实约束：Rust 生态的 `serde_yaml` 在“保留原文件注释/格式”方面能力有限。

### 阶段 A（先可用）

- 先实现：严格字段兼容 + 安全写入 + 可选注释注入（以模板方式写出“规范化 YAML”）。
- 保证：写出来的 YAML 能被 Go 版本读，并保持语义一致。

### 阶段 B（体验对齐）

- 增强：尽量保留用户原 YAML 的注释与顺序。
- 可能路线：
  - 采用 YAML AST 解析器实现“结构化 merge + 注释注入”；或
  - 维护一份“官方模板 YAML（带注释）”，对用户配置做字段 merge，并输出带注释的规范文件。

验收标准：

- 不丢字段、不改变语义；
- 注释保真以“可读性提升”为目标，不把它作为阻塞核心功能的硬门槛。

---

## 5. 里程碑与交付节奏

### Milestone 0：脚手架与边界冻结

- 建立 `rust/` workspace。
- 产出：核心对齐清单（本文件）+ 最小可编译的 `core/cli`。

### Milestone 1：配置层对齐（最关键）

- Rust `Config` 结构体与 Go `config.Config` 字段对齐。
- 读取 `config.yml` + 默认值补齐 + 写回（安全约束）。
- 单元测试：golden config（多份真实样例）。

### Milestone 2：iKuai API 客户端与模块同步

- 逐模块实现：custom isp / ip-group / ipv6-group / stream-domain / stream-ipport。
- 实现顺序执行与 Safe-Before。
- 引入模拟服务器做集成测试（mock iKuai endpoints）。

### Milestone 3：Rust CLI 行为对齐

- flags/模式对齐。
- cron/run-once/clean 对齐。
- 日志格式与中英文策略对齐。

### Milestone 4：Tauri v2 App + Astro UI

- App 端基本交互：加载配置、编辑保存、触发 run-once、查看日志、启停 cron。
- 响应式适配移动端。

### Milestone 5：发布与回归验证

- 桌面端（Win/macOS/Linux）打包。
- Android 打包验证。
- 与 Go 版本对照测试（同样配置与输入）。

---

## 6. 验收标准（Definition of Done）

1. 在同一份 `config.yml` 下：Rust CLI 的更新结果与 Go CLI 一致（规则命名、备注、数量、更新策略一致）。
2. 清理模式与 Safe-Before 策略无回归。
3. 配置保存不破坏安全约束；能被 Go 版正常读取。
4. App 与 CLI Web 复用同一套 Astro UI。
5. 基础测试通过：`cargo test`（Rust）与 `go test ./...`（Go）同时可跑。

---

## 7. 开发开始点（本分支要做什么）

在 `rust-dev` 分支上优先落地：

1) `rust/` workspace + `core/cli` 最小可编译；
2) 配置层（读取/默认值/安全保存）的 POC + 测试；
3) 定义与 Go WebUI 兼容的最小 API 形状（先 stub，为后续 Astro/Tauri 接入做准备）。
4. **永远不使用环境变量**：
   - Rust 实现不得通过环境变量影响行为（例如：不得使用 `XDG_CONFIG_HOME`、`IKB_CONFIG_PATH` 等）。
   - 默认配置路径按平台“标准位置”计算：Linux 采用 `~/.config`（不读取 XDG 覆盖变量），macOS 采用 `~/Library/Application Support`，Windows 采用 RoamingAppData（KnownFolder API）。
