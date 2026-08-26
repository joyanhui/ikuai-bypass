---
title: 分流模式 SVG 动画对比
type: docs
weight: 11
---
## ip分组和端口分流
<svg viewBox="0 0 980 440" width="980" height="440" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" style="width:100%;height:auto;background:#0f172a;border-radius:8px;">
  <defs>
    <marker id="arr" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto">
      <path d="M0,0 L8,4 L0,8 Z" fill="#e2e8f0"/>
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

  <!-- 绿：客户端 -> 规则 -->
  <path d="M200 205 L443 124" stroke="#4ade80" stroke-width="2" marker-end="url(#arr)" fill="none"/>
  <path id="pg" d="M200 205 L441 124" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotG)">
    <animateMotion dur="2s" repeatCount="indefinite"><mpath xlink:href="#pg"/></animateMotion>
  </circle>

  <!-- 红：规则 -> 下一跳 -> 旁路由 -> 返回 -> 出网 -->
  <path d="M525 143 L569 172 L768 180 L525 235 L525 348" stroke="#f87171" stroke-width="2" marker-end="url(#arr)" fill="none"/>
  <path id="pr" d="M525 143 L569 172 L768 180 L525 235 L525 346" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotR)">
    <animateMotion dur="6s" begin="2s" repeatCount="indefinite"><mpath xlink:href="#pr"/></animateMotion>
  </circle>

  <!-- 蓝：规则 -> WAN1 -> 直连出网 -->
  <path d="M525 143 L483 172 L483 348" stroke="#60a5fa" stroke-width="2" marker-end="url(#arr)" fill="none"/>
  <path id="pb" d="M525 143 L483 172 L483 346" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotB)">
    <animateMotion dur="4s" begin="2s" repeatCount="indefinite"><mpath xlink:href="#pb"/></animateMotion>
  </circle>
</svg>

### isp+ 域名分流

<svg viewBox="0 0 980 500" width="980" height="500" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" style="width:100%;height:auto;background:#0f172a;border-radius:8px;">
  <defs>
    <marker id="arrB" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto">
      <path d="M0,0 L8,4 L0,8 Z" fill="#e2e8f0"/>
    </marker>
    <radialGradient id="dotB2" cx="50%" cy="50%" r="50%">
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

  <!-- 路径 1：客户端 -> 爱快 -->
  <path d="M200 75 L290 75" stroke="#e2e8f0" stroke-width="2" marker-end="url(#arrB)" fill="none"/>
  <path id="pb1" d="M202 75 L288 75" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotB2)">
    <animateMotion dur="3s" repeatCount="indefinite"><mpath xlink:href="#pb1"/></animateMotion>
  </circle>

  <!-- 路径 2：爱快 -> WAN2 -> 旁路由 LAN -->
  <path d="M485 150 L703 177" stroke="#fbbf24" stroke-width="2" marker-end="url(#arrB)" fill="none"/>
  <path id="pb2" d="M485 150 L703 177" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotB2)">
    <animateMotion dur="3s" begin="3s" repeatCount="indefinite"><mpath xlink:href="#pb2"/></animateMotion>
  </circle>

  <!-- 路径 3：旁路由 WAN -> 回送爱快 LAN -->
  <path d="M705 233 L395 233 L395 218" stroke="#fbbf24" stroke-width="2" marker-end="url(#arrB)" fill="none"/>
  <path id="pb3" d="M703 233 L395 233 L395 216" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotB2)">
    <animateMotion dur="3s" begin="6s" repeatCount="indefinite"><mpath xlink:href="#pb3"/></animateMotion>
  </circle>

  <!-- 路径 4：爱快 -> WAN1 -> 光猫 -->
  <path d="M395 222 L395 408" stroke="#fde68a" stroke-width="2" marker-end="url(#arrB)" fill="none"/>
  <path id="pb4" d="M395 222 L395 406" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotB2)">
    <animateMotion dur="3s" begin="9s" repeatCount="indefinite"><mpath xlink:href="#pb4"/></animateMotion>
  </circle>

  <!-- 直连（未命中名单）静态示意 -->
  <path d="M348 170 L348 408" stroke="#94a3b8" stroke-width="1.5" stroke-dasharray="4 4" marker-end="url(#arrB)" fill="none" opacity="0.7"/>
</svg>
