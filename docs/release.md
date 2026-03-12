# Release

当前仓库的发布主线为 Rust 版本，GitHub Actions 位于 [.github/workflows/release.yml](/home/y/myworkspace/ikuai-bypass/.github/workflows/release.yml)。

## 触发方式

- `push tags`：推送 `v*` tag 时自动构建并发布
- `workflow_dispatch`：支持手动触发

手动触发支持以下输入：

- `release_tag`：可留空；留空时自动生成 `manually_年月日时分秒`
- `publish_release`
- `push_docker`

## 预发布自动识别

当 tag 或手动输入的 `release_tag` 包含以下关键字时，workflow 会自动标记为 `prerelease`：

- `alpha`
- `beta`
- `rc`
- `pre`
- `preview`
- `dev`
- `nightly`
- `test`
- `manually`

也就是说，手动触发且不填写 `release_tag` 时，仍然会发布 GitHub Release，但会强制作为预发布版本。

## CLI 构建矩阵

CLI 发布对齐旧版 `release.py` 的平台覆盖范围，当前 workflow 会尝试构建以下目标：

- `darwin-amd64`
- `darwin-arm64`
- `lxc-alpine-musl-amd64`
- `linux-386`
- `linux-amd64`
- `linux-arm5`
- `linux-arm6`
- `linux-arm7`
- `linux-arm64`
- `linux-ppc64le`
- `linux-riscv64`
- `freebsd-386`
- `freebsd-amd64`
- `windows-386`
- `windows-amd64`

CLI 压缩包统一命名为：

```text
ikuai-bypass-<platform-arch>.zip
```

LXC / Alpine / musl 版本额外发布为：

```text
ikuai-bypass-lxc-alpine-musl-amd64.tar.gz
```

当前策略：

- 能使用 `musl` 的 Linux CLI 目标，统一优先使用 `musl`
- `lxc-alpine-musl-amd64` 复用 `linux-amd64` 的同一份 `musl` 二进制

压缩包内容：

- `ikuai-bypass` 或 `ikuai-bypass.exe`
- `README.md`
- `config.yml`

## GUI 构建矩阵

GUI 使用 Tauri v2 原生打包，当前 workflow 覆盖：

- Windows x86_64
- Linux x86_64 AppImage
- macOS x86_64
- macOS aarch64

GUI 产物会直接上传到 GitHub Release。

## Docker 发布

Docker 与 GitHub Release 走同一套 workflow。

当前 Docker 目标平台：

- `linux/amd64`
- `linux/386`
- `linux/arm64`
- `linux/arm/v7`
- `linux/arm/v6`
- `linux/ppc64le`
- `linux/riscv64`

Docker tag 策略：

- 每个平台都会推送 `:<version>-<platform>`
- 再聚合为多架构 `:<version>`
- 非预发布的正式语义化版本 tag，再额外推送多架构 `:latest`

## 缓存

workflow 已启用：

- `Swatinem/rust-cache`：Rust 依赖与 `target`
- `actions/cache`：Bun 包缓存与 `app/frontend/node_modules`
- Docker Buildx 的 `type=gha` 层缓存

## UPX

CLI 发布默认强制尝试使用 `UPX -9` 压缩：

- Linux runner：安装并执行 `upx-ucl`
- Windows runner：安装并执行 `upx`
- 如果目标格式或平台不被 UPX 支持，则保留原始二进制继续发布

## 当前稳定版限制

由于当前 Rust stable 渠道与 `cross` 对 MIPS 系列目标缺少可用标准库组件，以下旧 Go 版本曾支持的目标暂未纳入 Rust 主线自动发布：

- `linux-mips`
- `linux-mipsle`
- `linux-mips64`
- `linux-mips64le`

## Docker 凭据约束

如果 `push_docker=true`，但仓库没有配置 `DOCKERHUB_USERNAME` 和 `DOCKERHUB_TOKEN`，workflow 会在元信息阶段直接失败，不会继续执行后续发布步骤。

## 本地构建

构建前端：

```bash
cd app/frontend
bun install
bun run build
```

构建 CLI：

```bash
cargo build --release -p ikb-cli
```

构建 GUI：

```bash
cargo tauri build --manifest-path app/src-tauri/Cargo.toml
```
