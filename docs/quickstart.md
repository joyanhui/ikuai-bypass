---
title: 🚀 快速上手与入门指南
type: docs
weight: 2
---

# 🚀 快速上手与入门指南

## 主要功能特性

- **多种分流模式**：支持自定义运营商、IPv4/v6分组、域名分流、端口分流等，满足不同网络需求。
- **自动定时运行和手动规则结合**：设置好后定时自动更新规则，ikuai-bypass管理的规则会使用特殊的名字和备注（一般以`IKB` `bypss`等字符标示）不会影响你手动管理的分流规则。
- **一键清理和干净卸载**：可以一键删除所有规则，本程序只有一个可执行二进制文件和配置文件，随时可以删除卸载。
- **全平台支持**：Windows、macOS、Linux、FreeBSD、安卓APP、苹果App 全平台覆盖，还有 Docker  PVE LXC/CT版本 以及 爱快应用市场版。支持mips、arm5等上古硬件。
- **尽可能的用户友好**：提供WebUI界面和桌面/手机 App协助编辑配置文件（亦可手动编辑配置文件）。

## 爱快两种分流模式解析

本项目支持两种主流的分流实现方案，您可以根据自己的网络拓扑选择最合适的模式。

- **自定义运营商分流模式和域名（推荐）**：旁路由作为 iKuai 的"虚拟运营商"，来实现IP分流。通过订阅域名分流规则实现域名分流。配合爱快的多wan功能可以实现宕机自愈。
- **IP 分组与端口分流模式**：通过ip v4/v6分组和端口分流的下一跳网关将流量指向旁路由。配置简单，无需多网卡。

> 两个模式实际上是可以按需组合使用的，一般来说 ip分组配合端口分流 这个模式 更加简单直观 也灵活,旁路由即便是单臂路由也可以。

[查看分流模式详细说明](router-mode.md)

## 一键安装

可以通过 [github release](https://github.com/joyanhui/ikuai-bypass/releases) 下载对应系统和架构的压缩包解压后 运行即可。同时提供以下安装方式
### 一键安装脚本
```bash
curl -fsSL https://joyanhui.github.io/ikuai-bypass/install.sh | sh
```
> 支持较新的ubuntu/debian/openwrt 等linux发行版，兼容systemd和init。支持中英文双语言提示。
### OpenWrt LuCI 可视化面板
```bash
opkg install https://github.com/joyanhui/ikuai-bypass/releases/latest/download/ikuai-bypass-luci-openwrt-all.ipk
```
> 在OpenWrt内增加一个可视化的服务管理面板，可以完成安装 卸载 启动停止  开机启动等逻辑。把`install.sh`的步骤改成了可视化交互界面。

视化的面板 你可以在这个可视化面板完成 更新和安装 iKuaiBypass的cli版本为系统服务 并可以检查和配置某些常用配置项

## 配置IKB（ikuai-bypass）
实际上默认配置文件 已经覆盖了99%用户的需求，你只需要修改爱快的ip和登录用户名密码，以及规则部分的ip地址段即可。

### 手动编辑配置方法
这是最简单直观的方法，编辑 `config.yml` 文件（你可以在启动ikuai-bypass的时候用`-c`参数指定配置文件的路径，默认是当前目录的config.yml文件），修改以下基本信息,以及后面的ip地址段适合你的情况就可以了：
```yaml
# 爱快路由器地址和登录信息
ikuai-url: http://192.168.9.1 # 改成你的爱快地址
username: admin # 登录用户名
password: your_password # 爱快路由器的登录密码
```

### 可视化编辑
执行 `./ikuai-bypass -r cron -c ./config.yml` 可以访问 `http://你的ip:19001` 然后使用默认的webui管理用户名密码 `admin` `admin888` 进入一个可视化表单来修改编辑配置。

### GUI（桌面/手机应用）

无需命令行，下载对应平台版本直接运行。桌面版支持 Windows / macOS / Linux；手机版支持 Android（APK）和 iOS（需自签名或越狱）。Linux 需已安装 `WebKitGTK` 和 `GTK3`。支持一键运行/停止和实时日志查看。

## 运行IKB

对 桌面版和APP版本，你打开即可配置和使用。

对于手动安装的 CLI/命令行版本（windows（cmd/ps）或者  openwrt或者其他linux以及macos下 ）命令行模式下下 你常用的启动方式

```bash
./ikuai-bypass -r cron -c ./config.yml  # 最常用：定时自动更新（推荐）
./ikuai-bypass -r once -c ./config.yml # 只运行一次就退出
./ikuai-bypass -r clean -tag cleanAll -c ./config.yml # 清理所有规则（慎用）
```
完整的 CLI 参数（运行模式、分流模式等）请查看 [CLI 参数说明](cli-params.md)。

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
