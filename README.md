# iKuai Bypass

![iKuai](https://img.shields.io/badge/Router-iKuai-brightgreen) ![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg) ![Rust](https://img.shields.io/badge/Language-Rust-orange)

**iKuai Bypass** 是专为爱快路由器打造的自动化分流规则同步工具。它会模拟爱快 Web 管理界面的行为，把远程订阅的 IP / 域名列表同步到路由器的分流设置里，实现自动化分流更新。

当前仓库主线已经切换为 **Rust 版本**，提供两种使用方式：

- `CLI`：适合服务器、OpenWrt、Docker、计划任务环境
- `GUI`：基于 Tauri 的桌面应用，适合直接在本机可视化操作

旧的 Go/Fyne 版本代码、历史文档和旧 CI 已归档到 [golang_archive](/home/y/myworkspace/ikuai-bypass/golang_archive)。

> 如果这个项目对你有帮助，请点个 Star。

---

## 分流模式说明

本项目主要支持两类常见用法。

### 1. 自定义运营商分流模式

适用场景：追求稳定性、希望终端无感分流。

实现方式：

- 把 OpenWrt 或其他旁路设备视为爱快里的一个“虚拟运营商”
- 将订阅的目标 IP 列表同步到爱快“自定义运营商”
- 由爱快把这些目标流量转发到旁路设备

特点：

- 普通流量仍可走主线路
- 终端设备不需要改网关
- 适合长期自动维护

参考：

- [查看实现方式](https://dev.leiyanhui.com/route/ikuai-bypass-joyanhui/)
- [恩山教程](https://www.right.com.cn/forum/thread-8252571-1-1.html)

### 2. IP 分组 / 端口分流模式

适用场景：简单旁路由方案，逻辑直接。

实现方式：

- 先把订阅 IP 列表同步到爱快的 IP 分组
- 再使用端口分流，把命中的目标流量转到指定外网线路或下一跳网关

参考：

- [实现方式参考](https://github.com/joyanhui/ikuai-bypass/issues/7)
- [恩山教程](https://www.right.com.cn/forum/thread-8288009-1-1.html)

---

## 主要功能

- 自动同步自定义运营商、IP 分组、IPv6 分组、域名分流、端口分流
- 支持 `once / cron / cronAft / clean / web`
- 支持 WebUI 可视化配置
- 支持桌面 GUI
- 支持 Docker
- 支持 LXC / Alpine / musl CLI 包
- 支持多平台多架构发布

当前 Linux CLI 发布默认优先使用 `musl` 静态构建，便于在更多轻量环境中直接运行。

---

## 快速上手

### 1. 下载

从 [Releases](https://github.com/joyanhui/ikuai-bypass/releases) 下载适合你系统的版本。

常见文件类型：

- CLI：`ikuai-bypass-<platform-arch>.zip`
- LXC / Alpine / musl：`ikuai-bypass-lxc-alpine-musl-amd64.tar.gz`
- GUI：Windows / AppImage / macOS 安装包

说明：

- Linux `amd64 / 386 / arm / arm64 / riscv64` CLI 现统一优先使用 `musl` 构建
- `linux-ppc64le` 当前使用 `gnu` 目标构建（`powerpc64le-unknown-linux-gnu`）
- `lxc-alpine-musl-amd64` 与 `linux-amd64` 复用同一份二进制，只是额外提供了更适合 LXC / Alpine 场景的打包格式

### 2. 配置

编辑 `config.yml`，至少填写这些内容：

```yaml
ikuai-url: http://192.168.9.1
username: admin
password: your_password
cron: 0 7 * * *
custom-isp:
  - tag: 国内IP列表
    url: https://raw.githubusercontent.com/Loyalsoldier/geoip/release/text/cn.txt
webui:
  port: "19001"
  user: admin
  pass: admin888
  enable: true
```

完整示例请直接参考 [config.yml](/home/y/myworkspace/ikuai-bypass/config.yml)。

### 3. 运行

标准模式：

```bash
./ikuai-bypass -r cron -c ./config.yml
```

单次执行：

```bash
./ikuai-bypass -r once -c ./config.yml
```

仅启动 WebUI：

```bash
./ikuai-bypass -r web -c ./config.yml
```

清理模式：

```bash
./ikuai-bypass -r clean -tag cleanAll -c ./config.yml
```

---

## WebUI 与 GUI

### WebUI

在配置文件里启用 `webui.enable: true` 后，可以启动 WebUI：

```bash
./ikuai-bypass -r web -c ./config.yml
```

然后在浏览器访问：

```text
http://你的IP:19001
```

默认端口是 `19001`，用户名和密码由 `config.yml` 的 `webui.user` 与 `webui.pass` 控制。

### GUI

GUI 是桌面版入口，适合不想手动敲命令的用户。

GUI 里可以完成这些事：

- 运行一次
- 启动 / 停止定时任务
- 修改配置
- 文本编辑 YAML
- 查看实时日志

如果你使用 GUI，一般不需要自己手动拼 CLI 参数。

---

## CLI 参数说明

常用参数如下：

| 参数 | 说明 |
| :--- | :--- |
| `-c` | 配置文件路径 |
| `-m` | 分流模块选择 |
| `-r` | 运行模式 |
| `-tag` | 清理模式使用的标签关键词 |
| `-login` | 临时覆盖爱快登录信息 |
| `-exportPath` | 导出路径 |
| `-isIpGroupNameAddRandomSuff` | IP 分组名称是否增加随机后缀 |

### 运行模式

| 模式 | 说明 |
| :--- | :--- |
| `cron` | 立即执行一次，然后进入定时任务等待 |
| `cronAft` | 不立即执行，直接进入定时任务等待 |
| `once` / `nocron` / `1` | 立即执行一次并退出 |
| `clean` | 清理本工具创建的规则 |
| `web` | 启动 WebUI |

### 分流模块

| 模块 | 说明 |
| :--- | :--- |
| `ispdomain` | 默认模式，偏向自定义运营商 / 域名分流 |
| `ipgroup` | IP 分组 |
| `ipv6group` | IPv6 分组 |
| `ii` | 混合模式 |
| `ip` | IPv4 + IPv6 分组 |
| `iip` | 更完整的混合模式 |

---

## Docker

Docker 镜像同样适合长期运行。

默认行为：

- 配置目录：`/etc/ikuai-bypass`
- 配置文件：`/etc/ikuai-bypass/config.yml`
- 首次启动会自动复制默认模板
- 默认命令：`-r cron`
- 如果启用了 WebUI，默认端口通常为 `19001`

拉取镜像：

```bash
docker pull joyanhui/ikuai-bypass:latest
```

常规运行：

```bash
docker run -d \
  --name ikuai-bypass \
  --restart=always \
  -p 19001:19001 \
  -v $(pwd)/data:/etc/ikuai-bypass \
  joyanhui/ikuai-bypass:latest
```

单次执行：

```bash
docker run --rm \
  -v $(pwd)/data:/etc/ikuai-bypass \
  joyanhui/ikuai-bypass:latest -r once
```

仅运行 WebUI：

```bash
docker run -d \
  --name ikuai-bypass-web \
  -p 19001:19001 \
  -v $(pwd)/data:/etc/ikuai-bypass \
  joyanhui/ikuai-bypass:latest -r web
```

Compose 示例：

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

---

## LXC / Alpine / musl

如果你运行在 LXC、Alpine 或更偏向 musl 的轻量环境，优先使用：

```text
ikuai-bypass-lxc-alpine-musl-amd64.tar.gz
```

这个包是为轻量 Linux 环境单独提供的 CLI 版本。

---

## 部署建议

### Linux / OpenWrt

建议配合系统计划任务，或直接使用 `-r cron` 长期运行。

### Windows

下载对应压缩包后解压，使用 CMD 或 PowerShell 运行即可。也可以配合任务计划程序使用 `-r once`。

### macOS

根据芯片选择：

- Apple Silicon：`darwin-arm64`
- Intel：`darwin-amd64`

### Docker

适合 NAS、服务器、群晖、长期后台运行场景。

---

## 注意事项

- `clean` 模式务必显式指定 `-tag`
- `tag` 不宜过长，爱快对规则名长度有限制
- WebUI 端口默认是 `19001`
- 如果远程订阅下载失败，程序会优先保留旧规则，避免误清空

---

## 更新与发布

发布包、GUI 安装包、Docker 镜像都在：

- [Releases](https://github.com/joyanhui/ikuai-bypass/releases)

如果你关心构建矩阵、Docker 发布和预发布规则，可查看：

- [docs/release.md](/home/y/myworkspace/ikuai-bypass/docs/release.md)

---

## 交流与反馈

- [GitHub Discussions](https://github.com/joyanhui/ikuai-bypass/discussions)
- [Issues](https://github.com/joyanhui/ikuai-bypass/issues)

感谢所有参与反馈、测试和贡献的用户与开发者。
