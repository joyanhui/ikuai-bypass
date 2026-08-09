# AGENTS_This.md（ikuai-bypass）

本文件只放项目特有内容，通用规范见同目录 `AGENTS.md`。

## 项目定位
- iKuai Bypass 的 Rust 主线版本：仓库根目录即当前可交付版本，旧的 Go/Fyne 代码、文档和旧 CI 已归档到 `golang_archive/`。
- 除非用户明确要求，不要把新功能继续做进 `golang_archive/` 归档目录。

## 文档事实来源
- `docs/`：Jekyll + GitHub Pages 文档站，部署于 `https://joyanhui.github.io/ikuai-bypass/`（子目录）；本地预览执行 `bash script/dev.sh docs:dev`。
- `api-docs/`：爱快 4.x API 抓包记录。
- `dev-docs/`：专题开发说明，含 `openwrt-luci-ipk构建和说明.md`、运行模式和分流模式配置原则、域名/端口/IPv4/IPv6 分组、添加运营商等。
- docs 内部链接必须使用标准 Markdown 相对路径 `](file.md)` 或 `](file.md#锚点)`，禁止 `](/根路径/)` 和 `]({{ site.baseurl }}/path/)` 等 Liquid 写法；`jekyll-relative-links` 插件构建时自动将 `file.md` 转 `/ikuai-bypass/file/`，同时 Obsidian 原生支持 `.md` 相对路径跳转和图谱。

## 开发环境
- 进入仓库目录后执行 `nix develop`（或 direnv），获得 Rust/前端/Jekyll 开发环境。

## 仓库结构
```text
ikuai-bypass/
├── crates/core/             # 核心业务库（配置、iKuai API、更新流程、运行时、日志）
├── apps/cli/                # CLI + Web 模式（完整功能本体）
├── apps/gui/                # Tauri v2 后端
├── apps/integration-tests/  # 集成测试模块（含 ikuai_simulator/ iKuai 真机模拟器，CI 默认使用）
├── frontends/app/           # Bun + Astro 单页前端（WebUI 与 Tauri 共用）
├── config.yml               # 示例配置
├── api-docs/                # 爱快 4.x API 抓包记录
├── docs/                    # Jekyll + GitHub Pages 文档站
├── dev-docs/                # 专题开发说明
├── packaging/               # 打包相关
└── golang_archive/          # Go 版本归档
```

## 前端技术栈（修正通用 AGENTS.md 的前端规范）
- 本项目前端是 Bun + Astro 4 单页（`frontends/app/`），不是通用 AGENTS.md 的 React Router v7 + Vite 技术栈；通用 AGENTS.md 中 React Router v7、TanStack Query、Zustand、shadcn/ui、Zod、React Hook Form、Lucide、Motion、typesafe-i18n、vite-plugin-checker 等约定不适用。
- 实际技术栈：Astro 4（static output）+ Tailwind v4（@tailwindcss/vite）+ TypeScript + 自定义 i18n（`src/lib/i18n.ts`，zh/en 字典）+ js-yaml/yaml（YAML 解析）+ monaco-editor（懒加载）+ vitest + playwright。
- i18n：用户可见文本必须走 `src/lib/i18n.ts` 字典，禁止硬编码文案；新增词条同步 ZH/EN 两字典。
- 存储：统一走 `src/lib/storage.ts`，禁止组件直接 `setItem` / `getItem`。
- 类型安全沿用通用 AGENTS.md 规则：禁止 `any`、禁止 `@ts-ignore`、禁止 `as any` 绕过类型系统。
- monaco 编辑器仅限 PC 模式可用；Tauri app 移动端禁止使用，会导致 webview 崩溃。
- 单测用 vitest（`src/lib/*.test.ts`），浏览器 e2e 用 playwright。

## 命令防卡死（修正通用 AGENTS.md）
- 通用 AGENTS.md 的 `vite build`（vite-plugin-checker）与 `typesafe-i18n` 卡死约定不适用于本项目：本项目无 vite-plugin-checker 与 typesafe-i18n，前端构建为 `astro build`（`bun run build`），会正常退出。
- 若执行其他不退出进程的命令，不可接 `| tail` / `| head` 等待管道 EOF；截断输出改用 `do sleep x` 或重定向到文件再 tail。

## 配置与编辑模型（修正通用 AGENTS.md 的配置规范）
- 本项目无数据库；配置以 `config.yml`（示例）为事实来源，前端配置编辑唯一真来源是 `rawYaml`。
- 可视化编辑必须通过 YAML AST 定点修改 `rawYaml`（`frontends/app/src/lib/yaml_ast.ts`）；文本编辑直接编辑 `rawYaml`。
- 后端保存必须先解析 YAML 校验，再按 `rawYaml` 原文写盘。
- 配置一致性：新增或修改配置项时，至少同步更新 `config.yml`、`crates/core/src/config.rs`、`frontends/app/src/lib/config_model.ts`、`frontends/app` 相关表单 / YAML AST / 保存逻辑。
- 统一使用 `tag` 字段作为用户标识，不再新增 `name` 字段语义。
- 配置覆写必须做 YAML 后缀、软链接和写入安全校验。

## 通用规范不适用项
- 错误码与 API 规范（OpenAPI 3.0、utoipa、错误码枚举唯一来源、TS 自动生成同步）：本项目是 CLI 为主的 iKuai 分流工具，调用爱快 HTTP API，自身无这套 API 契约体系，通用 AGENTS.md 该节不适用。
- 环境变量规范（`*_env(key, default)` 统一封装、启动时校验）：本项目未采用统一封装，环境变量（如 `IKB_TEST_IKUAI_URL`）按需读取。
- Cloudflare Worker（`bunx wrangler`）：本项目无 Cloudflare Worker，该约定不适用。

## 核心业务逻辑
### 规则标识与命名约定
- 名称前缀：`IKB`；统一备注：`IkuaiBypass`；命名规则：`IKB + tag + 序号`。
- 识别逻辑：名字以 `IKB` 开头或备注包含 `IkuaiBypass`。
- 旧版本兼容：清理/更新模式保留对 `joyanhui/ikuai-bypass` 与 `IKUAI_BYPASS` 的兼容识别。

### 执行与日志规范
- 所有更新任务必须严格顺序执行，禁止并发更新多个规则块。
- 面向用户的日志标签必须使用中文；API 和内部错误信息保持英文，便于定位。

### 更新与安全策略
- 原地更新：匹配则 Edit，不匹配则 Add，保持爱快内部 ID 稳定。
- 自定义运营商分片：同名 `IKB+tag`，通过备注中的分片序号匹配并清理冗余分片。
- Safe-Before：远程资源下载失败或 HTTP 状态异常时，立即终止当前项更新，严禁清理旧规则。
- 清理模式：必须显式指定 `-tag`，不得设置危险默认值。

## 架构约束
- CLI 是完整功能本体，GUI/WebUI 只是可视化入口。
- WebUI 与 Tauri 共用 `frontends/app/` 这一套 Astro 单页；Tauri IPC 语义需要和 Web API 对齐（`frontends/app/src/lib/bridge.ts`）。
- 核心逻辑避免无意义 clone、unwrap 和隐式 panic（通用 AGENTS.md 的 Rust 规范已覆盖更严格约束）。

## 注释与文案规范（修正通用 AGENTS.md）
- 本项目明确要求代码注释使用双语文本（中文 + English），优先解释为什么存在，再解释做了什么；这覆盖通用 AGENTS.md 的"除非明确要求，永远不要新增注释"。
- UI 返回文案与 API 错误信息保持英文。
- 测试代码是项目明确要求（见集成测试约定与前端测试），不受通用 AGENTS.md"不得新增测试代码"限制。

## 集成测试约定
- GitHub CI 默认使用 `apps/integration-tests/src/ikuai_simulator/` 里的 iKuai 模拟器，不再依赖在线 KVM。
- 本地集成测试默认优先使用 KVM/QEMU 真机链路，用于验证和模拟器的行为差异。
- 本地无 `qemu-system-x86_64` / `qemu-img` / `/dev/kvm` 时，允许通过 `IKB_TEST_IKUAI_URL` 连接开发者显式指定的爱快地址继续跑集成测试。
- 本地 KVM 默认镜像优先使用仓库内 `.github/smoke-test-ikuai.qcow2.7z` 解压得到的 `.github/smoke-test-ikuai.qcow2`，除非开发者通过环境变量覆盖。
- `webui` 浏览器 smoke 本地验证必须基于 `nix develop`，先预编译 `ikb-webui-fixture`，再以二进制路径运行 `apps/integration-tests/run-webui-browser-smoke.sh`。

## 安装脚本测试
- `docs/install.sh` 一键安装脚本 CI 测试覆盖 Ubuntu (systemd) 与 OpenWrt (KVM QEMU) 两种环境，验证 OS/arch 检测、版本获取、下载安装、服务文件注册、enable/start/stop/disable 生命周期、保留/删除配置卸载以及进程残留清理。

## CI 约束
- `.github/workflows/release.yml` 只允许 `tag push` 和 `workflow_dispatch` 触发，禁止恢复每日定时构建。
- 手动执行时 `publish_release` 与 `push_docker` 默认勾选；未填写 `release_tag` 但勾选发布时必须自动生成 `manual-release-年月日时分秒` 继续发布；手动执行发布一律标记为 prerelease；选择 `full` 时必须自动包含 nightly MIPS 架构。
- Tag push 仅在 tag 名包含 `test`、`rc`、`alpha`、`beta`、`pre`、`preview`、`dev`、`nightly` 时发布为 prerelease，否则发布为正式版并推送 Docker `latest`。
- 发布 workflow 注意事项：
  - 发布 workflow 运行期间不要在 main 上 push 新 commit；tag 指向的 commit 不再是默认分支 tip 时，GitHub 平台会拒绝 GITHUB_TOKEN 携带该 commit 作为 target_commitish 创建 release（403），该限制要求 PAT 级权限。
  - Publish Release 步骤使用 `gh release create` 而非 softprops/action-gh-release：tag 已存在时只关联 tag、不传 target_commitish，天然规避上述 403。
  - 上传资产只收集 release/ 第一层归档文件；`.zip.stage/` 等打包中间产物内含同名 `ikuai-bypass`/`README.md`/`config.yml`，上传会触发 asset 重名 422。
