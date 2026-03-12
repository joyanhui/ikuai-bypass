# Release

当前仓库的发布主线为 Rust 版本，GitHub Actions 位于 [.github/workflows/release.yml](/home/y/myworkspace/ikuai-bypass/.github/workflows/release.yml)。

## CLI 构建矩阵

CLI 发布对齐旧版 `release.py` 的平台覆盖范围，当前 workflow 会尝试构建以下目标：

- `darwin-amd64`
- `darwin-arm64`
- `linux-386`
- `linux-amd64`
- `linux-arm5`
- `linux-arm6`
- `linux-arm7`
- `linux-arm64`
- `linux-mips`
- `linux-mipsle`
- `linux-mips64`
- `linux-mips64le`
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
