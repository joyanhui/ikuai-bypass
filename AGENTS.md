# AI rules

这是 iKuai Bypass 的 **Rust 主线版本**。仓库根目录即当前可交付版本，旧的 Go/Fyne 代码、文档和旧 CI 已归档到 `golang_archive/`。

**永远用中文和我对话**

## 当前目录结构

- `crates/core/`：核心业务库（配置、iKuai API、更新流程、运行时、日志）
- `apps/cli/`：CLI + Web 模式
- `apps/integration-tests/`：集成测试模块
  其中 `apps/integration-tests/src/ikuai_simulator/` 是 iKuai 真机模拟器目录（CI 默认使用）
- `frontends/app/`：Bun + Astro 单页前端
- `apps/gui/`：Tauri v2 后端
- `config.yml`：示例配置
- `golang_archive/`：Go 版本归档，除非用户明确要求，否则不要把新功能继续做进归档目录

## 集成测试约定

- GitHub CI 默认使用 `apps/integration-tests/src/ikuai_simulator/` 里的 iKuai 模拟器，不再依赖在线 KVM
- 本地进行集成测试时，默认优先使用 KVM/QEMU 真机链路，用于验证和模拟器的行为差异
- 如果本地没有 `qemu-system-x86_64` / `qemu-img` / `/dev/kvm`，允许通过 `IKB_TEST_IKUAI_URL` 连接开发者显式指定的爱快地址继续跑集成测试
- 本地 KVM 默认镜像优先使用仓库内 `.github/ikuai.qcow2.7z` 解压得到的 `.github/ikuai.qcow2`，除非开发者自己通过环境变量覆盖

## 核心业务逻辑

### 1. 规则标识与命名约定

- 名称前缀：`IKB`
- 统一备注：`joyanhui/ikuai-bypass`
- 命名规则：`IKB + tag + 序号`
- 识别逻辑：名字以 `IKB` 开头或备注包含 `joyanhui/ikuai-bypass`
- 旧版本兼容：清理模式保留对 `IKUAI_BYPASS` 的兼容清理

### 2. 执行与日志规范

- 所有更新任务必须严格顺序执行，禁止并发更新多个规则块
- 所有面向用户的日志标签必须使用中文
- API 和内部错误信息保持英文，便于定位

### 3. 配置与编辑模型

- `rawYaml` 是前端配置编辑的唯一真来源
- 可视化编辑必须通过 YAML AST 定点修改 `rawYaml`
- 文本编辑直接编辑 `rawYaml`
- 后端保存需要先解析 YAML，再决定是否按原文保留注释写盘

### 4. 配置一致性要求

- 新增或修改配置项时，至少同步更新：
  - `config.yml`
  - `crates/core/src/config.rs`
  - `frontends/app/src/lib/config_model.ts`
  - `frontends/app` 相关表单 / YAML AST / 保存逻辑
- 统一使用 `tag` 字段作为用户标识，不再新增 `name` 字段语义

### 5. 更新与安全策略

- 原地更新：匹配则 Edit，不匹配则 Add，保持爱快内部 ID 稳定
- 自定义运营商分片：同名 `IKB+tag`，通过备注中的分片序号匹配并清理冗余分片
- Safe-Before：远程资源下载失败或 HTTP 状态异常时，立即终止当前项更新，严禁清理旧规则
- 清理模式：必须显式指定 `-tag`，不得设置危险默认值
- 配置覆写：必须做 YAML 后缀、软链接和写入安全校验

## 技术约束

- CLI 是完整功能本体，GUI/WebUI 只是可视化入口
- WebUI 与 Tauri 共用 `frontends/app/` 这一套 Astro 单页
- Tauri IPC 语义需要和 Web API 对齐
- 前端禁止 `as any` / `@ts-ignore` 绕过类型系统
- 核心逻辑避免无意义 clone、unwrap 和隐式 panic
- 前端monaco 编辑器仅限pc模式可用，在tauri app的移动端禁止使用，会导致webview崩溃
## 注释与文案规范

- 代码注释使用双语文本（中文 + English）
- 注释优先解释为什么存在，再解释做了什么
- UI 返回文案与 API 错误信息保持英文

## CI 约束

`.github/workflows/release.yml` 只允许 `tag push` 和 `workflow_dispatch` 触发，禁止恢复每日定时构建；手动执行时 `publish_release` 与 `push_docker` 默认勾选，未填写 `release_tag` 但勾选发布时必须自动生成 `manual-release-年月日时分秒` 继续发布，且手动执行发布一律标记为 prerelease，选择 `full` 时必须自动包含 nightly MIPS 架构。Tag push 仅在 tag 名包含 `test`、`rc`、`alpha`、`beta`、`pre`、`preview`、`dev`、`nightly` 时发布为 prerelease，否则发布为正式版并推送 Docker `latest`。
