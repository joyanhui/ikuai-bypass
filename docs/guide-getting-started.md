---
title: 🚀 快速上手与入门指南
nav_order: 2
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

## 快速上手

### 1. 下载

从 [Releases](https://github.com/joyanhui/ikuai-bypass/releases) 下载适合你系统的版本。

| 场景 | 推荐下载 |
| :--- | :--- |
| Windows 桌面 | `ikuai-bypass-gui-windows-x86_64.exe.zip` |
| macOS 桌面 | `ikuai-bypass-gui-macos-aarch64.dmg`（M芯片）或 `x86_64.dmg`（Intel） |
| Linux 桌面 | `ikuai-bypass-gui-linux-x86_64.zip`（需 WebKitGTK/GTK3） |
| Android | `.apk` |
| iOS | `.ipa`（仅支持自签名或越狱） |
| Linux/NAS/Docker | CLI `ikuai-bypass-cli-linux-xxx.zip` |
| LXC/PVE CT | `ikuai-bypass-lxc-alpine-musl-x86_64.tar.gz` |
| Docker | `joyanhui/ikuai-bypass` |
| iKuai v4 应用市场 | `.ipkg` 文件，在"高级应用→应用市场→本地安装"上传 |

> **新手建议**：电脑用户直接下载 GUI 版本，解压/安装即可，无需命令行。

### 2. 配置

编辑 `config.yml` 文件：

```yaml
ikuai-url: http://192.168.9.1
username: admin
password: your_password
cron: "0 7 * * *"
custom-isp:
  - tag: "国内IP"
    url: "https://example.com/cn-ip.txt"
```

> **提示**：完整配置示例参考 [config.yml](../config.yml)。GUI 版本可在界面直接配置。

### 3. 运行

**GUI 用户**：双击打开应用，界面配置即可。

**CLI 用户**：
```bash
./ikuai-bypass -r cron -c ./config.yml                  # 定时自动更新（推荐）
./ikuai-bypass -r once -c ./config.yml                   # 单次运行
./ikuai-bypass -r exportDomainSteamToTxt -c ./config.yml -exportPath /tmp  # 导出域名分流列表
./ikuai-bypass -r clean -tag cleanAll -c ./config.yml    # 清理所有规则
```

WebUI：cron 模式启动后访问 `http://你的IP:19001`

[查看快速上手完整文档](quickstart.md)

## WebUI 与 GUI

### WebUI（网页管理界面）

CLI 版本在计划任务模式启动后，访问 `http://你的IP:19001` 即可管理：修改配置、手动触发更新、查看日志、一键诊断。默认端口 `19001`，可在配置文件中修改。

### GUI（桌面/手机应用）

无需命令行，下载对应平台版本直接运行。桌面版支持 Windows / macOS / Linux；手机版支持 Android（APK）和 iOS（需自签名或越狱）。Linux 需已安装 `WebKitGTK` 和 `GTK3`。支持一键运行/停止和实时日志查看。

[查看 WebUI 与 GUI 完整文档](webui-gui.md)

## CLI 参数说明

### 常用参数

| 参数 | 说明 |
| :--- | :--- |
| `-c` | 配置文件路径 |
| `-r` | 运行模式（见下表） |
| `-m` | 分流模式（默认 `ispdomain`，一般不用改） |
| `-tag` | 清理模式必填，指定要清理的规则名 |
| `-login` | 覆盖配置文件登录信息（格式：`http://IP,username,password`） |
| `-exportPath` | 域名分流规则列表导出目录（用于调试/人工检查，默认 `/tmp`） |
| `-isIpGroupNameAddRandomSuff` | IP 分组名称是否增加随机后缀（`1` 开启 / `0` 关闭；默认开启） |

### 运行模式 (`-r`)

| 模式 | 说明 | 使用场景 |
| :--- | :--- | :--- |
| `cron` | 定时运行 | **最常用**，执行依次更新然后切换到任务计划模式等待定时再次触发 |
| `cronAft` | 定时运行 | 暂时不执行，直接进入计划任务模式 |
| `once` | 只运行一次 | 测试配置、手动更新 |
| `clean` | 清理规则 | 删掉所有规则和分组 |
| `exportDomainSteamToTxt` | 导出域名分流 TXT | 下载 `stream-domain` 的域名列表并导出到 `-exportPath` 目录 |

### 分流模式 (`-m`)

一般使用默认的 `ispdomain` 即可，特殊情况才需要改：

| 模式 | 说明 |
| :--- | :--- |
| `ispdomain` | 运营商+域名分流（默认，推荐） |
| `ipgroup` | IPv4分组模式 |
| `ipv6group` | IPv6分组模式 |
| `ii` | 运营商和域名分流+IPv4分组混合模式 |
| `ip` | IPv4 + IPv6 分组 |
| `iip` | 完整混合模式 ips+domain+ipv4+ipv6 |

## 部署方案

- **桌面用户/手机用户**：下载对应 GUI 版本直接运行即可
- **服务器 / CLI**：下载 CLI 版本，建议配置为系统服务。OpenWrt 用户可参考[服务脚本](openwrt-service-install.md)
- **Docker**：`docker run -itd --name ikuai-bypass --restart=always -e APP_RUN_MODE=ispdomain -p 19001:19001 -v ./data:/etc/ikuai-bypass joyanhui/ikuai-bypass:latest`，启动后在网页界面配置
- **iKuai v4 应用市场**：上传 `.ipkg` 包安装
- **Unraid / 群晖**：Docker 套件中搜索 `joyanhui/ikuai-bypass`，映射端口和配置目录

[查看部署方案完整文档](deployment.md)

## 注意事项

- 配置项中的规则名称/分组名不要太长（建议不超过 11 个汉字或字母），新版爱快不支持太长的名字，系统会自动加前缀。
- 与旧版ikuai-bypass不同，新版清理规则时必须指定 `-tag` 参数，避免误删
- 网页界面端口默认是 `19001`，可以在配置里改
