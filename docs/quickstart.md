---
title: 🚀 快速上手与入门指南
nav_order: 2
has_children: true
---

# 🚀 快速上手与入门指南

## 主要功能特性

- **安全可靠**：更新规则时不会中断现有分流功能。如果下载失败，会保留旧规则不清理，确保你的网络始终正常。
- **多种分流模式**：支持自定义运营商、IP分组、域名分流、端口分流等，满足不同网络需求。
- **全自动运行**：设置好后定时自动更新规则。
- **一键清理**：不需要了可以一键删除所有规则，干净利落。
- **全平台支持**：Windows、macOS、Linux、FreeBSD、Android、iOS 全平台覆盖，还有 Docker 和 PVE LXC/CT版本。甚至支持mips、arm5等上古硬件。
- **界面友好**：提供网页管理界面和桌面/手机 App，多端界面统一。

## 爱快两种分流模式解析

本项目支持两种主流的分流实现方案，您可以根据自己的网络拓扑选择最合适的模式。

- **自定义运营商分流模式（推荐）**：旁路由作为 iKuai 的"虚拟运营商"，由 iKuai 内核级调度。旁路由宕机不影响普通流量，终端无需更改网关配置，稳定性极高。
- **IP 分组与端口分流模式**：将 IP 列表同步到 iKuai 的 IP 分组，通过端口分流将流量指向旁路由。配置简单，旁路由宕机时被分流设备无法上网。

[查看分流模式完整文档](router-mode.md)

## 一键安装

### Linux 安装 CLI 服务

如果你在使用 Linux 服务器/LXC/OpenWRT，并希望直接安装 CLI 二进制和系统服务：

```bash
curl -fsSL https://joyanhui.github.io/ikuai-bypass/install.sh | sh
```

安装脚本会自动检测你的系统架构，下载对应 CLI 二进制文件，注册为系统服务（systemd 或 init. y）并启动 WebUI。这个脚本支持较新的ubuntu/debian/openwrt 等linux发行版。

### OpenWrt LuCI 面板

如果你只想在 OpenWrt 管理界面里安装 可视化管理面板，直接安装最新正式版 IPK：

```bash
opkg install https://github.com/joyanhui/ikuai-bypass/releases/latest/download/ikuai-bypass-luci-openwrt-all.ipk
```

安装后会有一个可视化的面板 你可以在这个可视化面板完成 更新和安装 iKuaiBypass的cli版本为系统服务 并可以检查和配置某些常用配置项

## 手动下载

从 [Releases](https://github.com/joyanhui/ikuai-bypass/releases) 下载适合你系统的版本。

| 场景               | 推荐下载                                                                      |
| :----------------- | :---------------------------------------------------------------------------- |
| Windows 桌面       | `ikuai-bypass-gui-windows-x86_64.exe.zip` 解压                                |
| macOS 桌面         | `ikuai-bypass-gui-macos-aarch64.dmg`（M芯片）或 `x86_64.dmg`（Intel）         |
| Linux 桌面         | `ikuai-bypass-gui-linux-x86_64.zip` 解压后运行（需系统已安装 WebKitGTK/GTK3） |
| Android            | `.apk` 侧载安装                                                               |
| iOS                | `.ipa`（仅支持自签名或越狱设备）                                              |
| 服务器/路由器/容器 | CLI 版本 `ikuai-bypass-cli-linux-xxx.zip`                                     |
| LXC/PVE CT 容器    | `ikuai-bypass-lxc-alpine-musl-x86_64.tar.gz`                                  |
| Docker             | [joyanhui/ikuai-bypass](https://hub.docker.com/r/joyanhui/ikuai-bypass/tags)  |
| iKuai v4 应用市场  | `ikuai-bypass-x86_64.ipkg`，在爱快"高级应用 -> 应用市场 -> 本地安装"上传      |

> **新手建议**：如果你在电脑上使用，直接下载 GUI 版本即可；Windows / Linux 解压后运行，macOS 打开 DMG 安装，无需命令行，或者安卓手机版也不错。

## 配置

编辑 `config.yml` 文件，填写以下基本信息：

```yaml
# 爱快路由器地址和登录信息
ikuai-url: http://192.168.9.1 # 改成你的爱快地址
username: admin # 登录用户名
password: your_password # 登录密码

# 定时更新（每天早上7点）
cron: "0 7 * * *"

# 要同步的规则列表
custom-isp:
  - tag: "国内IP"
    url: "https://example.com/cn-ip.txt"
```

> **提示**：完整配置示例请参考 [config.yml](../config.yml)，里面有详细注释。GUI 版本可以在界面里直接配置。关于 proxy 与 github-proxy 的区别 [查看文档](proxy-vs-github-proxy-guide.md)。

## 运行

**GUI 用户**：双击打开应用，在界面里配置即可，无需命令行。

**CLI 用户**：

```bash
# 最常用：定时自动更新（推荐）
./ikuai-bypass -r cron -c ./config.yml
# 只运行一次就退出
./ikuai-bypass -r once -c ./config.yml
# 导出域名分流列表到 TXT（不连接 iKuai，仅用于调试/人工导入）
./ikuai-bypass -r exportDomainSteamToTxt -c ./config.yml -exportPath /tmp
# 清理所有规则（慎用）
./ikuai-bypass -r clean -tag cleanAll -c ./config.yml

# WebUI：cron / cronAft 启动后，若配置中启用 WebUI（webui.enable=true），
# 可直接访问 http://你的IP:19001 查看状态、修改配置、停止定时任务
```

完整的 CLI 参数（运行模式、分流模式等）请查看 [CLI 参数说明](cli-params.md)。

## WebUI 与 GUI

### WebUI（网页管理界面）

CLI 版本在计划任务模式启动后，访问 `http://你的IP:19001` 即可管理：修改配置、手动触发更新、查看日志、一键诊断。默认端口 `19001`，可在配置文件中修改。

### GUI（桌面/手机应用）

无需命令行，下载对应平台版本直接运行。桌面版支持 Windows / macOS / Linux；手机版支持 Android（APK）和 iOS（需自签名或越狱）。Linux 需已安装 `WebKitGTK` 和 `GTK3`。支持一键运行/停止和实时日志查看。

## 部署方案

- **桌面用户/手机用户**：下载对应 GUI 版本直接运行即可
- **Linux / OpenWrt CLI 服务**：执行 `curl -fsSL https://joyanhui.github.io/ikuai-bypass/install.sh | sh`
- **OpenWrt LuCI 面板**：执行 `opkg install https://github.com/joyanhui/ikuai-bypass/releases/latest/download/ikuai-bypass-luci-openwrt-all.ipk`
- **服务器 / CLI**：下载 CLI 版本，建议配置为系统服务。OpenWrt 用户可参考[服务脚本](openwrt-service-install.md)
- **Docker / LXC / 群晖 / Unraid**：详见[部署方案完整文档](deployment.md)

[查看部署方案完整文档](deployment.md)

## 注意事项

- 配置项中的规则名称/分组名不要太长（建议不超过 11 个汉字或字母），新版爱快不支持太长的名字，系统会自动加前缀。
- 与旧版ikuai-bypass不同，新版清理规则时必须指定 `-tag` 参数，避免误删
- 网页界面端口默认是 `19001`，可以在配置里改
