# iKuai Bypass

![iKuai](https://img.shields.io/badge/Router-iKuai-brightgreen) ![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg) ![Go](https://img.shields.io/badge/Language-Go-blue)

**iKuai Bypass** 是专为爱快路由器打造的自动化分流规则同步工具。通过模拟 Web 管理界面，自动将远程订阅的 IP/域名列表同步到路由器，实现精准流量调度。支持自定义运营商、IP/IPv6分组、域名分流、端口分流等多种模式，具备高并发处理、平滑更新、定时任务、可视化配置界面等特性，兼容全平台多架构。。[旧版本说明](README_bakcup.md)


> **如果这个项目对你有帮助，请点个 ⭐️ Star！** star数是作者唯一的维护动力。


> 关于dns分流解析，建议用 ADGuard home自建，这里有一个本人利用githubaction自动维护相关规则文件的adguardhome规则.[[joyanhui/adguardhome-rules]](https://github.com/joyanhui/adguardhome-rules)（规则文件在release_file分支48小时更新一次）。并提供的教程，可以简单自动更新dns分流解析规则，广告屏蔽，以及ipv4优先等功能

---

## 可视化 界面展示

v4.1.0 版本新增了基于 Web 的可视化配置界面，支持在线配置和命令参数生成。当然你可以继续使用纯cli和手动修改配置文件的方式使用。

![webui-screenshot.gif](assets/webui-screenshot.gif)


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
<img src="assets/img.png"  alt="自定义运营商分流模式拓扑图">
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

-   🚀 **高并发处理**：采用协程并发同步运营商/域名规则，同步速度提升。
-   🛡️ **安全平滑更新**：默认采用“先增后删”策略，确保在规则更新期间路由器分流功能不中断。
-   🌐 **多维度支持**：涵盖运营商分流 (IPv4/IPv6分流)、ip分组、域名分流、端口分流 (Next-hop)。
-   📅 **全自动运营**：内置 Cron 计划任务，兼容系统计划任务，支持多配置文件并发运行。
-   🛠️ **工具链完备**：支持一键清理、单次运行、规则导出（导出为爱快可导入的 TXT 格式）。
-   💻 **广泛兼容**：提供全平台（win,linux,macos,freebsd）多架构支持（arm, mips, amd64, 386 等）。

---

## 快速上手

### 版本选择
- v4.2.0 增加了 按IP分组名字搜索来源IP功能 [#99](https://github.com/joyanhui/ikuai-bypass/issues/99)以及 修复了stream-ipport配置为空的时候依旧添加分流规则的bug [#101](https://github.com/joyanhui/ikuai-bypass/issues/101)。此版本未经过全面测试，请谨慎使用。
- v4.1.0 增加了启用的基于web的可视化配置界面和命令参数生成界面。稳定版本只存在不影响使用的小问题。
- v4.0.1 重构了项目结构，修复了端口分流只保留一条的bug[#96](https://github.com/joyanhui/ikuai-bypass/issues/96)；优化了IP和IPv6分组更新流程[#97](https://github.com/joyanhui/ikuai-bypass/pull/97)。
- v3.0.0 版本 增加了ipv6分组 此功能由 [[dscao]](https://github.com/dscao) 提供。本版功能完备但是存在一处bug[#96](https://github.com/joyanhui/ikuai-bypass/issues/96)
- v2.1.2-alpha1 虽然是alpha版，但功能稳定 存在少量不影响使用的bug

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

# WebUI 模式（启动可视化配置界面）
./ikuai-bypass -r web

# 调试/单次模式（立即执行并退出）
./ikuai-bypass -r once

# 清理模式（删除所有 IKUAI_BYPASS 标记的规则）
./ikuai-bypass -r clean
```

### 3. WebUI 配置与使用
在 `config.yml` 的 `webui` 配置项中设置端口、用户名和密码。运行 `./ikuai-bypass -r web` 启动后，浏览器访问 `http://IP:19000` 即可在线修改配置和生成命令参数。

---


## 参数说明 (CLI Flags)

| 参数 | 说明 | 示例/取值 |
| :--- | :--- | :--- |
| `-c` | 配置文件路径 | `-c ./config.yml` |
| `-m` | **分流模块选择** | `ispdomain` (默认), `ipgroup`, `ipv6group`, `ii` (混合), `ip` (ipv4和ipv6分组) ，`iip` (ii+ip混合) |
| `-r` | 运行模式 | 见下表 |
| `-tag` | 清理模式下的标签关键词 | 默认为 `cleanAll`备注和名字中包含IKUAI_BYPASS和IKB的规则和分组|
| `-login` | 覆盖配置文件登录信息 | `http://IP,username,password` |
| `-delOldRule`| 删除旧规则时机 | `after` (默认-更新后删), `before` (更新前删)。**注意：在 ikuaiV4 中此参数被移除，强制使用 before** |
| `-exportPath` | 域名分流规则导出文件路径 | 默认为 `/tmp` |
| `-isIpGroupNameAddRandomSuff` | IP分组名称是否增加随机数后缀 | `1` (添加), `0` (不添加)。仅 ipgroup 模式有效 |

### 运行模式 (`-r`) 详细说明

| 模式 | 名称 | 说明 |
| :--- | :--- | :--- |
| `cron` | 计划任务模式 | **默认模式**。立即执行一次更新，随后进入定时任务等待模式。若启用了 WebUI 则同步启动。 |
| `cronAft` | 延迟计划任务 | 不立即执行更新，直接进入定时任务等待模式。若启用了 WebUI 则同步启动。 |
| `once` / `nocron` / `1` | 单次模式 | 立即执行一次规则更新，完成后立即退出程序。 |
| `clean` | 清理模式 | 删除所有带有 `IKUAI_BYPASS_` 前缀（或 `-tag` 指定）的规则。 |
| `web`v4.1版本才有的功能 | WebUI 模式 | 启动可视化 Web 管理界面，用于在线修改配置。不做其他操作 |
| `exportDomainSteamToTxt` | 导出模式 | 将域名分流规则导出为爱快兼容的 TXT 格式，方便手动导入。 |

---

## 部署方案

<details>
<summary><b>Linux / OpenWrt (推荐)</b></summary>
建议通过服务脚本安装为开机自启。(或者使用系统自带的crontab 配合参数 `-r once` 实现自动运行)
<a href="https://github.com/joyanhui/ikuai-bypass/blob/main/example/script/AddOpenwrtService.sh">参考安装openwrt脚本</a>
</details>

<details>
<summary><b>Windows</b></summary>
从 Releases 下载 Windows 版本的压缩包，解压后通过命令提示符 (CMD) 或 PowerShell 运行即可。（或者使用系统自带的 计划任务管理器 配合参数 `-r once` 实现自动运行）<br> 
注意：由于使用了 UPX 压缩且未加壳，部分杀软可能会误报。如果不信任，建议自行克隆代码并编译。作者没有 Windows 环境，不负责处理此类误报问题。详见 <a href="https://github.com/joyanhui/ikuai-bypass/issues/6">#6</a>。
</details>

<details>
<summary><b>macOS</b></summary>
根据您的芯片架构（Apple Silicon 下载 <code>darwin-arm64</code>，Intel 下载 <code>darwin-amd64</code>）下载对应的压缩包，解压后在终端运行。(或者使用系统自带的crontab 配合参数 `-r once` 实现自动运行)
</details>

<details>
<summary><b>Docker 部署</b></summary>
我没有专门去维护一个docker镜像，因为这个项目没有任何外部依赖只需要一个二进制程序和一个配置文件就可以运行了。你需要先从 Releases 下载对应系统的二进制文件。然后随便找一个linux的docker镜像就可以了。
<code>
docker run -itd --name ikuai-bypass --privileged=true --restart=always \
    -p 19000:19000 \
    -v ~/ikuai-bypass/:/opt/ikuai-bypass/ \
    alpine:latest /opt/ikuai-bypass/ikuai-bypass -c /opt/ikuai-bypass/config.yml -r cron
</code>
注意：<code>-p 19000:19000</code> 是 WebUI 管理界面的端口映射，如果不需要使用 WebUI 可以移除此参数。默认端口为 19000，可在配置文件中修改。
</details>

<details>
<summary><b>iKuai Docker 环境</b></summary>
使用 <code>alpine:latest</code> 镜像，挂载可执行文件，启动命令设置为：
<code>/bin/sh -c "chmod +x /opt/ikuai-bypass/ikuai-bypass && /opt/ikuai-bypass/ikuai-bypass -r cron -c /opt/ikuai-bypass/config.yml"</code>
注意：需要在 Docker 配置中添加端口映射 <code>19000:19000</code> 以访问 WebUI 管理界面。默认端口为 19000，可在配置文件中修改。
</details>

<details>
<summary><b>群晖环境docker</b></summary>
使用 <code>compose.yaml</code> 内容为：

```
version: '3.8'
services:
  ikuai-bypass:
    image: dscao/ikuai-bypass
    container_name: ikuai-bypass
    privileged: true
    environment:
      TZ: "Asia/Shanghai"
    volumes:
      - /volume1/docker/ikuai-bypass/data/:/opt/ikuai-bypass
    ports:
      - "19000:19000"
    command: sh -c "/app/ikuai-bypass -c /opt/ikuai-bypass/config.yml -r cron -m ipv6group & sleep 30 ; /app/ikuai-bypass -c /opt/ikuai-bypass/config2.yml -r cron -m ii ; wait"
    tty: true
```
注意：<code>ports: - "19000:19000"</code> 是 WebUI 管理界面的端口映射，如果不需要使用 WebUI 可以移除此配置。默认端口为 19000，可在配置文件中修改。
</details>

---

## 更新日志
- 2026-06-15 因爱快v4版本禁止同tagname的分流规则存在，所以暂时移除delOldRule参数的支持，强制使用before删除旧规则模式。
- 2026-02-09 增加ispgroup和ipv4group ipv6group 三分流模块 一起使用模式 `-m iip`模式 [#104](https://github.com/joyanhui/ikuai-bypass/issues/104)
- 2026-02-09 支持自定义域名清单文本规则中 用 #开头的行注释 并忽略包含_的域名。
- 2026-02-09 准备支持爱快4.x内侧版本 
- 2026-01-15 重构部分代码结构，去掉对GitHub路径依赖，拆分utils包等。
- 2026-01-15 fix [#101](https://github.com/joyanhui/ikuai-bypass)， stream-ipport配置为空的时候依旧添加分流规则的bug，新增可选配置项 stream-ipport[].opt-tagname
- 2026-01-15 features  [#99](https://github.com/joyanhui/ikuai-bypass/issues/99)  可以按照ip分组名字自动搜索来源ip 不 新增配置项目 src-addr-opt-ipgroup（最终名）支持端口分流和域名分流
- 2026-01-07 增加中文可视化界面 创建带参数的命令行 以及 在线可视化构建配置文件。
- 2026-01-06 优化ip、ipv6分组的更新流程，先获取到新数据后删除旧分组，再增加新分组数据。分组名称保持统一。delOldRule与ip、ipv6分组不再有关联。[97](https://github.com/joyanhui/ikuai-bypass/pull/97)
- 2026-01-05 代码目录结构调整 修复端口分流配置只能添加最后的一条的bug[#96](https://github.com/joyanhui/ikuai-bypass/issues/96)
- 2025-04-23 部分代码规范性处理以及nilness的逻辑修复
- 2025-04-23 增加开关isIpGroupNameAddRandomSuff [[#76]](https://github.com/joyanhui/ikuai-bypass/issues/76)
- 2025-04-23 修复域名分流规则末行空行的bug [[#24]](https://github.com/joyanhui/ikuai-bypass/issues/24)
- 2025-03-25 增加端口分流时能够选择更多参数：负载模式、线路绑定，修复完善delOldRule参数，对于ip分组、ipv6分组及端口分流都默认为先增加后删除，防止增加失败导致原来的规则丢失.
- 2025-03-23 增加ipv6分组
- 2024-10-04 提供完整的最新的config.yml 文件，供参考
- 2024-10-04 修复端口分流规则自动添加未能关联ip分组的bug，本次修改更新了一下config.yml的默认内容，请注意更新您的配置文件.[[#30]](https://github.com/joyanhui/ikuai-bypass/issues/30)
- 2024-10-04 修复清理模式的删除规则问题 [[#27#issuecomment-2388114699]](https://github.com/joyanhui/ikuai-bypass/issues/27#issuecomment-2388114699)
- 2024-10-04 ip分组第一行的备注问题 [[#22]](https://github.com/joyanhui/ikuai-bypass/issues/22)
- 2024-10-04 修复 卡`ip分组== 正在查询  备注为: IKUAI_BYPASS_ 的ip分组规则` 的bug [[#24]](https://github.com/joyanhui/ikuai-bypass/issues/24) [[#27]](https://github.com/joyanhui/ikuai-bypass/issues/27)
- 2024-10-04 修复运营商分流的ip列表会添加一个空行的bug [[#24]](https://github.com/joyanhui/ikuai-bypass/issues/24)
- 2024-06-29 修复清理模式无法清理ip分组和端口分流规则的问题 v2.0.1以后版本有效
- 2024-06-29 增加运营商和域名分流规则旧规则删除模式参数 `-delOldRule` [[#15]](https://github.com/joyanhui/ikuai-bypass/issues/15) v2.0.1以后版本有效
- 2024-06-29 修改-m参数默认值错误导致的不配置-m参数无法执行的问题 构建 v2.0.0-beta2 版本 这是一个未经过详细测试的版本，请谨慎使用.
- 2024-05-26 修复OLOrz996分支里端口分流规则模式无法删除的bug
- 2024-05-26 合并ztc1997的ip分组和下一跳网关功能[[#7]](https://github.com/joyanhui/ikuai-bypass/issues/7) 增加了 `-m`参数
- 2024-05-26 命令行参数增加-login参数，可以覆盖配置文件内的爱快地址和用户名密码
- 2024-03-23 增加域名分流规则导出为爱快兼容的可导入的txt文件 [[5#2016320900]](https://github.com/joyanhui/ikuai-bypass/issues/5#issuecomment-2016320900)
- 2024-03-23 尝试修复列表太多导致爱快处理超时的问题 [[#5]](https://github.com/joyanhui/ikuai-bypass/issues/5)
- 2024-03-07 openwrt服务安装脚本增加无代理环境安装
- 2024-02-25 增加去广告功能演示规则 [[参考]](https://github.com/joyanhui/ikuai-bypass/blob/main/config.yml)
- 2024-02-7 添加一个openwrt下开机自动运行 [[参考脚本]](https://github.com/joyanhui/ikuai-bypass/blob/main/example/script/AddOpenwrtService.sh)
- 2024-02-1 优化清理模式的提示信息，增加`once`或 `1`模式等同于nocron模式
- 2024-02-1 某一分组规则更新失败导致相关的旧规则被删除的bug [[#3]](https://github.com/joyanhui/ikuai-bypass/issues/3)
- 2024-02-1 清理模式增加附加参数`-tag` 可以清理全部备注名包含`IKUAI_BYPASS`的分流规则，或者指定备注名全程或者后缀名的分流规则
- 旧的更新记录没啥价值也未单独记，小工具代码简单，请参考commit记录
---

## 赞助支持

如果您觉得这个项目对您有所帮助，并且愿意支持我的持续开发和维护工作，非常感谢您的慷慨！

为使用者较多，不忍心停止维护，虽世道艰难也一直在坚持维护，如果您的一份心意能让我更有动力继续完善这个工具。

- **TRX (Tron TRC20) 钱包地址**：`TLiv9F6i38uZEGdp8VoB5qLxJx43aV9XSZ`

当然，您也可以通过在 GitHub 上给项目点一个 ⭐️ Star 来支持我，这对项目的发展同样非常重要！

---

## 交流与反馈

-   **交流讨论**：[GitHub Discussions](https://github.com/joyanhui/ikuai-bypass/discussions) 或 恩山无线论坛。
-   **Bug 反馈**：[Issues](https://github.com/joyanhui/ikuai-bypass/issues)。
-   **致谢**：感谢 [ztc1997](https://github.com/ztc1997/ikuai-bypass/) 的初始版本思路，以及所有提供 PR 的开发者。

> **如果这个项目对你有帮助，请点个 ⭐️ Star！**
