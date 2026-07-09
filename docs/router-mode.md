---
title: 分流模式：运营商 vs IP 分组
parent: 🚀 快速上手与入门指南
nav_order: 1
mermaid: true
---

# 爱快两种分流模式解析

本项目支持两种主流的分流实现方案，您可以根据自己的网络拓扑选择最合适的模式。运行 CLI 时通过 `-m` 参数选择分流模式，详见 [CLI 参数说明](cli-params.md#分流模式--m)。


---
### 1. IP 分组与端口分流模式 

**适用场景：** 简单的旁路由方案，逻辑直接。

**实现逻辑：**

1. **IP 分组**：本工具将订阅的 IP 列表同步到 iKuai 的"IP 分组"中。
2. **策略路由**：利用 iKuai 的"端口分流"功能，匹配目标地址为该分组IP的流量，将其"下一跳网关"指向 旁路由（通常是OpenWRT） 的 IP。把流量导向旁路由

**数据流向：**

```
客户端 → iKuai 路由 → 检查请求的IP地址 → iKuai 物理 WAN1 接口 → 运营商光猫
                                   → 端口分流（下一跳指向 旁路由特殊处理) → 返回iKuai  → 运营商光猫

```

**特点**：配置简单直接，OpenWrt 宕机时匹配到该分组的规则将无法上网。

```mermaid
graph TD
    %% 外部环境
    光猫设备[光猫设备 / WAN出口]
    
    %% 局域网客户端
    subgraph 局域网客户端
        Client1["客户端<br/>例如: 手机/电脑"]
        ClientIP[ip: 192.168.1.2-254]
    end
    
    %% 爱快主路由系统（独立子图）
    subgraph iKuai_Sys[iKuai 爱快主路由]
        Check{分流规则}
        iKuaiIP[ip192.168.1.1]
    end

    %% 旁路由系统（独立子图）
    subgraph Proxy_Sys[旁路由 OpenWrt]
        Proxy_Core["核心工具<br/>解密/加密/代理加速"]
        OpenWrtIP[ip192.168.1.2]
    end

    %% --- 流量走向连线 ---
    
    %% 1. 客户端发起请求
    Client1 --> |所有流量默认网关指向爱快| Check
    
    %% 2. 爱快内部规则判断
    Check --> |"匹配IP分组(需加速/加密/代理)"| NextHop[下一跳网关]
    NextHop --> |网关给旁路由IP192.168.1.2| Proxy_Core
    
    Check --> |"匹配IP分组(直连)"| 光猫设备
    Check --> |"来自旁路由192.168.1.2"| 光猫设备

    %% 3. 旁路由处理完毕后回传
    Proxy_Core ==> |"加密后的代理流量<br/>(源IP变为旁路由自己)"| Check

    %% --- 样式与颜色优化 ---
    
    linkStyle 4 stroke:#10b981,stroke-width:4px;
    linkStyle 5 stroke:#10b981,stroke-width:4px;
    
    style iKuai_Sys fill:#fff5f5,stroke:#f87171,stroke-width:2px
    style Proxy_Sys fill:#f0fdf4,stroke:#4ade80,stroke-width:2px

    
    style Check fill:#fbbf24,stroke:#d97706,stroke-width:2px
    style NextHop fill:#38bdf8,stroke:#0284c7,stroke-width:1px
    style 光猫设备 fill:#a78bfa,stroke:#7c3aed,stroke-width:2px,color:#fff
    
    style ClientIP fill:#38bdf8,stroke:#f87171,stroke-width:2px
    style iKuaiIP fill:#38bdf8,stroke:#f87171,stroke-width:2px
    style OpenWrtIP fill:#38bdf8,stroke:#f87171,stroke-width:2px
```

**参考文档**：[实现方式参考](https://github.com/joyanhui/ikuai-bypass/issues/7) 或 [恩山y2kji的教程](https://www.right.com.cn/forum/thread-8288009-1-1.html)。

### 2. 自定义运营商分流模式 

**适用场景：** 追求极致稳定性、网络自愈、终端无感分流。（需要多网卡或能添加虚拟网卡）
.
**实现逻辑：**
这种模式下，iKuai 将 OpenWrt（旁路由）视为一个"虚拟的上级运营商"。

1. **链路设计**：OpenWrt 作为 iKuai 的下级设备接收流量，处理后再将出口流量"绕回"给 iKuai 的 WAN 口。
2. **规则同步**：本工具将目标 IP 列表导入 iKuai 的"自定义运营商"。iKuai 会认为这些 IP 属于该"虚拟运营商"，从而将流量转发给 OpenWrt。

**数据流向：**

```
客户端 → iKuai 路由 → 检查IP/域名  

     → 直接走wan1/运营商光猫
     → 走wan2  → 旁路由 插件处理 → 重新交回 iKuai 的lan口 → ikuai根据来源 请求wan1/运营商光猫
```

**核心优势：**

- **极高可靠性**：旁路由 宕机只会导致被分流的流量中断，普通流量依然通过主线直连，不会全家断网。
- **配置无感**：终端设备无需更改网关配置，完全由 iKuai 在内核层级完成调度。
- **性能优异**：直连速度最快，旁路仅处理特定流量。

**参考文档**：[查看具体实现方式](https://dev.leiyanhui.com/route/ikuai-bypass-joyanhui/) 或 [恩山eezz的教程](https://www.right.com.cn/forum/thread-8252571-1-1.html)。


```mermaid
graph TD
    %% 外部设备与局域网客户端
    光猫设备[光猫设备]

    %% 局域网客户端
    subgraph 局域网客户端
        局域网其他设备1[局域网客户端 1]
        局域网其他设备2[局域网客户端 2]
        局域网其他设备3[其他客户端 ...]
    end
    
    %% 爱快系统容器
    subgraph iKuai[爱快主路由系统]
        爱快["爱快<br/>IP: 192.168.1.1"]
        爱快lan[爱快 LAN 口]
        Check{规则判断}
        爱快wan1["爱快 WAN1<br/>IP等由ppoe或者其他方式分配"]
        爱快wan2["爱快 WAN2<br/>IP:10.0.0.2<br/>网关：10.0.0.1「旁路由」"]
        
        爱快 --> |IP/域名列表分流规则检查或者判断是否来自旁路由| Check 
        Check --> |直连| 爱快wan1 
        Check --> |需要特殊处理的「加速/加密」的请求| 爱快wan2 
        %% line 3
        爱快lan ==> 爱快
    end

    %% 旁路由通常为OpenWrt
    subgraph OpenWrt[旁路由通常为OpenWrt]
        旁路由[旁路由核心处理程序]
        旁路由lan["旁路由 LAN 口<br/>IP: 10.0.0.1"]
        旁路由wan["旁路由 WAN 口<br/>IP: 192.168.1.2<br/>网关：192.168.1.1「爱快」"]
        
        旁路由lan --> 旁路由
        旁路由 --> |特殊处理加密/加速代理| 旁路由wan
    end

    %% 系统间的数据流向连接
    局域网其他设备1 --> 爱快lan
    局域网其他设备2 --> 爱快lan
    局域网其他设备3 --> 爱快lan
    
    %% line 9
    Check ==>|来源ip判断，让旁路由「192.168.1.2」过来的流量直接去wan1| 爱快wan1
    爱快wan1 -->|运营商网络| 光猫设备
    爱快wan2 --> 旁路由lan
            %% line 12
    旁路由wan ==> |"加密后的代理流量<br/>(源IP变为旁路由自己)"|爱快lan

    %% 样式与颜色优化
    style iKuai fill:#fdf2f2,stroke:#f8b4b4,stroke-width:2px
    style OpenWrt fill:#f0f5ff,stroke:#adc6ff,stroke-width:2px
    
    style 爱快 fill:#f87171,stroke:#b91c1c,stroke-width:1px,color:#fff
    style Check fill:#fbbf24,stroke:#d97706,stroke-width:2px
    
    style 旁路由 fill:#60a5fa,stroke:#1d4ed8,stroke-width:1px,color:#fff
    style 光猫设备 fill:#34d399,stroke:#047857,stroke-width:1px,color:#fff
    
    style 局域网其他设备1 fill:#e5e7eb,stroke:#9ca3af,stroke-width:1px
    style 局域网其他设备2 fill:#e5e7eb,stroke:#9ca3af,stroke-width:1px
    style 局域网其他设备3 fill:#e5e7eb,stroke:#9ca3af,stroke-width:1px
    
    linkStyle 9 stroke:#10b981,stroke-width:4px;
    linkStyle 12 stroke:#10b981,stroke-width:4px;
```
<details>
<summary>点击这里展开查看详细图文说明(自定义运营商分流模式拓扑图)</summary>
<img src="https://raw.githubusercontent.com/joyanhui/ikuai-bypass/refs/heads/v4.4.13/assets/img.png" alt="自定义运营商分流模式拓扑图">
</details>

---

