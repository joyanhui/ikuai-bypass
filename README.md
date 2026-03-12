# iKuai Bypass

iKuai Bypass 现已以 **Rust 版本**作为仓库主线，提供两条产品形态：

- `CLI`：完整功能本体，支持 `once / cron / cronAft / clean / web`
- `GUI`：基于 **Tauri v2 + Astro + Bun** 的桌面应用，共享同一套配置助手与运行面板

旧的 Go/Fyne 版本、历史文档、旧发布脚本与旧 CI 已统一归档到 [golang_archive](/home/y/myworkspace/ikuai-bypass/golang_archive)。

## 项目结构

- [core](/home/y/myworkspace/ikuai-bypass/core)：核心业务库，负责配置、iKuai API、更新流程、运行时与日志
- [cli](/home/y/myworkspace/ikuai-bypass/cli)：CLI 与 Web 模式
- [app/frontend](/home/y/myworkspace/ikuai-bypass/app/frontend)：Astro 单页前端，供 WebUI 与 Tauri 共用
- [app/src-tauri](/home/y/myworkspace/ikuai-bypass/app/src-tauri)：Tauri v2 后端
- [config.yml](/home/y/myworkspace/ikuai-bypass/config.yml)：示例配置
- [golang_archive](/home/y/myworkspace/ikuai-bypass/golang_archive)：Go 版本归档

## 开发

安装前端依赖：

```bash
cd app/frontend
bun install
```

运行 CLI：

```bash
cargo run --bin ikb-cli -- -r once -c ./config.yml
```

运行 GUI：

```bash
bash script/dev.sh gui:dev
```

仅调试前端：

```bash
bash script/dev.sh webui:dev
```

## 构建

构建前端静态资源：

```bash
bash script/dev.sh webui:build
```

构建 CLI：

```bash
cargo build --release -p ikb-cli
```

构建 GUI：

```bash
cd app/frontend && bun install && bun run build
cd ../src-tauri && cargo tauri build
```

## 发布

GitHub Actions 已切换为 Rust 主线发布：

- CLI：覆盖旧 `release.py` 对齐的主要平台/架构矩阵
- GUI：支持
  - Windows x86_64
  - Linux x86_64 AppImage
  - macOS x86_64
  - macOS aarch64

发布工作流位于 [.github/workflows/release.yml](/home/y/myworkspace/ikuai-bypass/.github/workflows/release.yml)。

构建矩阵与产物命名说明见 [docs/release.md](/home/y/myworkspace/ikuai-bypass/docs/release.md)。
