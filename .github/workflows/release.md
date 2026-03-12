# Release Workflow 规则说明

本文档描述当前 [release.yml](/home/y/myworkspace/ikuai-bypass/.github/workflows/release.yml) 的实际构建与发布规则，后续修改 workflow 时应同步更新。

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
- `enable_experimental_nightly`
- `publish_release`
- `push_docker`

手动触发时的规则：

- 如果 `publish_release=true` 或 `push_docker=true`，则必须填写 `release_tag`
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

仅 `full` 且 `enable_experimental_nightly=true` 时启用：

- `linux-mipsel` -> `mipsel-unknown-linux-musl`
- `linux-mips64` -> `mips64-unknown-linux-gnuabi64`
- `linux-mips64el` -> `mips64el-unknown-linux-gnuabi64`
- `linux-mips` -> `mips-unknown-linux-gnu`

说明：

- 这组目标属于实验性 nightly 架构
- 默认关闭，避免拖累正常发布链
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

### iOS GUI

仅 `full`：

- `ios-aarch64`

## 7. 附加产物规则

### LXC / Alpine

当 stable CLI 中包含 `linux-amd64` 时构建：

- `ikuai-bypass-lxc-alpine-musl-amd64.tar.gz`

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

## 8. 发布条件

`publish` job 仅在以下条件满足时执行：

- `publish_release=true`
- 依赖 job 中没有 `failure`
- 依赖 job 中没有 `cancelled`

发布内容：

- 收集所有构建产物
- 创建或更新 GitHub Release
- 自动生成 Release Notes
- 根据版本名决定是否标记为 `prerelease`

## 9. 关键实现文件

- [release.yml](/home/y/myworkspace/ikuai-bypass/.github/workflows/release.yml)
- [build_matrix.jsonc](/home/y/myworkspace/ikuai-bypass/.github/build_matrix.jsonc)
- [arch-helpers.sh](/home/y/myworkspace/ikuai-bypass/.github/scripts/arch-helpers.sh)
- [prepare-container-binaries.sh](/home/y/myworkspace/ikuai-bypass/.github/scripts/prepare-container-binaries.sh)

## 10. 维护约束

后续如果调整以下内容，必须同步更新本文档：

- 触发方式
- prerelease 判定关键字
- Docker `latest` 规则
- stable / nightly / BSD / GUI 构建矩阵
- experimental nightly 开关逻辑
- 发布门槛与产物收集逻辑
