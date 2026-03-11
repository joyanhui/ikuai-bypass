# AI rules (Rust 版本)

这是 iKuai Bypass 的 Rust 重写版本：**Rust + Tauri v2 + Bun + Astro**。

本项目的核心形态是 **CLI + GUI(桌面/移动)** 双模式：
- **CLI 是完整功能本体**（对齐 Go CLI 的全部 flags/行为/iKuai API）。
- **GUI(App) 只是把 CLI 的完整能力做成可视化界面**（运行控制 + 配置入口 + 日志）。
- 所谓 **WebUI 只是 CLI/GUI 附带的“配置助手/运行面板”能力**，不是独立主产品。

## 语言

- **永远用中文和我对话**。
- 面向用户的日志：**中文标签**（例如 `[LOGIN:登录失败]`）。
- API/内部错误返回：**英文错误信息**（便于定位与跨平台兼容）。

## 技术栈与形态

- `rust/core`：核心业务库（配置、iKuai API、更新流程、运行时、日志）。
- `rust/cli`：CLI + Web 模式（Axum 提供 `/api/*` + SSE + 静态前端 dist）。
- `rust/cli` 必须在没有 WebUI 的情况下也能完成全部任务；WebUI 仅作为附加入口。
- `rust/app/frontend`：**Bun + Astro** 单页前端（WebUI 与 Tauri 共用同一份页面）。
- `rust/app/src-tauri`：Tauri v2 后端（复用 `ikb-core`）。

前端要求：
- **WebUI 与 Tauri 复用同一套 Astro 单页**。
- 在 **Tauri** 下默认呈现类似 Go/Fyne 的“运行面板（运行/定时/日志）”，并提供进入“配置助手”的入口。
- 在“配置助手”视图中，对齐 Go 版本 `pkg/webui/webui.html` 的功能（表单向导、命令生成、YAML 预览/保存、远程配置加载、运行与日志）。

## 核心业务约束（必须对齐 Go 版本）

### 1) 规则识别与命名（关键）

- 名称前缀：`IKB`（所有由本工具创建的规则/分组名都必须以此开头）。
- 统一备注：`joyanhui/ikuai-bypass`（仅包含该常量）。
- 命名规则：`IKB + tag + 序号`（严禁随意更改模板）。
- 识别逻辑：名字以 `IKB` 开头或备注包含 `joyanhui/ikuai-bypass`。
- 旧版兼容：清理模式保留对 `IKUAI_BYPASS` 旧备注的兼容。
- tagname 长度限制：爱快 v4 对 tagname 有 **15 字符**限制（`IKB` 前缀占 3）。

### 2) 顺序执行（禁止并发核心块）

- 所有更新任务必须 **严格顺序执行**。
- 核心更新流程中禁止并发执行多个模块（避免日志交织与 iKuai API 高并发失败）。

### 3) Safe-Before（故障停机保护）

- 远程资源下载失败或 HTTP 状态异常时：必须立即终止当前项更新。
- **严禁在失败情况下执行删除/清理**，确保旧规则持续有效。

### 4) 配置路径与安全

- 配置路径：必须由 `-c` 指定；否则使用符合 **XDG** 且适配移动端的默认路径。
- **永远不使用环境变量** 作为配置来源。
- 保存配置必须做：后缀白名单（.yml/.yaml）+ 软链接校验 + 写入权限控制（类 Unix 0600）。

## CLI / Web / App 对齐

- CLI flags 与 Go CLI 对齐：`-c -r -m -tag -login -exportPath -isIpGroupNameAddRandomSuff`。
- run-mode 对齐：`web / cron / cronAft / once|nocron|1 / clean`。
- Web 模式提供与 Go WebUI 相同的接口语义：
  - `GET /api/config`（含注释映射）
  - `POST /api/save`（支持 `with_comments`）
  - runtime 控制与日志：`/api/runtime/*` + SSE `/api/runtime/logs/stream`
- Tauri IPC commands 与 Web API 语义保持一一对应；日志用事件推送（如 `ikb://log`）。

## 变更纪律

- 修改/新增配置字段时：确保 Rust 的配置结构、默认值与前端表单/生成逻辑同步更新。
- 不允许通过类型绕过（例如 `as any` / `@ts-ignore` 类行为）。
- 不允许在核心逻辑引入并发执行多个模块。

## Rust 开发规范

禁止隐式 panic、无意义 clone 与 unwrap；前端禁止 `as any` 与 `@ts-ignore` 绕过类型检查。
