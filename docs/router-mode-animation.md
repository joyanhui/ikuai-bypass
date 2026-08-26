---
title: 分流模式 SVG 动画对比
type: docs
weight: 11
---

# 分流模式 SVG 动画对比

本页用动画直观展示本项目两种分流模式的流量走向差异，并标注每个节点的 IP 配置与物理连接。文字详解见 [分流模式解析](router-mode.md)。

两种模式对应的分流参数见 [CLI 参数说明](cli-params.md#分流模式--m)：

- `ispdomain`（默认）— 自定义运营商 + 域名分流
- `ipgroup` — IPv4 分组 + 端口分流

## 模式一：ipgroup（IP 分组 + 端口分流）

旁路由没有多网口时的简单方案。工具把订阅 IP 同步进 iKuai 的 "IP 分组"，再用 "端口分流" 把目标命中分组的流量，下一跳网关指向旁路由。物理上所有设备都挂在同一个局域网（192.168.1.0/24），旁路由只有一个网口接入。

<svg viewBox="0 0 980 440" xmlns="http://www.w3.org/2000/svg" style="width:100%;height:auto;background:#0f172a;border-radius:8px;">
  <defs>
    <marker id="arr" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto">
      <path d="M0,0 L8,4 L0,8 Z" fill="#e2e8f0"/>
    </marker>
    <radialGradient id="dotA" cx="50%" cy="50%" r="50%">
      <stop offset="0%" stop-color="#4ade80"/>
      <stop offset="100%" stop-color="#16a34a"/>
    </radialGradient>
  </defs>

  <!-- 客户端 -->
  <rect x="20" y="150" width="180" height="110" rx="10" fill="#1e293b" stroke="#38bdf8" stroke-width="2"/>
  <text x="110" y="180" fill="#e2e8f0" font-size="15" text-anchor="middle">客户端</text>
  <text x="110" y="205" fill="#7dd3fc" font-size="12" text-anchor="middle">ip 192.168.1.100</text>
  <text x="110" y="224" fill="#94a3b8" font-size="11" text-anchor="middle">网关 192.168.1.1</text>
  <text x="110" y="242" fill="#94a3b8" font-size="11" text-anchor="middle">网口连爱快 LAN</text>

  <!-- 爱快路由 -->
  <rect x="430" y="40" width="190" height="180" rx="10" fill="#1e293b" stroke="#f87171" stroke-width="2"/>
  <text x="525" y="70" fill="#fca5a5" font-size="15" text-anchor="middle">爱快主路由 iKuai</text>
  <text x="525" y="90" fill="#94a3b8" font-size="11" text-anchor="middle">LAN ip 192.168.1.1 / WAN1 pppoe</text>
  <rect x="445" y="105" width="160" height="38" rx="6" fill="#334155" stroke="#f87171" stroke-width="1.5"/>
  <text x="525" y="128" fill="#e2e8f0" font-size="12" text-anchor="middle">规则：目标 IP 匹配分组</text>
  <rect x="445" y="155" width="76" height="34" rx="6" fill="#334155" stroke="#94a3b8" stroke-width="1.5"/>
  <text x="483" y="176" fill="#e2e8f0" font-size="11" text-anchor="middle">WAN1 真实</text>
  <rect x="533" y="155" width="72" height="34" rx="6" fill="#38bdf8" stroke="#0284c7" stroke-width="1.5"/>
  <text x="569" y="176" fill="#0f172a" font-size="11" text-anchor="middle">下一跳</text>

  <!-- 旁路由 -->
  <rect x="770" y="150" width="190" height="110" rx="10" fill="#1e293b" stroke="#4ade80" stroke-width="2"/>
  <text x="865" y="180" fill="#4ade80" font-size="15" text-anchor="middle">旁路由 OpenWrt</text>
  <text x="865" y="205" fill="#86efac" font-size="12" text-anchor="middle">ip 192.168.1.2</text>
  <text x="865" y="224" fill="#94a3b8" font-size="11" text-anchor="middle">单网口、同网段</text>
  <text x="865" y="242" fill="#94a3b8" font-size="11" text-anchor="middle">解密/加速/代理</text>

  <!-- 光猫 -->
  <rect x="430" y="350" width="190" height="60" rx="10" fill="#1e293b" stroke="#a78bfa" stroke-width="2"/>
  <text x="525" y="386" fill="#c4b5fd" font-size="15" text-anchor="middle">运营商光猫 / WAN</text>

  <!-- 路径 -->
  <!-- 客户端 -> 路由 -->
  <path d="M200 205 L430 205" stroke="#e2e8f0" stroke-width="2" marker-end="url(#arr)" fill="none"/>
  <path id="pa1" d="M202 205 L428 205" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotA)">
    <animateMotion dur="3s" repeatCount="indefinite"><mpath href="#pa1"/></animateMotion>
  </circle>

  <!-- 路由判断 命中去旁路由 -->
  <path d="M525 143 L525 172 L569 172 L865 172 L865 150" stroke="#38bdf8" stroke-width="2" marker-end="url(#arr)" fill="none"/>
  <path id="pa2" d="M525 143 L525 172 L865 172 L865 148" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotA)">
    <animateMotion dur="3s" begin="3s" repeatCount="indefinite"><mpath href="#pa2"/></animateMotion>
  </circle>

  <!-- 旁路由处理完 源IP变 返回爱快 -->
  <path d="M770 240 L525 240 L525 350" stroke="#4ade80" stroke-width="2" marker-end="url(#arr)" fill="none"/>
  <path id="pa3" d="M768 240 L525 240 L525 348" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotA)">
    <animateMotion dur="3s" begin="6s" repeatCount="indefinite"><mpath href="#pa3"/></animateMotion>
  </circle>

  <!-- 直连走 wan1 出光猫 -->
  <path d="M430 380 L525 380" stroke="#cbd5e1" stroke-width="2" marker-end="url(#arr)" fill="none" opacity="0.6"/>

  <!-- 标注 -->
  <text x="525" y="335" fill="#86efac" font-size="12" text-anchor="middle">回程源 IP 已变 192.168.1.2 → 放行 → 真实 WAN</text>
  <text x="20" y="425" fill="#94a3b8" font-size="12">数据包流向：客户端 → iKuai 目标IP匹配分组 → 命中⇢下一跳指向旁路由 → 处理完毕原路返回 → 走真实 WAN 出网</text>
</svg>

物理连接与 IP：

- 所有设备（客户端 / 爱快 / 旁路由）处于同一局域网 192.168.1.0/24，旁路由单网口接入爱快 LAN。
- 客户端 ip 192.168.1.100，网关 192.168.1.1（爱快 LAN）。
- 爱快 LAN ip 192.168.1.1；WAN1 由 pppoe 从运营商分配，出真实光猫。
- 旁路由 ip 192.168.1.2，与爱快同网段，仅一个逻辑网口。
- 分流由爱快 "端口分流" 规则实现，把命中分组流量下一跳指向 192.168.1.2。

数据流向：

- 客户端所有流量默认网关指向爱快。
- 爱快按目标 IP 匹配 "IP 分组"，命中的流量通过 "端口分流" 下一跳网关指向旁路由。
- 旁路由处理（解密 / 加速 / 代理）后，源 IP 变为 192.168.1.2，经同一网段原路返回爱快。
- 爱快按来源 IP 放行，将其交给真实 WAN 出网。
- 特点：逻辑简单直接、灵活；旁路由无需多网口。
- 局限：终端需确认流量均默认指向爱快；无多 WAN 自动切换自愈能力。

## 模式二：ispdomain（自定义运营商分流）

追求稳定 / 自愈 / 无感分流。旁路由被 iKuai 同时视为"虚拟上级运营商"和"下级终端"：旁路由的 LAN 口接爱快的 WAN2（旁路由作为上级，供爱快把流量转给它），旁路由的 WAN 口接爱快的 LAN（旁路由作为下级终端出网）。需要爱快额外占用一个 WAN 和一个 LAN，旁路由最好双网口。

<svg viewBox="0 0 980 500" xmlns="http://www.w3.org/2000/svg" style="width:100%;height:auto;background:#0f172a;border-radius:8px;">
  <defs>
    <marker id="arrB" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto">
      <path d="M0,0 L8,4 L0,8 Z" fill="#e2e8f0"/>
    </marker>
    <radialGradient id="dotB" cx="50%" cy="50%" r="50%">
      <stop offset="0%" stop-color="#fbbf24"/>
      <stop offset="100%" stop-color="#d97706"/>
    </radialGradient>
  </defs>

  <!-- 客户端 -->
  <rect x="20" y="30" width="180" height="90" rx="10" fill="#1e293b" stroke="#38bdf8" stroke-width="2"/>
  <text x="110" y="62" fill="#e2e8f0" font-size="14" text-anchor="middle">客户端</text>
  <text x="110" y="85" fill="#7dd3fc" font-size="12" text-anchor="middle">ip 192.168.1.100</text>
  <text x="110" y="104" fill="#94a3b8" font-size="11" text-anchor="middle">网关 192.168.1.1（爱快 LAN）</text>

  <!-- 爱快 -->
  <rect x="290" y="20" width="210" height="200" rx="10" fill="#1e293b" stroke="#f87171" stroke-width="2"/>
  <text x="395" y="52" fill="#fca5a5" font-size="14" text-anchor="middle">爱快主路由 iKuai</text>

  <rect x="305" y="66" width="180" height="38" rx="6" fill="#334155" stroke="#f87171" stroke-width="1.5"/>
  <text x="395" y="90" fill="#e2e8f0" font-size="12" text-anchor="middle">规则：自定义运营商名单</text>
  <text x="395" y="116" fill="#94a3b8" font-size="11" text-anchor="middle">LAN ip 192.168.1.1</text>

  <rect x="305" y="130" width="86" height="40" rx="6" fill="#334155" stroke="#94a3b8" stroke-width="1.5"/>
  <text x="348" y="146" fill="#e2e8f0" font-size="11" text-anchor="middle">WAN1 真实</text>
  <text x="348" y="162" fill="#94a3b8" font-size="10" text-anchor="middle">pppoe 出光猫</text>

  <rect x="403" y="130" width="82" height="40" rx="6" fill="#fbbf24" stroke="#d97706" stroke-width="1.5"/>
  <text x="444" y="146" fill="#000" font-size="11" text-anchor="middle">WAN2</text>
  <text x="444" y="162" fill="#000" font-size="10" text-anchor="middle">ip 10.0.0.2</text>

  <!-- 旁路由 -->
  <rect x="690" y="110" width="230" height="180" rx="10" fill="#1e293b" stroke="#4ade80" stroke-width="2"/>
  <text x="805" y="140" fill="#4ade80" font-size="14" text-anchor="middle">旁路由 OpenWrt（虚拟运营商）</text>

  <rect x="705" y="155" width="200" height="44" rx="6" fill="#334155" stroke="#4ade80" stroke-width="1.5"/>
  <text x="805" y="173" fill="#7dd3fc" font-size="11" text-anchor="middle">LAN 口 ip 10.0.0.1（接爱快 WAN2）</text>
  <text x="805" y="190" fill="#94a3b8" font-size="10" text-anchor="middle">作为"上级运营商"面板</text>

  <rect x="705" y="211" width="200" height="44" rx="6" fill="#334155" stroke="#86efac" stroke-width="1.5"/>
  <text x="805" y="229" fill="#7dd3fc" font-size="11" text-anchor="middle">WAN 口 ip 192.168.1.2</text>
  <text x="805" y="246" fill="#94a3b8" font-size="10" text-anchor="middle">网关 192.168.1.1（接爱快 LAN, 出网）</text>

  <text x="805" y="278" fill="#94a3b8" font-size="11" text-anchor="middle">解密 / 加速 / 代理</text>

  <!-- 光猫 -->
  <rect x="290" y="410" width="210" height="60" rx="10" fill="#1e293b" stroke="#a78bfa" stroke-width="2"/>
  <text x="395" y="446" fill="#c4b5fd" font-size="14" text-anchor="middle">运营商光猫 / 真实 WAN</text>

  <!-- 路径 -->
  <!-- 客户端 -> 爱快 -->
  <path d="M200 75 L290 75" stroke="#e2e8f0" stroke-width="2" marker-end="url(#arrB)" fill="none"/>
  <path id="pb1" d="M202 75 L288 75" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotB)">
    <animateMotion dur="3s" repeatCount="indefinite"><mpath href="#pb1"/></animateMotion>
  </circle>

  <!-- 爱快判断 -> wan2 -> 旁路由LAN -->
  <path d="M395 104 L444 104 L444 130 L805 130 L805 155" stroke="#fbbf24" stroke-width="2" marker-end="url(#arrB)" fill="none"/>
  <path id="pb2" d="M395 104 L444 104 L444 130 L805 130 L805 153" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotB)">
    <animateMotion dur="3s" begin="3s" repeatCount="indefinite"><mpath href="#pb2"/></animateMotion>
  </circle>

  <!-- 旁路由处理完 从WAN口 回送爱快LAN -->
  <path d="M690 233 L520 233 L520 360 L395 360 L395 410" stroke="#fbbf24" stroke-width="2" marker-end="url(#arrB)" fill="none"/>
  <path id="pb3" d="M688 233 L520 233 L520 360 L395 360 L395 408" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotB)">
    <animateMotion dur="3s" begin="6s" repeatCount="indefinite"><mpath href="#pb3"/></animateMotion>
  </circle>

  <!-- 标注 -->
  <text x="395" y="398" fill="#fde68a" font-size="12" text-anchor="middle">旁路由 WAN 回送爱快 LAN，来源 IP 192.168.1.2 识别放行</text>
  <text x="20" y="488" fill="#94a3b8" font-size="12">数据包流向：客户端 → iKuai 规则判断 → 命中自定义运营商名单 → WAN2 转给旁路由 LAN → 处理完毕从旁路由 WAN 回送爱快 → 走真实 WAN 出网</text>
</svg>

物理连接与 IP：

- 旁路由双网口，构成"回环"拓扑：旁路由 LAN 口接爱快 WAN2，旁路由 WAN 口接爱快 LAN。
- 爱快侧：LAN ip 192.168.1.1（接客户端与旁路由 WAN）；WAN1 由 pppoe 从运营商分配，出真实光猫；WAN2 ip 10.0.0.2、网关 10.0.0.1（即旁路由 LAN）。
- 旁路由侧：LAN 口 ip 10.0.0.1（作为爱快 WAN2 的"虚拟上级运营商"）；WAN 口 ip 192.168.1.2、网关 192.168.1.1（作为下级终端经爱快出网）。
- 客户端 ip 192.168.1.100，网关 192.168.1.1，无需修改。

数据流向：

- 客户端无需改网关，默认流量全部交给爱快。
- 工具把目标 IP 导入爱快 "自定义运营商"，iKuai 视这些 IP 属于旁路由这个"虚拟运营商"。
- 命中名单的流量经 WAN2（10.0.0.2 → 10.0.0.1）转给旁路由处理。
- 旁路由处理（解密 / 加速 / 代理）后，从自身 WAN 口（192.168.1.2 → 192.168.1.1）回送爱快 LAN；iKuai 依据来源 IP 识别后放行到真实 WAN 出网。
- 特点：终端无感、直连流量最快、旁路只处理特定流量；可利用爱快多 WAN 自动切换实现网络自愈。
- 局限：拓扑较复杂，需占用爱快一个 WAN 和一个 LAN，旁路由最好双网口。

## 差异对比

| 维度 | ipgroup（IP 分组 + 端口分流） | ispdomain（自定义运营商） |
| :--- | :--- | :--- |
| 底层原理 | IP 分组 + 端口分流（下一跳网关） | 自定义运营商（旁路由虚拟成运营商） |
| 物理链接 | 单局域网，旁路由单网口 | 旁路由双网口回环（LAN↔WAN2、WAN↔LAN） |
| 旁路由网口 | 1 个即可 | 最好 2 个 |
| 爱快占用 | 仅逻辑规则 | 额外占一个 WAN + 一个 LAN |
| 客户端 IP | 192.168.1.x，需默认指向爱快 | 192.168.1.x，无需改网关 |
| 网络自愈 | 无多 WAN 自动切换 | 可用多 WAN 自动切换实现自愈 |
| 性能 | 正常 | 直连最快、旁路只处理特定流量 |
| 复杂度 | 简单直接 | 拓扑较复杂 |
| 适用场景 | 简单旁路由、无多网口 | 追求稳定自愈、无感分流 |

> 提示：cli-params 中 `ipv6group / ii / ip / iip` 等为上述两种基础方案的组合扩展。默认推荐 `ispdomain`。
