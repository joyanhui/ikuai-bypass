# iKuai Bypass

![iKuai](https://img.shields.io/badge/Router-iKuai-brightgreen) ![License](https://img.shields.io/badge/License-MIT-blue) ![Go](https://img.shields.io/badge/Language-Go-blue)

**iKuai Bypass** 是一个专为爱快（iKuai）路由器开发的自动化分流规则同步工具。它通过模拟 Web 管理界面行为，将远程订阅的 IP/域名列表自动同步到路由器的分流设置中，实现精准、高效的自动化流量调度。 [旧版本说明](README_bakcup.md)

> **如果这个项目对你有帮助，请点个 ⭐️ Star！**

---

## 爱快喜闻乐见分流模式解析

本项目支持两种主流的分流实现方案，您可以根据自己的网络拓扑选择最合适的模式。

### 1. 自定义运营商分流模式 (推荐)
**适用场景：** 追求极致稳定性、网络自愈、终端无感分流。

*   **实现逻辑：**
    这种模式下，iKuai 将 OpenWrt（或其他网关）视为一个“虚拟的上级运营商”。
    1.  **链路设计**：OpenWrt 作为 iKuai 的下级设备接收流量，处理后再将出口流量“绕回”给 iKuai 的物理 WAN 口。
    2.  **规则同步**：本工具将目标 IP 列表导入 iKuai 的“自定义运营商”。iKuai 会认为这些 IP 属于该“虚拟运营商”，从而将流量转发给 OpenWrt。
*   **核心优势：**
    *   **极高可靠性**：OpenWrt 宕机只会导致被分流的流量中断，国内/普通流量依然通过主线直连，不会全家断网。
    *   **配置无感**：终端设备无需更改网关配置，完全由 iKuai 在内核层级完成调度。
    *   **性能优异**：国内网站直连速度最快，旁路仅处理特定流量。
*   **参考文档**：[查看具体实现方式](https://dev.leiyanhui.com/route/ikuai-bypass-joyanhui/) 或 [恩山eezz的教程](https://www.right.com.cn/forum/thread-8252571-1-1.html)。

<details>
<summary>点击这里展开查看详细图文说明</summary>
<img src="assets/img.png"  alt="图文说明">
</details>

### 2. IP 分组与端口分流模式 (传统模式)
**适用场景：** 简单的旁路由方案，逻辑直接。

*   **实现逻辑：**
    1.  **IP 分组**：本工具将订阅的 IP 列表同步到 iKuai 的“IP 分组”中。
    2.  **策略路由**：利用 iKuai 的“端口分流”功能，匹配目标地址为该分组的流量，将其“下一跳网关”指向 OpenWrt 的 IP。
*   **特点**：配置简单直接，但在 OpenWrt 宕机时，匹配到该分组的规则将无法上网（当然也无伤大雅）。
*   **参考文档**：[实现方式参考](https://github.com/joyanhui/ikuai-bypass/issues/7) 或 [恩山y2kji的教程](https://www.right.com.cn/forum/thread-8288009-1-1.html)。

---

## 主要功能特性

-   🚀 **高并发处理**：采用协程并发同步运营商/域名规则，同步速度极快。
-   🛡️ **安全平滑更新**：默认采用“先增后删”策略，确保在规则更新期间路由器分流功能不中断。
-   🌐 **多维度支持**：涵盖运营商分流 (IPv4/IPv6)、域名分流、端口分流 (Next-hop)。
-   📅 **全自动运营**：内置 Cron 计划任务，支持多配置文件并发运行。
-   🛠️ **工具链完备**：支持一键清理、单次运行、规则导出（导出为爱快可导入的 TXT 格式）。
-   💻 **广泛兼容**：提供全平台架构支持（arm, mips, amd64, 386 等）。

---

## 快速上手

### 1. 下载与配置
1.  从 [Releases](https://github.com/joyanhui/ikuai-bypass/releases) 下载对应系统的二进制文件。
2.  编辑 `config.yml`，填写爱快登录信息及订阅 URL：
    ```yaml
    ikuai-url: http://192.168.9.1
    username: admin
    password: your_password
    cron: "0 7 * * *" # 每天早上 7 点更新
    ```

### 2. 运行模式
```bash
# 标准模式（先执行一次，后进入定时任务）
./ikuai-bypass -r cron

# 调试/单次模式（立即执行并退出）
./ikuai-bypass -r once

# 清理模式（删除所有 IKUAI_BYPASS 标记的规则）
./ikuai-bypass -r clean
```

---

## 参数说明 (CLI Flags)

| 参数 | 说明 | 示例/取值 |
| :--- | :--- | :--- |
| `-c` | 配置文件路径 | `-c ./config.yml` |
| `-m` | **分流模块选择** | `ispdomain` (默认), `ipgroup`, `ipv6group`, `ii` (混合) |
| `-r` | 运行模式 | `cron`, `once`, `clean`, `exportDomainSteamToTxt` |
| `-tag` | 清理模式下的标签关键词 | 默认为 `cleanAll` |
| `-login` | 覆盖配置文件登录信息 | `http://IP,username,password` |
| `-delOldRule`| 删除旧规则的时机 | `after` (更新后删), `before` (更新前删) |

---

## 部署方案

<details>
<summary><b>Linux / OpenWrt (推荐)</b></summary>
建议通过服务脚本安装为开机自启。
<a href="https://github.com/joyanhui/ikuai-bypass/blob/main/example/script/AddOpenwrtService.sh">参考安装脚本</a>
</details>

<details>
<summary><b>Docker 部署</b></summary>
我没有专门去维护一个docker镜像，因为这个项目没有任何外部依赖只需要一个二进制程序和一个配置文件就可以运行了。你需要先从 Releases 下载对应系统的二进制文件。然后随便找一个linux的docker镜像就可以了。
<code>
docker run -itd --name ikuai-bypass --privileged=true --restart=always \
    -v ~/ikuai-bypass/:/opt/ikuai-bypass/ \
    alpine:latest /opt/ikuai-bypass/ikuai-bypass -c /opt/ikuai-bypass/config.yml -r cron
</code>
</details>

<details>
<summary><b>iKuai Docker 环境</b></summary>
使用 <code>alpine:latest</code> 镜像，挂载可执行文件，启动命令设置为：
<code>/bin/sh -c "chmod +x /opt/ikuai-bypass/ikuai-bypass && /opt/ikuai-bypass/ikuai-bypass -r cron -c /opt/ikuai-bypass/config.yml"</code>
</details>

---

## 更新日志

-   **2026-01-05 (v4.0.0)**: 重构项目目录结构，采用标准 pkg 布局；修复了端口分流在多规则下可能被覆盖的 Bug。
-   **2025-03-23**: 增加 IPv6 分组支持。
-   **2024-10-04**: 优化了大规模域名列表同步的稳定性。

---

## 交流与反馈

-   **交流讨论**：[GitHub Discussions](https://github.com/joyanhui/ikuai-bypass/discussions) 或 恩山无线论坛。
-   **Bug 反馈**：[Issues](https://github.com/joyanhui/ikuai-bypass/issues)。
-   **致谢**：感谢 [ztc1997](https://github.com/ztc1997/ikuai-bypass/) 的核心功能贡献，以及所有提供 PR 的开发者。

> **如果这个项目对你有帮助，请点个 ⭐️ Star！**
