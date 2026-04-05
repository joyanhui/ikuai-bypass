# Release Workflow 规则说明

本文档描述当前 `.github/workflows/release.yml` 的实际构建与发布规则，后续修改 workflow 时应同步更新。

## 1. 触发方式

- 仅支持 `tag push`
- 仅支持 `workflow_dispatch`
- 不包含 `schedule`，不会每日自动构建

### Tag 触发

- 任意 tag push 都会触发发布流程
- tag 名称直接作为版本号 `version`
- tag push 时默认：
  - `publish_release=true`
  - `push_docker=true`

### 手动触发

可选输入参数：

- `release_tag`
- `build_mode`
- `build_target`
- `publish_release`
- `push_docker`

手动触发时的规则：

- `publish_release` 默认是 `true`
- `push_docker` 默认是 `true`
- 如果未填写 `release_tag` 但手动勾选了发布或推送 Docker，workflow 会自动生成版本号 `manual-release-年月日时分秒` 并继续发布
- 如果只是手动测试构建，可以不填 `release_tag`
- 不填 `release_tag` 且不发布时，会生成内部版本号 `manual-build-${GITHUB_RUN_ID}`

## 2. 预发布与正式发布判定

版本名转小写后，只要包含以下任一关键字，就视为预发布：

- `alpha`
- `beta`
- `rc`
- `pre`
- `preview`
- `dev`
- `nightly`
- `test`

规则如下：

- `workflow_dispatch` 手动执行：GitHub Release 一律标记为 `prerelease`
- 命中上述关键字：GitHub Release 标记为 `prerelease`
- 未命中上述关键字：GitHub Release 视为正式版本

## 3. Docker 标记规则

Docker 镜像标签始终包含：

- `${version}`

如果当前版本不是预发布，则额外打：

- `latest`

这意味着：

- `v1.2.3` 会发布 `v1.2.3` 和 `latest`
- `v1.2.3-rc1` 只会发布 `v1.2.3-rc1`
- `v1.2.3-test` 只会发布 `v1.2.3-test`

## 4. 构建模式

### `build_mode`

- `minimal`
- `full`

### `build_target`

- `cli+gui`
- `cli`
- `gui`

矩阵裁剪规则：

- `cli` 只构建 CLI、BSD、LXC、Docker 相关目标
- `gui` 只构建桌面 GUI、Android GUI、iOS GUI
- `cli+gui` 构建全部启用目标

## 5. 当前 CLI 构建矩阵

### Stable CLI

`minimal`：

- `linux-amd64`
- `windows-amd64`

`full` 额外包含：

- `linux-386`
- `linux-arm5`
- `linux-arm6`
- `linux-arm7`
- `linux-arm64`
- `linux-ppc64le`
- `linux-riscv64`
- `macos-amd64`
- `macos-arm64`

### BSD CLI

仅 `full`：

- `freebsd-amd64`
- `freebsd-386`

### Experimental Nightly CLI

仅 `full` 时启用：

- `linux-mipsel` -> `mipsel-unknown-linux-musl`
- `linux-mips64` -> `mips64-unknown-linux-gnuabi64`
- `linux-mips64el` -> `mips64el-unknown-linux-gnuabi64`
- `linux-mips` -> `mips-unknown-linux-gnu`

说明：

- 这组目标属于实验性 nightly 架构
- 选择 `full` 时自动包含
- job 名称固定为 `CLI Nightly Experimental`
- 该 job 设置了 `continue-on-error: true`

## 6. 当前 GUI 构建矩阵

### Desktop GUI

`minimal`：

- `linux-x86_64-appimage`

`full` 额外包含：

- `windows-x86_64`
- `macos-aarch64`
- `macos-x86_64`

### Android GUI

仅 `full`：

- `android-aarch64`
- `android-armv7`
- `android-x86_64`

说明：

- CI 会优先收集 `universal release unsigned.apk`
- 上传前会执行 `zipalign` 和 `apksigner`
- 最终发布的是可直接安装的已签名 APK，而不是原始 unsigned APK

### iOS GUI

仅 `full`：

- `ios-aarch64`

## 7. 最终发布文件命名

GitHub Actions 内部 artifact 名仅用于 job 间传递：

- `cli-*`
- `gui-*`
- `gui-android-*`
- `gui-ios-*`

真正上传到 GitHub Release 的最终文件名统一使用 `ikuai-bypass-*` 前缀，并显式带上系统与架构，避免 `linux/freebsd/windows/macos` 的同架构产物互相覆盖。

### CLI / BSD / Nightly CLI

- `ikuai-bypass-cli-linux-x86_64.zip`
- `ikuai-bypass-cli-linux-x86_32.zip`
- `ikuai-bypass-cli-linux-aarch64.zip`
- `ikuai-bypass-cli-linux-riscv64gc.zip`
- `ikuai-bypass-cli-linux-mips.zip`
- `ikuai-bypass-cli-linux-mipsle.zip`
- `ikuai-bypass-cli-linux-mips64.zip`
- `ikuai-bypass-cli-linux-mips64le.zip`
- `ikuai-bypass-cli-freebsd-x86_64.zip`
- `ikuai-bypass-cli-freebsd-x86_32.zip`
- `ikuai-bypass-cli-windows-x86_64.zip`
- `ikuai-bypass-cli-macos-x86_64.zip`
- `ikuai-bypass-cli-macos-aarch64.zip`

### Desktop GUI

- Windows GUI：`ikuai-bypass-gui-windows-x86_64.zip`
- Linux AppImage：`ikuai-bypass-gui-linux-x86_64.AppImage`
- macOS DMG：`ikuai-bypass-gui-macos-x86_64.dmg`
- macOS DMG：`ikuai-bypass-gui-macos-aarch64.dmg`

### Mobile GUI

- Android APK：`ikuai-bypass-gui-android-aarch64.apk`
- Android APK：`ikuai-bypass-gui-android-armv7.apk`
- Android APK：`ikuai-bypass-gui-android-x86_64.apk`
- iOS IPA：`ikuai-bypass-gui-ios-aarch64.ipa`

规则说明：

- `apk`、`AppImage`、`dmg`、`ipa` 直接以原生格式发布，不再额外打包为 zip
- 文件名不包含 `.upx`

## 8. 附加产物规则

### LXC / Alpine

当 stable CLI 中包含 `linux-amd64` 时构建：

- `ikuai-bypass-lxc-alpine-musl-x86_64.tar.gz`

### iKuai v4 ipkg

当 stable CLI 中包含 `linux-amd64` 时构建：

- `ikuai-bypass-<version>.ipkg`

触发与依赖规则：

- `build-ipkg` 仅在 `has_linux_amd64_cli=true` 时执行
- `build-ipkg` 依赖 `build-cli` 的 `cli-linux-amd64` artifact
- `build-ipkg` 依赖 `build-frontend` 的 `frontend-dist` artifact，因为 ipkg 内置 WebUI 静态文件
- 为了满足上面的依赖，`build-frontend` 在 `has_gui=true`、`push_docker=true` 或 `has_linux_amd64_cli=true` 任一条件满足时都会执行

版本规则：

- ipkg 的 `manifest.json` 与最终文件名会对 semver 预发布后缀做归一化，例如 `4.4.100-alpha9.2` 会写成 `4.4.100`
- 如果 workflow 的发布版本号不是 semver（例如 `manual-build-*` / `manual-release-*`），ipkg 会回退读取 `apps/cli/Cargo.toml` 的版本并继续归一化，确保 `manifest.json` 始终是 `X.Y.Z`

实现位置：

- `packaging/ikuai-ipkg/`
- `packaging/ikuai-ipkg/build-ipkg.sh`

### OpenWrt LuCI

当 CLI 构建启用时构建：

- `ikuai-bypass-luci-openwrt-all.ipk`

触发与实现规则：

- `build-openwrt-luci` 仅在 `has_cli=true` 时执行
- LuCI 包固定为 `Architecture: all`
- LuCI 包本身不内置 `ikuai-bypass` CLI 二进制，因此是一个通用跨架构包
- LuCI 页面运行时通过 GitHub API 查询 `joyanhui/ikuai-bypass` releases，并按当前 OpenWrt 路由器架构匹配 `ikuai-bypass-cli-linux-*.zip`
- 页面语言按浏览器语言自动切换，`zh-*` 显示中文，其余显示英文，且不依赖 LuCI 语言包
- 页面支持在 stable（`prerelease=false`）与 prerelease（`prerelease=true`）之间切换
- 点击安装后，由路由器本机下载对应 release asset，解压并安装到 `/usr/bin/ikuai-bypass`
- 若 release archive 内包含 `config.yml`，会额外落地到 `/etc/ikuai-bypass/config.yml.example`

实现位置：

- `packaging/openwrt-luci/luci-app-ikuai-bypass/`
- `packaging/openwrt-luci/build-openwrt-luci-package.sh`

### Docker Multi-Arch

仅在以下条件同时满足时执行：

- `push_docker=true`
- Docker 凭据存在
- CLI 矩阵中存在可用于 Docker 的 Linux 目标

当前 Docker 平台来自 stable CLI 矩阵中的：

- `linux/amd64`
- `linux/386`
- `linux/arm/v7`
- `linux/arm64`
- `linux/ppc64le`
- `linux/riscv64`

说明：

- FreeBSD、macOS、Windows、nightly MIPS 不参与 Docker multi-arch

## 9. 发布条件

`publish` job 仅在以下条件满足时执行：

- `publish_release=true`
- 依赖 job 中没有 `failure`
- 依赖 job 中没有 `cancelled`

发布内容：

- 收集所有构建产物
- 创建或更新 GitHub Release
- 自动生成 Release Notes
- 根据版本名决定是否标记为 `prerelease`

## 10. 关键实现文件

- `.github/workflows/release.yml`
- `.github/build_matrix.jsonc`
- `.github/scripts/arch-helpers.sh`
- `packaging/docker/prepare-container-binaries.sh`
- `packaging/ikuai-ipkg/build-ipkg.sh`
- `packaging/openwrt-luci/build-openwrt-luci-package.sh`
- `packaging/openwrt-luci/luci-app-ikuai-bypass/`

## 11. 维护约束

后续如果调整以下内容，必须同步更新本文档：

- 触发方式
- prerelease 判定关键字
- Docker `latest` 规则
- stable / nightly / BSD / GUI 构建矩阵
- `full` 模式下的 nightly 构建逻辑
- 最终发布文件命名规则
- ipkg / OpenWrt LuCI 构建条件、版本处理与目录位置
- 发布门槛与产物收集逻辑
