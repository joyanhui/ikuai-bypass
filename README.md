# iKuai Bypass

![iKuai](https://img.shields.io/badge/Router-iKuai-brightgreen) ![License](https://img.shields.io/badge/License-AGPL%203.0-blue.svg) ![Rust](https://img.shields.io/badge/Language-Rust-orange)

**iKuai Bypass** 是一款爱快路由器专用的分流规则自动同步工具。它可以自动从网上下载 IP/域名列表并同步到你的路由器，让你的流量自动走正确的线路。比如：国内流量直连、国外流量走代理，无需手动配置每一条规则。

提供两种使用方式：
- **CLI**：适合服务器、路由器、OpenWrt、Docker、LXC、计划任务环境
- **GUI**：图形化桌面和手机 App，支持 Windows / macOS / Linux 桌面端和 Android / iOS 移动端，用户友好，几乎开箱即用

> **提示**：旧版本代码已归档到分支[v4.4.13](https://github.com/joyanhui/ikuai-bypass/releases/tag/untagged-08d99dfe501855617744),新版本使用性能更好，内存安全更好的rust语言构建。

> **如果这个项目对你有帮助，请点个 ⭐️ Star！** star数是作者唯一的维护动力。


> 关于dns分流解析，建议用 ADGuard home自建，这里有一个本人利用github action自动维护相关规则文件的adguardhome规则.[[joyanhui/adguardhome-rules]](https://github.com/joyanhui/adguardhome-rules)（规则文件在release_file分支48小时更新一次）。并提供的教程，可以简单自动更新dns分流解析规则，广告屏蔽，以及ipv4优先等功能

---

## 可视化界面展示

本项目提供 WebUI 和 Tauri GUI 两种可视化界面，支持在线配置和运行状态监控。

- **WebUI**：供CLI 模式启动后通过浏览器访问，进提供可视化编辑配置文件功能。
- **桌面或移动设备 可视化**：Windows / macOS / Linux 桌面应用 / 安卓apk / ios自签ipa 

---

## 爱快喜闻乐见分流模式解析

本项目支持两种主流的分流实现方案，您可以根据自己的网络拓扑选择最合适的模式。

### 1. 自定义运营商分流模式 (推荐)

**适用场景：** 追求极致稳定性、网络自愈、终端无感分流。

**实现逻辑：**
这种模式下，iKuai 将 OpenWrt（或其他网关）视为一个"虚拟的上级运营商"。
1. **链路设计**：OpenWrt 作为 iKuai 的下级设备接收流量，处理后再将出口流量"绕回"给 iKuai 的物理 WAN 口。
2. **规则同步**：本工具将目标 IP 列表导入 iKuai 的"自定义运营商"。iKuai 会认为这些 IP 属于该"虚拟运营商"，从而将流量转发给 OpenWrt。

**核心优势：**
- **极高可靠性**：OpenWrt 宕机只会导致被分流的流量中断，普通流量依然通过主线直连，不会全家断网。
- **配置无感**：终端设备无需更改网关配置，完全由 iKuai 在内核层级完成调度。
- **性能优异**：直连速度最快，旁路仅处理特定流量。

**参考文档**：[查看具体实现方式](https://dev.leiyanhui.com/route/ikuai-bypass-joyanhui/) 或 [恩山eezz的教程](https://www.right.com.cn/forum/thread-8252571-1-1.html)。

<details>
<summary>点击这里展开查看详细图文说明</summary>
<img src="golang_archive/assets/img.png"  alt="自定义运营商分流模式拓扑图">
</details>

### 2. IP 分组与端口分流模式 (传统模式)

**适用场景：** 简单的旁路由方案，逻辑直接。

**实现逻辑：**
1. **IP 分组**：本工具将订阅的 IP 列表同步到 iKuai 的"IP 分组"中。
2. **策略路由**：利用 iKuai 的"端口分流"功能，匹配目标地址为该分组的流量，将其"下一跳网关"指向 OpenWrt 的 IP。

**特点**：配置简单直接，但在 OpenWrt 宕机时，匹配到该分组的规则将无法上网（当然也无伤大雅）。

**参考文档**：[实现方式参考](https://github.com/joyanhui/ikuai-bypass/issues/7) 或 [恩山y2kji的教程](https://www.right.com.cn/forum/thread-8288009-1-1.html)。

---

## 主要功能特性

- 🛡️ **安全可靠**：更新规则时不会中断现有分流功能。如果下载失败，会保留旧规则不清理，确保你的网络始终正常。
- 🌐 **多种分流模式**：支持自定义运营商、IP分组、域名分流、端口分流等，满足不同网络需求。
- 📅 **全自动运行**：设置好后可以定时自动更新规则，完全不用管。
- 🛠️ **一键清理**：不需要了可以一键删除所有规则，干净利落。
- 💻 **全平台支持**：Windows、macOS、Linux、Android、iOS 全覆盖，还有 Docker 版本。
- 🖥️ **界面友好**：提供网页管理界面和桌面/手机 App，点点鼠标就能配置，新手也能轻松上手。

---

## 快速上手

### 1. 下载

从 [Releases](https://github.com/joyanhui/ikuai-bypass/releases) 下载适合你系统的版本。

**选哪个版本？**

| 你的系统 | 推荐下载 |
| :--- | :--- |
| Windows 桌面 | `ikuai-bypass_x64-setup.msi` 或 `.exe` |
| macOS 桌面 | `ikuai-bypass_aarch64.dmg`（M芯片）或 `x64.dmg`（Intel） |
| Linux 桌面 | `.AppImage` 或 `.deb` |
| Android 手机 | `.apk` |
| iOS 手机 | `.ipa`（仅支持自签名或越狱设备） |
| 服务器/路由器/容器 | CLI 版本 `ikuai-bypass-linux-xxx.zip` |
| LXC/Alpine 容器 | `ikuai-bypass-lxc-alpine-musl-amd64.tar.gz` |

> **新手建议**：如果你在电脑上使用，直接下载 GUI 版本（安装包），双击安装即可，无需命令行。

### 2. 配置

编辑 `config.yml` 文件，填写以下基本信息：

```yaml
# 爱快路由器地址和登录信息
ikuai-url: http://192.168.9.1   # 改成你的爱快地址
username: admin                   # 登录用户名
password: your_password           # 登录密码

# 定时更新（每天早上7点）
cron: "0 7 * * *"

# 要同步的规则列表
custom-isp:
  - tag: "国内IP"
    url: "https://example.com/cn-ip.txt"
```

> **提示**：完整配置示例请参考 [config.yml](./config.yml)，里面有详细注释。GUI 版本可以在界面里直接配置。

### 3. 运行

**GUI 用户**：双击打开应用，在界面里配置即可，无需命令行。

**CLI 用户**：

```bash
# 最常用：定时自动更新（推荐）
./ikuai-bypass -r cron -c ./config.yml

# 只运行一次就退出
./ikuai-bypass -r once -c ./config.yml

# 打开网页管理界面
./ikuai-bypass -r web -c ./config.yml

# 清理所有规则（慎用）
./ikuai-bypass -r clean -tag cleanAll -c ./config.yml
```

---

## WebUI 与 GUI

### WebUI（网页管理界面）

启动后用浏览器访问 `http://你的IP:19001` 就能看到管理界面，可以在网页上：
- 修改配置
- 手动触发更新
- 查看运行日志

启动方式：
```bash
./ikuai-bypass -r web -c ./config.yml
```

默认端口 `19001`，可以在配置文件里修改。

### GUI（桌面/手机应用）

如果你不想折腾命令行，直接下载 GUI 版本：
- **桌面版**：Windows / macOS / Linux，双击安装即可
- **手机版**：Android 直接安装 APK；iOS 仅支持自签名或越狱用户

GUI 功能：
- 可视化配置，填表单就行
- 一键运行/停止
- 实时查看日志

> **新手首选 GUI**，简单易懂，不用记命令。

---

## CLI 参数说明

> **GUI 用户可以跳过这部分**，在界面里操作即可。

### 常用参数

| 参数 | 说明 |
| :--- | :--- |
| `-c` | 配置文件路径 |
| `-r` | 运行模式（见下表） |
| `-m` | 分流模式（默认 `ispdomain`，一般不用改） |
| `-tag` | 清理模式必填，指定要清理的规则名 |

### 运行模式 (`-r`)

| 模式 | 说明 | 使用场景 |
| :--- | :--- | :--- |
| `cron` | 定时运行 | **最常用**，设置好后自动更新 |
| `once` | 只运行一次 | 测试配置、手动更新 |
| `web` | 只启动网页界面 | 想用网页管理时 |
| `clean` | 清理规则 | 不需要了，删掉所有规则 |

### 分流模式 (`-m`)

一般使用默认的 `ispdomain` 即可，特殊情况才需要改：

| 模式 | 说明 |
| :--- | :--- |
| `ispdomain` | 运营商+域名分流（默认，推荐） |
| `ipgroup` | IP分组模式 |
| `ipv6group` | IPv6分组模式 |
| `ii` | 混合模式 |
| `ip` | IPv4 + IPv6 分组 |
| `iip` | 完整混合模式 |

---

## 部署方案

### 电脑桌面用户（推荐新手）

1. 下载对应系统的安装包（Windows 用 `.msi` 或 `.exe`，Mac 用 `.dmg`）
2. 双击安装
3. 打开软件，在界面里配置爱快地址和登录信息
4. 点击"运行一次"测试，没问题后开启定时任务

### 手机用户

1. Android：直接下载 `.apk` 安装
2. iOS：下载 `.ipa` 后需要自签名（AltStore 等）或越狱设备安装，普通用户不太推荐

### 服务器 / 路由器用户

下载 CLI 版本，用命令行运行。建议配置成系统服务，开机自启动。

**OpenWrt 用户**：可以参考这个[服务脚本](https://github.com/joyanhui/ikuai-bypass/blob/main/golang_archive/example/script/AddOpenwrtService.sh)

### Docker 用户

适合 NAS、群晖、或者喜欢用容器的用户，详见下方 Docker 章节。

---

## Docker 部署

适合 NAS、群晖、或者习惯用 Docker 的用户。

### 快速开始

```bash
# 拉取镜像
docker pull joyanhui/ikuai-bypass:latest

# 运行（会自动创建配置文件）
docker run -d \
  --name ikuai-bypass \
  --restart=always \
  -p 19001:19001 \
  -v ./data:/etc/ikuai-bypass \
  joyanhui/ikuai-bypass:latest
```

启动后：
1. 用浏览器打开 `http://你的IP:19001`
2. 在网页界面里配置爱快地址和登录信息
3. 点击"运行一次"测试，成功后开启定时任务

### 常用命令

```bash
# 查看日志
docker logs -f ikuai-bypass

# 手动触发更新
docker exec ikuai-bypass ikuai-bypass -r once

# 重启容器
docker restart ikuai-bypass

# 升级版本
docker pull joyanhui/ikuai-bypass:latest
docker stop ikuai-bypass && docker rm ikuai-bypass
# 然后重新运行 docker run 命令（配置文件会保留）
```

### 群晖 / NAS 部署

在群晖的 Docker 套件里：
1. 搜索 `joyanhui/ikuai-bypass` 并下载
2. 创建容器，映射端口 `19001`
3. 映射一个文件夹到 `/etc/ikuai-bypass` 存放配置
4. 启动后访问网页界面配置

或者用 docker-compose：

```yaml
version: "3.8"
services:
  ikuai-bypass:
    image: joyanhui/ikuai-bypass:latest
    container_name: ikuai-bypass
    restart: always
    ports:
      - "19001:19001"
    volumes:
      - ./data:/etc/ikuai-bypass
```

---

## 注意事项

- 规则名称不要太长（建议不超过 11 个汉字或字母），系统会自动加前缀
- 清理规则时必须指定 `-tag` 参数，避免误删
- 网页界面端口默认是 `19001`，可以在配置里改
- 如果规则列表下载失败，程序会保留旧规则不更新，不影响正常使用

---

## 更新与下载

所有版本都在 [Releases](https://github.com/joyanhui/ikuai-bypass/releases) 页面下载，包括：
- 桌面版安装包
- 手机版安装包
- CLI 命令行版本
- Docker 镜像

---

## 赞助支持

虽世道艰难也一直在坚持维护这个项目，您的一份心意能让我更有动力继续完善这个工具，非常感谢您的支持！

- **TRX (Tron TRC20) 钱包地址**：`TLiv9F6i38uZEGdp8VoB5qLxJx43aV9XSZ`

当然，您也可以通过在 GitHub 上给项目点一个 ⭐️ Star 来支持我，这对我也是莫大的支持！

---

## 交流与反馈

- **交流讨论**：[GitHub Discussions](https://github.com/joyanhui/ikuai-bypass/discussions) 或 恩山无线论坛
- **Bug 反馈**：[GitHub Issues](https://github.com/joyanhui/ikuai-bypass/issues)
- **致谢**：感谢 [ztc1997](https://github.com/ztc1997/ikuai-bypass/) 的初始版本思路，以及所有提供 PR 的开发者。
