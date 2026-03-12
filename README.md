# iKuai Bypass

![iKuai](https://img.shields.io/badge/Router-iKuai-brightgreen) ![License](https://img.shields.io/badge/License-AGPL%203.0-blue.svg) ![Rust](https://img.shields.io/badge/Language-Rust-orange)

**iKuai Bypass** 是专为爱快路由器打造的自动化分流规则同步工具。通过模拟 Web 管理界面，自动将远程订阅的 IP/域名列表同步到路由器，实现精准流量调度。支持自定义运营商、IP/IPv6分组、域名分流、端口分流等多种模式，具备安全就地更新、定时任务、可视化配置界面等特性，兼容全平台多架构。

当前仓库主线已经切换为 **Rust 版本**，提供两种使用方式：
- **CLI**：适合服务器、OpenWrt、Docker、LXC、计划任务环境
- **GUI**：基于 Tauri v2 的跨平台应用，支持 Windows / macOS / Linux 桌面端和 Android / iOS 移动端

旧的 Go/Fyne 版本代码、历史文档和旧 CI 已归档到 [golang_archive](./golang_archive)。

> **如果这个项目对你有帮助，请点个 ⭐️ Star！** star数是作者唯一的维护动力。


> 关于dns分流解析，建议用 ADGuard home自建，这里有一个本人利用github action自动维护相关规则文件的adguardhome规则.[[joyanhui/adguardhome-rules]](https://github.com/joyanhui/adguardhome-rules)（规则文件在release_file分支48小时更新一次）。并提供的教程，可以简单自动更新dns分流解析规则，广告屏蔽，以及ipv4优先等功能

---

## 可视化界面展示

本项目提供 WebUI 和 Tauri GUI 两种可视化界面，支持在线配置和运行状态监控。

- **WebUI**：CLI 启动后通过浏览器访问，支持所有平台
- **桌面 GUI**：Windows / macOS / Linux 桌面应用
- **移动 GUI**：Android / iOS 移动应用

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

- 🛡️ **安全就地更新**：采用"就地更新"策略，匹配则 Edit，不匹配则 Add，保持爱快内部 ID 稳定。远程资源下载失败时立即终止当前项更新，严禁清理旧规则，确保分流功能不中断。
- 🌐 **多维度支持**：涵盖运营商分流 (IPv4/IPv6分流)、IP分组、IPv6分组、域名分流、端口分流 (Next-hop)。
- 📅 **全自动运营**：内置 Cron 计划任务，兼容系统计划任务，支持多配置文件并发运行。
- 🛠️ **工具链完备**：支持一键清理、单次运行、规则导出。
- 💻 **广泛兼容**：提供全平台多架构支持，Linux 默认使用 musl 静态构建，便于在更多轻量环境中直接运行。
- 🖥️ **多端支持**：CLI + WebUI 模式，以及基于 Tauri v2 的跨平台 GUI 应用，支持 Windows / macOS / Linux 桌面端和 Android / iOS 移动端。

---

## 快速上手

### 1. 下载

从 [Releases](https://github.com/joyanhui/ikuai-bypass/releases) 下载适合你系统的版本。

常见文件类型：
- **CLI**：`ikuai-bypass-<platform-arch>.zip`
- **LXC / Alpine / musl**：`ikuai-bypass-lxc-alpine-musl-amd64.tar.gz`
- **桌面 GUI**：
  - Windows：`.msi` 或 `.exe` 安装包
  - macOS：`.dmg` 安装包
  - Linux：`.AppImage` 或 `.deb`
- **移动端 GUI**：
  - Android：`.apk` 安装包
  - iOS：`.ipa`（需要自行签名）

说明：
- Linux `amd64 / 386 / arm / arm64 / riscv64` CLI 现统一优先使用 `musl` 构建
- `linux-ppc64le` 当前使用 `gnu` 目标构建（`powerpc64le-unknown-linux-gnu`）
- `lxc-alpine-musl-amd64` 与 `linux-amd64` 复用同一份二进制，只是额外提供了更适合 LXC / Alpine 场景的打包格式

### 2. 配置

编辑 `config.yml`，填写爱快登录信息及订阅 URL：

```yaml
ikuai-url: http://192.168.9.1
username: admin
password: your_password
cron: "0 7 * * *" # 每天早上 7 点更新
custom-isp:
  - tag: "演示ip分组"
    url: "https://example.com/same.txt"
```

完整示例请直接参考 [config.yml](./config.yml)。

### 3. 运行

```bash
# 标准模式（先执行一次，后进入定时任务）
./ikuai-bypass -r cron -c ./config.yml

# WebUI 模式（启动可视化配置界面）
./ikuai-bypass -r web -c ./config.yml

# 单次模式（立即执行并退出）
./ikuai-bypass -r once -c ./config.yml

# 清理模式（删除所有名字包含 IKB 前缀的规则）
./ikuai-bypass -r clean -tag cleanAll -c ./config.yml
```

---

## WebUI 与 GUI

### WebUI

在配置文件里启用 `webui.enable: true` 后，可以启动 WebUI：

```bash
./ikuai-bypass -r web -c ./config.yml
```

然后在浏览器访问：`http://你的IP:19001`

默认端口是 `19001`，用户名和密码由 `config.yml` 的 `webui.user` 与 `webui.pass` 控制。

> **注意**：新版本已移除 `enable_updateCAN` 参数，WebUI 现在直接支持所有配置项的编辑和保存。

### GUI

GUI 是基于 Tauri v2 的跨平台桌面应用和移动应用，适合不想手动敲命令的用户。

GUI 里可以完成这些事：
- 运行一次
- 启动 / 停止定时任务
- 修改配置
- 文本编辑 YAML
- 查看实时日志

如果你使用 GUI，一般不需要自己手动拼 CLI 参数。

**支持平台**：
- 桌面端：Windows、macOS、Linux
- 移动端：Android、iOS

---

## 参数说明 (CLI Flags)

| 参数 | 说明 | 示例/取值 |
| :--- | :--- | :--- |
| `-c` | 配置文件路径 | `-c ./config.yml` |
| `-m` | **分流模块选择** | `ispdomain` (默认), `ipgroup`, `ipv6group`, `ii` (混合), `ip` (ipv4和ipv6分组), `iip` (ii+ip混合) |
| `-r` | 运行模式 | 见下表 |
| `-tag` | 清理模式下的标签关键词 | **必填项**。用于匹配 TagName 或名字，使用 `cleanAll` 清理全部 |
| `-login` | 覆盖配置文件登录信息 | `http://IP,username,password` |
| `-exportPath` | 导出路径 | 默认为 `/tmp` |
| `-isIpGroupNameAddRandomSuff` | IP分组名称是否增加随机数后缀 | `1` (添加), `0` (不添加)。仅 ipgroup 模式有效 |

### 运行模式 (`-r`) 详细说明

| 模式 | 名称 | 说明 |
| :--- | :--- | :--- |
| `cron` | 计划任务模式 | **默认模式**。立即执行一次更新，随后进入定时任务等待模式。若启用了 WebUI 则同步启动。 |
| `cronAft` | 延迟计划任务 | 不立即执行更新，直接进入定时任务等待模式。若启用了 WebUI 则同步启动。 |
| `once` / `nocron` / `1` | 单次模式 | 立即执行一次规则更新，完成后立即退出程序。 |
| `clean` | 清理模式 | 删除所有带有 `IKB` 前缀（或 `-tag` 指定）的规则和分组。 |
| `web` | WebUI 模式 | 启动可视化 Web 管理界面，用于在线修改配置。不做其他操作 |

---

## 部署方案

### Linux / OpenWrt (推荐)

建议配合系统计划任务，或直接使用 `-r cron` 长期运行。也可以使用系统自带的 crontab 配合参数 `-r once` 实现自动运行。

[参考安装 OpenWrt 服务脚本](https://github.com/joyanhui/ikuai-bypass/blob/main/golang_archive/example/script/AddOpenwrtService.sh)

### Windows

从 Releases 下载 Windows 版本的压缩包，解压后通过命令提示符 (CMD) 或 PowerShell 运行即可。也可以配合系统自带的任务计划程序使用 `-r once` 实现自动运行。

### macOS

根据芯片架构选择：
- Apple Silicon：下载 `darwin-arm64`
- Intel：下载 `darwin-amd64`

解压后在终端运行，也可以使用系统自带的 crontab 配合参数 `-r once` 实现自动运行。

### LXC / Alpine / musl 环境

如果你运行在 LXC、Alpine 或更偏向 musl 的轻量环境，优先使用：

```
ikuai-bypass-lxc-alpine-musl-amd64.tar.gz
```

这个包是为轻量 Linux 环境单独提供的 CLI 版本，与 `linux-amd64` 复用同一份二进制。

#### LXC 容器导入步骤

1. **下载并解压**：
   ```bash
   # 下载对应架构的 tar.gz 文件
   wget https://github.com/joyanhui/ikuai-bypass/releases/latest/download/ikuai-bypass-lxc-alpine-musl-amd64.tar.gz
   
   # 解压
   tar -xzf ikuai-bypass-lxc-alpine-musl-amd64.tar.gz
   ```

2. **放置二进制文件**：
   ```bash
   # 移动到系统路径
   mv ikuai-bypass /usr/local/bin/
   chmod +x /usr/local/bin/ikuai-bypass
   ```

3. **创建配置目录**：
   ```bash
   mkdir -p /etc/ikuai-bypass
   # 下载示例配置文件
   wget -O /etc/ikuai-bypass/config.yml https://raw.githubusercontent.com/joyanhui/ikuai-bypass/main/config.yml
   ```

4. **编辑配置文件**：
   ```bash
   vi /etc/ikuai-bypass/config.yml
   # 填写爱快地址、用户名、密码等信息
   ```

5. **运行测试**：
   ```bash
   # 单次运行测试
   ikuai-bypass -r once -c /etc/ikuai-bypass/config.yml
   ```

6. **配置为系统服务**（可选）：
   ```bash
   # 创建 systemd 服务文件
   cat > /etc/systemd/system/ikuai-bypass.service << 'EOF'
   [Unit]
   Description=iKuai Bypass
   After=network.target
   
   [Service]
   Type=simple
   ExecStart=/usr/local/bin/ikuai-bypass -r cron -c /etc/ikuai-bypass/config.yml
   Restart=always
   RestartSec=10
   
   [Install]
   WantedBy=multi-user.target
   EOF
   
   # 启用并启动服务
   systemctl daemon-reload
   systemctl enable ikuai-bypass
   systemctl start ikuai-bypass
   ```

> LXC / Alpine 包内已内置 OpenRC 服务脚本 `/etc/init.d/ikuai-bypass`，并默认加入 `/etc/runlevels/default/`。
> 容器启动后会自动以服务方式运行。如需禁用，删除 `/etc/runlevels/default/ikuai-bypass` 即可。

---

## Docker 部署

### 镜像信息

- **镜像地址**：[joyanhui/ikuai-bypass](https://hub.docker.com/r/joyanhui/ikuai-bypass)
- **基础镜像**：Alpine 3.20
- **支持架构**：`linux/amd64`, `linux/386`, `linux/arm64`, `linux/arm/v7`, `linux/arm/v6`, `linux/ppc64le`, `linux/riscv64`

### 目录结构

| 路径 | 说明 |
| :--- | :--- |
| `/etc/ikuai-bypass` | 配置目录（需要挂载） |
| `/etc/ikuai-bypass/config.yml` | 配置文件 |
| `/opt/ikuai-bypass/config.yml` | 内置配置模板 |
| `/usr/local/bin/ikuai-bypass` | 可执行文件 |

### 快速启动

```bash
# 拉取镜像
docker pull joyanhui/ikuai-bypass:latest

# 常规运行（定时任务 + WebUI）
docker run -d \
  --name ikuai-bypass \
  --restart=always \
  -p 19001:19001 \
  -v $(pwd)/data:/etc/ikuai-bypass \
  joyanhui/ikuai-bypass:latest

# 单次执行
docker run --rm \
  -v $(pwd)/data:/etc/ikuai-bypass \
  joyanhui/ikuai-bypass:latest -r once

# 仅运行 WebUI
docker run -d \
  --name ikuai-bypass-web \
  -p 19001:19001 \
  -v $(pwd)/data:/etc/ikuai-bypass \
  joyanhui/ikuai-bypass:latest -r web
```

### 配置文件自动初始化

容器启动时会自动检查配置文件：
- 如果 `/etc/ikuai-bypass/config.yml` 不存在，会自动从内置模板复制
- 你可以通过 WebUI 或直接编辑配置文件来修改配置

### 自定义启动参数

默认命令是 `-r cron`，你可以在 `docker run` 后追加参数覆盖：

```bash
# 使用 ipgroup 模式
docker run -d \
  --name ikuai-bypass \
  -v $(pwd)/data:/etc/ikuai-bypass \
  joyanhui/ikuai-bypass:latest -r cron -m ipgroup

# 使用自定义 cron 表达式（需在配置文件中设置）
docker run -d \
  --name ikuai-bypass \
  -v $(pwd)/data:/etc/ikuai-bypass \
  joyanhui/ikuai-bypass:latest -r cronAft
```

### Docker Compose

```yaml
version: "3.8"
services:
  ikuai-bypass:
    image: joyanhui/ikuai-bypass:latest
    container_name: ikuai-bypass
    restart: always
    volumes:
      - ./data:/etc/ikuai-bypass
    ports:
      - "19001:19001"
    command: ["-r", "cron"]
```

### 群晖 Docker 部署

```yaml
version: '3.8'
services:
  ikuai-bypass:
    image: joyanhui/ikuai-bypass:latest
    container_name: ikuai-bypass
    privileged: true
    environment:
      TZ: "Asia/Shanghai"
    volumes:
      - /volume1/docker/ikuai-bypass/data/:/etc/ikuai-bypass
    ports:
      - "19001:19001"
    command: ["-r", "cron"]
    tty: true
```

### 爱快内置 Docker 部署

爱快路由器内置 Docker 功能，可以直接在爱快内部署：

1. **下载镜像**：进入爱快后台 → 高级应用 → Docker → 镜像管理 → 添加，选择"镜像库下载"，搜索 `joyanhui/ikuai-bypass`，下载 `latest` 标签

2. **创建容器**：
   - 容器名称：`ikuai-bypass`
   - 镜像：选择刚下载的镜像
   - 重启策略：`always`
   - 卷映射：`/etc/ikuai-bypass` → 选择一个本地目录存放配置
   - 端口映射：`19001` → `19001`（如需 WebUI）

3. **配置文件**：首次启动后会自动生成配置文件，可通过 WebUI 修改

### 常见问题

<details>
<summary><b>Q: 如何更新规则？</b></summary>

容器默认以 `cron` 模式运行，会按照配置文件中的 `cron` 字段定时更新。你也可以手动触发：

```bash
docker exec ikuai-bypass ikuai-bypass -r once -c /etc/ikuai-bypass/config.yml
```

或者重启容器：

```bash
docker restart ikuai-bypass
```
</details>

<details>
<summary><b>Q: 如何查看日志？</b></summary>

```bash
docker logs -f ikuai-bypass
```

或通过 WebUI 查看实时日志。
</details>

<details>
<summary><b>Q: 如何升级版本？</b></summary>

```bash
docker pull joyanhui/ikuai-bypass:latest
docker stop ikuai-bypass
docker rm ikuai-bypass
# 重新运行 docker run 命令，使用相同的卷映射
```

配置文件会保留在挂载的卷中。
</details>

<details>
<summary><b>Q: 不需要 WebUI 怎么办？</b></summary>

移除 `-p 19001:19001` 端口映射即可。或者将 `webui.enable` 设置为 `false`。
</details>

---

## 注意事项

- `clean` 模式务必显式指定 `-tag`
- `tag` 不宜过长，爱快对规则名长度有限制（建议不超过 11 个字符，系统会自动添加 `IKB` 前缀）
- WebUI 端口默认是 `19001`
- 如果远程订阅下载失败，程序会优先保留旧规则，避免误清空

---

## 更新与发布

发布包、GUI 安装包、Docker 镜像都在：[Releases](https://github.com/joyanhui/ikuai-bypass/releases)

如果你关心构建矩阵、Docker 发布和预发布规则，可查看：[docs/release.md](./docs/release.md)

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
