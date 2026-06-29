---
title: 分流模式：运营商 vs IP 分组
nav_order: 4
---

# 爱快两种分流模式解析

本项目支持两种主流的分流实现方案，您可以根据自己的网络拓扑选择最合适的模式。运行 CLI 时通过 `-m` 参数选择分流模式，详见 [CLI 参数说明](cli-params.html#分流模式--m)。

{% include head_custom_mermaid.html %}

## 网络拓扑总览

<pre class="mermaid">
graph TD
    Client1[内网客户端&lt;br&gt;IP: 192.168.1.X]
    Client2[内网客户端&lt;br&gt;IP: 192.168.1.X]
    Client3[内网客户端&lt;br&gt;IP: 192.168.1.X]
    iKuai[iKuai 主路由&lt;br&gt;网关: 192.168.1.1]
    Check{iKuai 端口分流规则 / 域名分流}
    WAN1_ISP[爱快物理 WAN1 接口&lt;br&gt;拨号/静态公网]
    China_Web[国内网络 / 目标网站&lt;br&gt;百度、淘宝、微信等]
    Op_WAN[OpenWrt 虚拟运营商线路&lt;br&gt;爱快分流目的地]
    OpenWrt[OpenWrt 副路由&lt;br&gt;IP: 192.168.1.2&lt;br&gt;核心：特殊插件]
    Op_Out[OpenWrt 流量流出&lt;br&gt;网关指向 iKuai]
    WAN2_ISP[爱快物理 WAN2 接口&lt;br&gt;专线/特殊出口]
    Global_Web[海外/特殊网络&lt;br&gt;GitHub、Google 等]

    Client1 -->|发出流量请求| iKuai
    Client2 -->|发出流量请求| iKuai
    Client3 -->|发出流量请求| iKuai
    iKuai --> Check
    Check -->|方式一：匹配国内IP/域名| WAN1_ISP
    WAN1_ISP -->|直连高带宽| China_Web
    Check -->|方式二：匹配特殊IP/域名| Op_WAN
    Op_WAN -->|下一跳网关| OpenWrt
    OpenWrt -->|插件处理| Op_Out
    Op_Out -->|重新回到爱快物理接口| WAN2_ISP
    WAN2_ISP -->|安全访问| Global_Web

    style Client fill:#f9f,stroke:#333,stroke-width:2px
    style iKuai fill:#bbf,stroke:#333,stroke-width:2px
    style OpenWrt fill:#fbb,stroke:#333,stroke-width:2px
    style Check fill:#ff9,stroke:#333,stroke-width:2px
</pre>

---

### 1. 自定义运营商分流模式 (推荐)

**适用场景：** 追求极致稳定性、网络自愈、终端无感分流。（需要多网卡或能添加虚拟网卡）

**实现逻辑：**
这种模式下，iKuai 将 OpenWrt（旁路由）视为一个"虚拟的上级运营商"。

1. **链路设计**：OpenWrt 作为 iKuai 的下级设备接收流量，处理后再将出口流量"绕回"给 iKuai 的 WAN 口。
2. **规则同步**：本工具将目标 IP 列表导入 iKuai 的"自定义运营商"。iKuai 会认为这些 IP 属于该"虚拟运营商"，从而将流量转发给 OpenWrt。

**数据流向：**

```
客户端 → iKuai 路由 → 端口分流（下一跳指向 OpenWrt）
                        → OpenWrt 插件处理
                        → 重新交回 iKuai 物理 WAN2/WAN1
                        → 海外/特殊目标网站
```

**核心优势：**

- **极高可靠性**：OpenWrt 宕机只会导致被分流的流量中断，普通流量依然通过主线直连，不会全家断网。
- **配置无感**：终端设备无需更改网关配置，完全由 iKuai 在内核层级完成调度。
- **性能优异**：直连速度最快，旁路仅处理特定流量。

**参考文档**：[查看具体实现方式](https://dev.leiyanhui.com/route/ikuai-bypass-joyanhui/) 或 [恩山eezz的教程](https://www.right.com.cn/forum/thread-8252571-1-1.html)。

<details>
<summary>点击这里展开查看详细图文说明</summary>
<img src="https://raw.githubusercontent.com/joyanhui/ikuai-bypass/refs/heads/v4.4.13/assets/img.png" alt="自定义运营商分流模式拓扑图">
</details>

---

### 2. IP 分组与端口分流模式 (传统模式)

**适用场景：** 简单的旁路由方案，逻辑直接。

**实现逻辑：**

1. **IP 分组**：本工具将订阅的 IP 列表同步到 iKuai 的"IP 分组"中。
2. **策略路由**：利用 iKuai 的"端口分流"功能，匹配目标地址为该分组的流量，将其"下一跳网关"指向 OpenWrt 的 IP。

**数据流向：**

```
客户端 → iKuai 路由 → iKuai 物理 WAN1 接口 → 国内目标网站
（国内流量不受影响，直接硬件转发）
```

**特点**：配置简单直接，OpenWrt 宕机时匹配到该分组的规则将无法上网。

**参考文档**：[实现方式参考](https://github.com/joyanhui/ikuai-bypass/issues/7) 或 [恩山y2kji的教程](https://www.right.com.cn/forum/thread-8288009-1-1.html)。
