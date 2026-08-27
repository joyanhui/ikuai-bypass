---
title: 分流模式：运营商(ISP)分流+域名分流 vs IP分组和端口分流
type: docs
weight: 3
---

# 爱快两种分流模式解析

本项目支持两种主流的分流实现方案，您可以根据自己的网络拓扑选择最合适的模式。运行 CLI 时通过 `-m` 参数选择分流模式 [CLI 参数说明](cli-params.md#分流模式--m)。两种模式均不需要修改终端设备的网关地址。

如何选择：如果你旁路由没有多网口，那么选择 IP分组和端口分流的。如果支持多网口（虚拟机 pve也可以）并且追求旁路由宕机后自愈，那么建议使用 运行运营商(ISP)分流+域名分流

---
### 1. IP分组和端口分流

**适用场景：** 简单的旁路由方案，逻辑直接。旁路由可以是单臂路由，也可以是单虚拟网卡虚拟机。

- **IP 分组**：本工具将订阅的 IP 列表同步到 iKuai 的"IP 分组"中，支持ipv4和ipv6。
- **下一跳网关**：利用 iKuai 的"端口分流"功能，匹配目标地址为该分组IP的流量，将其"下一跳网关"指向 旁路由的 IP。把流量导向旁路由

**数据流向：**

```
客户端 → iKuai 路由 → 检查请求的IP地址
      → iKuai 物理 WAN1 接口 → 光猫
      → 端口分流（下一跳指向 旁路由特殊处理) → 返回iKuai → 光猫
```
#### 示意图
下图svg动画为指示的用户请求。`ikuai-bypass`负责处理ip分组的订阅也会创建`下一跳`的规则（下一跳规则记得避开旁路由的ip地址）。

<svg viewBox="80 20 560 410" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" style="width:100%;height:auto;background:#0f172a;border-radius:8px;">
  <defs>
    <marker id="arr" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto">
      <path d="M0,0 L8,4 L0,8 Z" fill="#e2e8f0"/>
    </marker>
    <marker id="arrG" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto">
      <path d="M0,0 L8,4 L0,8 Z" fill="#4ade80"/>
    </marker>
    <marker id="arrR" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto">
      <path d="M0,0 L8,4 L0,8 Z" fill="#f87171"/>
    </marker>
    <marker id="arrB" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto">
      <path d="M0,0 L8,4 L0,8 Z" fill="#60a5fa"/>
    </marker>
    <radialGradient id="dotG" cx="50%" cy="50%" r="50%">
      <stop offset="0%" stop-color="#4ade80"/>
      <stop offset="100%" stop-color="#16a34a"/>
    </radialGradient>
    <radialGradient id="dotR" cx="50%" cy="50%" r="50%">
      <stop offset="0%" stop-color="#fca5a5"/>
      <stop offset="100%" stop-color="#dc2626"/>
    </radialGradient>
    <radialGradient id="dotB" cx="50%" cy="50%" r="50%">
      <stop offset="0%" stop-color="#93c5fd"/>
      <stop offset="100%" stop-color="#2563eb"/>
    </radialGradient>
  </defs>

  <!-- 终端设备 -->
  <rect x="430" y="144" width="190" height="126" rx="10" fill="#14532d" stroke="#4ade80" stroke-width="2"/>

  <!-- 爱快路由 -->
  <rect x="90" y="40" width="310" height="205" rx="10" fill="#1e293b" stroke="#2563eb" stroke-width="2"/>
  <rect x="105" y="105" width="160" height="38" rx="6" fill="#334155" stroke="#a855f7" stroke-width="1.5"/>
  <rect x="195" y="148" width="95" height="38" rx="6" fill="#38bdf8" stroke="#0284c7" stroke-width="1.5"/>
  <rect x="105" y="188" width="80" height="46" rx="6" fill="#334155" stroke="#94a3b8" stroke-width="1.5"/>
  <rect x="193" y="188" width="125" height="44" rx="6" fill="#334155" stroke="#38bdf8" stroke-width="1.5"/>

  <!-- 旁路由 -->
  <rect x="430" y="280" width="190" height="110" rx="10" fill="#0c4a6e" stroke="#38bdf8" stroke-width="2"/>


  <!-- 光猫 -->
  <rect x="90" y="360" width="190" height="60" rx="10" fill="#1e293b" stroke="#a78bfa" stroke-width="2"/>

  <!-- 绿：终端设备 -> LAN口 -> 规则 -->
  <path d="M430 270 L310 270 L310 210 L350 210 L350 126 L265 126" stroke="#4ade80" stroke-width="4" marker-end="url(#arrG)" fill="none"/>
  <path d="M430 270 L368 270" stroke="none" fill="none" marker-end="url(#arrG)"/>
  <path d="M310 210 L340 210" stroke="none" fill="none" marker-end="url(#arrG)"/>
  <path d="M350 210 L350 173" stroke="none" fill="none" marker-end="url(#arrG)"/>
  <path d="M350 126 L300 126" stroke="none" fill="none" marker-end="url(#arrG)"/>
  <path id="pg" d="M430 270 L310 270 L310 210 L350 210 L350 136 L265 136" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotG)">
    <animateMotion dur="2s" repeatCount="indefinite"><mpath xlink:href="#pg"/></animateMotion>
  </circle>
  <circle r="7" fill="url(#dotG)">
    <animateMotion dur="2s" begin="1s" repeatCount="indefinite"><mpath xlink:href="#pg"/></animateMotion>
  </circle>

  <!-- 红：规则 -> 下一跳 -> LAN口左侧 -> 垂出 -> 旁路由 -> LAN口左下 -> Wan1右下 -> 光猫 -->
  <path d="M185 143 L202 160 L213 210 L213 290 L440 290 L440 304 L200 304 L200 226 L120 226 L120 358" stroke="#f87171" stroke-width="2" marker-end="url(#arrR)" fill="none"/>
  <path d="M213 235 L213 272" stroke="none" fill="none" marker-end="url(#arrR)"/>
  <path d="M230 290 L320 290" stroke="none" fill="none" marker-end="url(#arrR)"/>
  <path d="M440 290 L440 300" stroke="none" fill="none" marker-end="url(#arrR)"/>
  <path d="M200 304 L200 255" stroke="none" fill="none" marker-end="url(#arrR)"/>
  <path d="M120 226 L120 287" stroke="none" fill="none" marker-end="url(#arrR)"/>
  <path d="M189 147 L197 155" stroke="none" fill="none" marker-end="url(#arrR)"/>
  <path d="M205 174 L209 192" stroke="none" fill="none" marker-end="url(#arrR)"/>
  <path id="pr" d="M185 143 L202 160 L213 210 L213 290 L440 290 L440 304 L200 304 L200 226 L120 226 L120 356" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotR)">
    <animateMotion dur="2s" begin="2s" repeatCount="indefinite"><mpath xlink:href="#pr"/></animateMotion>
  </circle>
  <circle r="7" fill="url(#dotR)">
    <animateMotion dur="2s" begin="3s" repeatCount="indefinite"><mpath xlink:href="#pr"/></animateMotion>
  </circle>

  <!-- 蓝：规则 -> Wan1 -> 直连出网 -->
  <path d="M185 143 A 90 90 0 0 0 108 205 L108 358" stroke="#60a5fa" stroke-width="2" marker-end="url(#arrB)" fill="none"/>
  <path d="M150 155 L132 167" stroke="none" fill="none" marker-end="url(#arrB)"/>
  <path d="M108 240 L108 292" stroke="none" fill="none" marker-end="url(#arrB)"/>
  <path id="pb" d="M185 143 A 90 90 0 0 0 108 205 L108 356" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotB)">
    <animateMotion dur="2s" begin="2s" repeatCount="indefinite"><mpath xlink:href="#pb"/></animateMotion>
  </circle>
  <circle r="7" fill="url(#dotB)">
    <animateMotion dur="2s" begin="3s" repeatCount="indefinite"><mpath xlink:href="#pb"/></animateMotion>
  </circle>
  <text x="525" y="176" fill="#e2e8f0" font-size="15" text-anchor="middle">终端设备</text>
  <text x="525" y="190" fill="#94a3b8" font-size="10" text-anchor="middle">AP/手机/电脑/平板</text>
  <text x="525" y="212" fill="#7dd3fc" font-size="12" text-anchor="middle">ip 192.168.1.3-254</text>
  <text x="525" y="232" fill="#94a3b8" font-size="11" text-anchor="middle">网关 192.168.1.1</text>
  <text x="525" y="252" fill="#94a3b8" font-size="11" text-anchor="middle">网口连爱快 LAN</text>
  <text x="245" y="70" fill="#fca5a5" font-size="15" text-anchor="middle">爱快主路由 iKuai</text>
  <text x="245" y="90" fill="#94a3b8" font-size="11" text-anchor="middle">LAN ip 192.168.1.1 / WAN1 pppoe</text>
  <text x="185" y="128" fill="#e2e8f0" font-size="12" text-anchor="middle">规则：目标 IP 匹配分组</text>
  <text x="242" y="162" fill="#0f172a" font-size="9" text-anchor="middle">下一跳网关</text>
  <text x="242" y="178" fill="#0f172a" font-size="8" text-anchor="middle">到旁路由 192.168.1.2</text>
  <text x="145" y="208" fill="#e2e8f0" font-size="11" text-anchor="middle">WAN1(PPPOE)</text>
  <text x="255.5" y="208" fill="#e2e8f0" font-size="11" text-anchor="middle">LAN口</text>
  <text x="525" y="310" fill="#4ade80" font-size="15" text-anchor="middle">旁路由 </text>
  <text x="525" y="335" fill="#94a3b8" font-size="11" text-anchor="middle">单网口单臂/多网口都可以、同网段</text>
  <text x="525" y="354" fill="#86efac" font-size="12" text-anchor="middle">ip 192.168.1.2</text>
  <text x="525" y="368" fill="#94a3b8" font-size="10" text-anchor="middle">DHCP获取或者手动配置</text>
  <text x="185" y="396" fill="#c4b5fd" font-size="15" text-anchor="middle">光猫 </text>
</svg>

**参考文档**：[实现方式参考](https://github.com/joyanhui/ikuai-bypass/issues/7) 或 [恩山y2kji的教程](https://www.right.com.cn/forum/thread-8288009-1-1.html)。

### 2. 运营商(ISP)分流+域名分流

**适用场景：** 追求极致稳定性、网络自愈、终端无感分流。（需要多网卡或能添加虚拟网卡的pve等环境）
这种模式下，iKuai 将 旁路由 同时视为"虚拟的上级运营商"和下级终端设备。

- **链路设计**：旁路由作为 iKuai 的上级isp接收流量，处理后再将出口流量"绕回"给 iKuai ,然后ikuai根据来源ip（旁路由的lan地址），然后再发给真实的 WAN 口（光猫）。
- **规则同步**：本工具将目标 IP 列表导入 iKuai 的"自定义运营商"。iKuai 会认为这些 IP 属于该"虚拟运营商"，从而将流量转发给 OpenWrt。

**数据流向：**

```
客户端 → iKuai 路由 → 检查IP/域名
     → 直接走wan1 → 光猫
     → 或者 走wan2  → 旁路由 → 重回iKuai的lan口 → ikuai根据来源 → 请求wan1/光猫
```

**特性：**
- **性能优异**：直连速度最快，旁路仅处理特定流量。
- **缺点** ： 网络拓扑理解起来较为复杂，需要占用ikuai的一个wan一个lan，旁路由自身最好也需要双网口。
**参考文档**：可以参考 [小类的文章](https://dev.leiyanhui.com/route/ikuai-bypass-joyanhui/) 或 [恩山eezz的教程](https://www.right.com.cn/forum/thread-8252571-1-1.html)。 

#### 示意图
下图svg动画为指示的用户请求。`ikuai-bypass`负责处理 `域名分流规则`到wan2 ,以及wan2 的ip订阅；你需要手动添加一条来源ip到wan1的规则，如果有需要可以另外在wan2口上手动添加故障转移（可以在旁路由故障的时候全部走wan1）

<svg viewBox="13 10 607 500" width="607" height="500" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" style="width:100%;height:auto;background:#0f172a;border-radius:8px;">
  <defs>
    <marker id="arrG2" markerWidth="8" markerHeight="8" refX="6" refY="4" orient="auto"><path d="M0,0 L8,4 L0,8 z" fill="#22c55e"/></marker>
    <marker id="arrB2" markerWidth="8" markerHeight="8" refX="6" refY="4" orient="auto"><path d="M0,0 L8,4 L0,8 z" fill="#3b82f6"/></marker>
    <marker id="arrR2" markerWidth="8" markerHeight="8" refX="6" refY="4" orient="auto"><path d="M0,0 L8,4 L0,8 z" fill="#ef4444"/></marker>
  </defs>
  <g transform="translate(-260,0)">

  <!-- 终端设备 -->
  <rect x="665" y="149" width="180" height="102" rx="10" fill="#1e293b" stroke="#22c55e" stroke-width="2"/>

  <!-- 爱快 -->
  <rect x="283" y="20" width="355" height="200" rx="10" fill="#1e293b" stroke="#2563eb" stroke-width="2"/>

  <rect x="297" y="66" width="274" height="36" rx="6" fill="#334155" stroke="#a855f7" stroke-width="1.5"/>

  <rect x="450" y="114" width="92" height="40" rx="6" fill="#334155" stroke="#a855f7" stroke-width="1.5"/>

  <rect x="510" y="162" width="88" height="40" rx="6" fill="#334155" stroke="#94a3b8" stroke-width="1.5"/>

  <rect x="290" y="162" width="96" height="40" rx="6" fill="#334155" stroke="#94a3b8" stroke-width="1.5"/>

  <rect x="400" y="162" width="82" height="40" rx="6" fill="#fbbf24" stroke="#d97706" stroke-width="1.5"/>

  <!-- 旁路由 -->
  <rect x="640" y="260" width="230" height="154" rx="10" fill="#1e293b" stroke="#7dd3fc" stroke-width="2"/>

  <rect x="655" y="268" width="200" height="44" rx="6" fill="#334155" stroke="#7dd3fc" stroke-width="1.5"/>

  <rect x="655" y="320" width="200" height="44" rx="6" fill="#334155" stroke="#7dd3fc" stroke-width="1.5"/>

  <!-- 光猫 -->
  <rect x="280" y="440" width="210" height="60" rx="10" fill="#1e293b" stroke="#a78bfa" stroke-width="2"/>

  <!-- 绿：客户端 -> LAN 口 -> 规则（线、箭头、小球均为绿色） -->
  <path d="M665 251 L588 251 L588 182 L606 182 L606 84 L571 84" stroke="#22c55e" stroke-width="4" fill="none" marker-end="url(#arrG2)"/>
  <path id="pg2" d="M665 251 L588 251 L588 182 L606 182 L606 84 L571 84" stroke="none" fill="none"/>
  <circle r="7" fill="#22c55e">
    <animateMotion dur="2s" repeatCount="indefinite" begin="0s"><mpath xlink:href="#pg2"/></animateMotion>
  </circle>
  <circle r="7" fill="#22c55e">
    <animateMotion dur="2s" repeatCount="indefinite" begin="1s"><mpath xlink:href="#pg2"/></animateMotion>
  </circle>

  <!-- 蓝：规则 -> WAN1(弧线到内部左侧) -> 垂直到光猫（参考图1：末端大箭头停在光猫上边缘） -->
  <path d="M395 102 A 90 90 0 0 0 293 182 L293 438" stroke="#3b82f6" stroke-width="2" fill="none" marker-end="url(#arrB2)"/>
  <path id="pb2" d="M395 102 A 90 90 0 0 0 293 182 L293 438" stroke="none" fill="none"/>
  <circle r="7" fill="#3b82f6">
    <animateMotion dur="2s" repeatCount="indefinite" begin="2s"><mpath xlink:href="#pb2"/></animateMotion>
  </circle>
  <circle r="7" fill="#3b82f6">
    <animateMotion dur="2s" repeatCount="indefinite" begin="3s"><mpath xlink:href="#pb2"/></animateMotion>
  </circle>

  <!-- 红：规则 -> WAN2 -> 旁路由LAN -> 旁路由WAN -> 主路由LAN口 -> 来源IP -> WAN1 -> 光猫（独立红线到光猫，规则到 WAN2 走右侧空隙、回程经 LAN口->来源IP->WAN1） -->
  <path d="M395 102 L415 182 L415 360 L665 360 L665 308 L520 308 L520 182 L458 134 A 200 200 0 0 0 408 135.1 Q403.5 120.7 399 135.5 A 200 200 0 0 0 303 182 L303 438" stroke="#ef4444" stroke-width="2" fill="none" marker-end="url(#arrR2)"/>
  <path id="pr2" d="M395 102 L415 182 L415 360 L665 360 L665 308 L520 308 L520 182 L458 134 A 200 200 0 0 0 408 135.1 Q403.5 120.7 399 135.5 A 200 200 0 0 0 303 182 L303 438" stroke="none" fill="none"/>
  <circle r="7" fill="#ef4444">
    <animateMotion dur="2s" repeatCount="indefinite" begin="2s"><mpath xlink:href="#pr2"/></animateMotion>
  </circle>
  <circle r="7" fill="#ef4444">
    <animateMotion dur="2s" repeatCount="indefinite" begin="3s"><mpath xlink:href="#pr2"/></animateMotion>
  </circle>

  <!-- 方向箭头：仅放在直线段中点，朝运动方向；拐弯处不加箭头 -->
  <path d="M616.5 251 L606.5 251" stroke="#22c55e" stroke-width="1" marker-end="url(#arrG2)"/>
  <path d="M588 221.5 L588 211.5" stroke="#22c55e" stroke-width="1" marker-end="url(#arrG2)"/>
  <path d="M592 182 L602 182" stroke="#22c55e" stroke-width="1" marker-end="url(#arrG2)"/>
  <path d="M606 138 L606 128" stroke="#22c55e" stroke-width="1" marker-end="url(#arrG2)"/>
  <path d="M588.5 84 L578.5 84" stroke="#22c55e" stroke-width="1" marker-end="url(#arrG2)"/>
  <path d="M330.6 117.4 L322.8 123.6" stroke="#3b82f6" stroke-width="1" marker-end="url(#arrB2)"/>
  <path d="M293 305 L293 315" stroke="#3b82f6" stroke-width="1" marker-end="url(#arrB2)"/>
  <path d="M398.8 117.2 L401.2 126.9" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M415 266 L415 276" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M535 360 L545 360" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M665 339 L665 329" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M597.5 308 L587.5 308" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M520 250 L520 240" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M493.0 161.1 L485.0 154.9" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M450 133.5 L440 133" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M303 305 L303 315" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <text x="755" y="182" fill="#e2e8f0" font-size="14" text-anchor="middle">终端设备</text>
  <text x="755" y="192" fill="#94a3b8" font-size="10" text-anchor="middle">AP/手机/电脑/平板</text>
  <text x="755" y="214" fill="#4ade80" font-size="12" text-anchor="middle">ip 192.168.1.3-254</text>
  <text x="755" y="234" fill="#94a3b8" font-size="11" text-anchor="middle">网关 192.168.1.1（爱快 LAN）</text>
  <text x="445" y="38" fill="#fca5a5" font-size="14" text-anchor="middle">爱快主路由 iKuai</text>
  <text x="445" y="54" fill="#94a3b8" font-size="10" text-anchor="middle">LAN ip 192.168.1.1 / WAN1 pppoe /WAN2接旁路由的lan</text>
  <text x="434" y="90" fill="#e2e8f0" font-size="9" text-anchor="middle">规则：IP匹配自定义虚拟运营商或者域名匹配</text>
  <text x="496" y="139" fill="#e2e8f0" font-size="10" text-anchor="middle">规则：来源 IP</text>
  <text x="496" y="150" fill="#94a3b8" font-size="8" text-anchor="middle">（来自192.168.1.2）</text>
  <text x="554" y="178" fill="#e2e8f0" font-size="10" text-anchor="middle">LAN 口</text>
  <text x="338" y="182" fill="#e2e8f0" font-size="11" text-anchor="middle">WAN1(PPPOE)</text>
  <text x="441" y="182" fill="#000" font-size="9" text-anchor="middle">WAN2(虚拟运营商)</text>
  <text x="441" y="198" fill="#000" font-size="10" text-anchor="middle">ip 10.0.0.2</text>
  <text x="755" y="386" fill="#7dd3fc" font-size="14" text-anchor="middle">旁路由</text>
  <text x="755" y="402" fill="#94a3b8" font-size="11" text-anchor="middle">双网口、跨网段</text>
  <text x="755" y="286" fill="#7dd3fc" font-size="11" text-anchor="middle">WAN 口 ip 192.168.1.2</text>
  <text x="755" y="303" fill="#94a3b8" font-size="10" text-anchor="middle">网关 192.168.1.1（接爱快 LAN）</text>
  <text x="755" y="338" fill="#7dd3fc" font-size="11" text-anchor="middle">LAN 口 ip 10.0.0.1</text>
  <text x="755" y="355" fill="#94a3b8" font-size="10" text-anchor="middle">（接爱快 WAN2，最好关闭DHCP服务）</text>
  <text x="385" y="476" fill="#c4b5fd" font-size="14" text-anchor="middle">光猫</text>
  </g>
</svg>

<details>
<summary>⭐ 点击这里展开查看详细图文说明(运营商(ISP)分流+域名分流模式拓扑图)</summary>
<img src="https://raw.githubusercontent.com/joyanhui/ikuai-bypass/refs/heads/v4.4.13/assets/img.png" alt="运营商(ISP)分流+域名分流模式拓扑图">
</details>

---
