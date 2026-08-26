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
  <rect x="430" y="160" width="190" height="110" rx="10" fill="#1e293b" stroke="#38bdf8" stroke-width="2"/>
  <text x="525" y="192" fill="#e2e8f0" font-size="15" text-anchor="middle">终端设备</text>
  <text x="525" y="215" fill="#7dd3fc" font-size="12" text-anchor="middle">ip 192.168.1.100</text>
  <text x="525" y="234" fill="#94a3b8" font-size="11" text-anchor="middle">网关 192.168.1.1</text>
  <text x="525" y="252" fill="#94a3b8" font-size="11" text-anchor="middle">网口连爱快 LAN</text>

  <!-- 爱快路由 -->
  <rect x="90" y="40" width="310" height="205" rx="10" fill="#1e293b" stroke="#2563eb" stroke-width="2"/>
  <text x="245" y="70" fill="#fca5a5" font-size="15" text-anchor="middle">爱快主路由 iKuai</text>
  <text x="245" y="90" fill="#94a3b8" font-size="11" text-anchor="middle">LAN ip 192.168.1.1 / WAN1 pppoe</text>
  <rect x="105" y="105" width="160" height="38" rx="6" fill="#334155" stroke="#f87171" stroke-width="1.5"/>
  <text x="185" y="128" fill="#e2e8f0" font-size="12" text-anchor="middle">规则：目标 IP 匹配分组</text>
  <rect x="170" y="150" width="66" height="22" rx="6" fill="#38bdf8" stroke="#0284c7" stroke-width="1.5"/>
  <text x="203" y="164" fill="#0f172a" font-size="9" text-anchor="middle">下一跳</text>
  <rect x="105" y="188" width="80" height="46" rx="6" fill="#334155" stroke="#94a3b8" stroke-width="1.5"/>
  <text x="145" y="211" fill="#e2e8f0" font-size="11" text-anchor="middle">Wan1</text>
  <rect x="193" y="188" width="125" height="44" rx="6" fill="#334155" stroke="#38bdf8" stroke-width="1.5"/>
  <text x="255.5" y="208" fill="#e2e8f0" font-size="11" text-anchor="middle">LAN口</text>
  <text x="255.5" y="224" fill="#7dd3fc" font-size="10" text-anchor="middle">ip 192.168.1.1</text>

  <!-- 旁路由 -->
  <rect x="430" y="280" width="190" height="110" rx="10" fill="#1e293b" stroke="#4ade80" stroke-width="2"/>
  <text x="525" y="310" fill="#4ade80" font-size="15" text-anchor="middle">旁路由 </text>
  <text x="525" y="335" fill="#94a3b8" font-size="11" text-anchor="middle">单网口、同网段</text>
  <text x="525" y="354" fill="#86efac" font-size="12" text-anchor="middle">ip 192.168.1.2</text>


  <!-- 光猫 -->
  <rect x="90" y="350" width="190" height="60" rx="10" fill="#1e293b" stroke="#a78bfa" stroke-width="2"/>
  <text x="185" y="386" fill="#c4b5fd" font-size="15" text-anchor="middle">光猫 </text>

  <!-- 绿：终端设备 -> LAN口 -> 规则 -->
  <path d="M430 270 L306 270 L306 210 L306 136 L265 136" stroke="#4ade80" stroke-width="2" marker-end="url(#arrG)" fill="none"/>
  <path d="M430 270 L368 270" stroke="none" fill="none" marker-end="url(#arrG)"/>
  <path d="M306 270 L306 240" stroke="none" fill="none" marker-end="url(#arrG)"/>
  <path d="M306 210 L306 173" stroke="none" fill="none" marker-end="url(#arrG)"/>
  <path d="M306 136 L285.5 136" stroke="none" fill="none" marker-end="url(#arrG)"/>
  <path id="pg" d="M430 270 L306 270 L306 210 L306 136 L263 136" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotG)">
    <animateMotion dur="2s" repeatCount="indefinite"><mpath xlink:href="#pg"/></animateMotion>
  </circle>

  <!-- 红：规则 -> 下一跳 -> LAN口左侧 -> 垂出 -> 旁路由 -> LAN口左下 -> Wan1右下 -> 光猫 -->
  <path d="M185 143 L203 161 L210 210 L210 290 L440 290 L440 344 L200 344 L200 226 L170 226 L170 348" stroke="#f87171" stroke-width="2" marker-end="url(#arrR)" fill="none"/>
  <path d="M210 210 L210 250" stroke="none" fill="none" marker-end="url(#arrR)"/>
  <path d="M210 290 L325 290" stroke="none" fill="none" marker-end="url(#arrR)"/>
  <path d="M440 290 L440 317" stroke="none" fill="none" marker-end="url(#arrR)"/>
  <path d="M200 344 L200 285" stroke="none" fill="none" marker-end="url(#arrR)"/>
  <path d="M170 226 L170 287" stroke="none" fill="none" marker-end="url(#arrR)"/>
  <path id="pr" d="M185 143 L203 161 L210 210 L210 290 L440 290 L440 344 L200 344 L200 226 L170 226 L170 346" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotR)">
    <animateMotion dur="6s" begin="2s" repeatCount="indefinite"><mpath xlink:href="#pr"/></animateMotion>
  </circle>

  <!-- 蓝：规则 -> Wan1 -> 直连出网 -->
  <path d="M185 143 L145 211 L145 348" stroke="#60a5fa" stroke-width="2" marker-end="url(#arrB)" fill="none"/>
  <path d="M185 143 L165 177" stroke="none" fill="none" marker-end="url(#arrB)"/>
  <path d="M145 211 L145 279.5" stroke="none" fill="none" marker-end="url(#arrB)"/>
  <path id="pb" d="M185 143 L145 211 L145 346" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotB)">
    <animateMotion dur="4s" begin="2s" repeatCount="indefinite"><mpath xlink:href="#pb"/></animateMotion>
  </circle>
</svg>

### isp+ 域名分流

<svg viewBox="0 0 840 520" width="840" height="520" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" style="width:100%;height:auto;background:#0f172a;border-radius:8px;">
  <defs>
    <marker id="arrG2" markerWidth="8" markerHeight="8" refX="6" refY="4" orient="auto"><path d="M0,0 L8,4 L0,8 z" fill="#22c55e"/></marker>
    <marker id="arrB2" markerWidth="8" markerHeight="8" refX="6" refY="4" orient="auto"><path d="M0,0 L8,4 L0,8 z" fill="#3b82f6"/></marker>
    <marker id="arrR2" markerWidth="8" markerHeight="8" refX="6" refY="4" orient="auto"><path d="M0,0 L8,4 L0,8 z" fill="#ef4444"/></marker>
  </defs>
  <g transform="translate(-180,0)">

  <!-- 终端设备 -->
  <rect x="715" y="115" width="180" height="86" rx="10" fill="#1e293b" stroke="#22c55e" stroke-width="2"/>
  <text x="805" y="148" fill="#e2e8f0" font-size="14" text-anchor="middle">终端设备</text>
  <text x="805" y="172" fill="#4ade80" font-size="12" text-anchor="middle">ip 192.168.1.100</text>
  <text x="805" y="191" fill="#94a3b8" font-size="11" text-anchor="middle">网关 192.168.1.1（爱快 LAN）</text>

  <!-- 爱快 -->
  <rect x="283" y="20" width="325" height="200" rx="10" fill="#1e293b" stroke="#2563eb" stroke-width="2"/>
  <text x="445" y="52" fill="#fca5a5" font-size="14" text-anchor="middle">爱快主路由 iKuai</text>

  <rect x="297" y="66" width="274" height="36" rx="6" fill="#334155" stroke="#f87171" stroke-width="1.5"/>
  <text x="434" y="90" fill="#e2e8f0" font-size="9" text-anchor="middle">规则：IP匹配自定义虚拟运营商或者域名匹配</text>

  <rect x="390" y="114" width="92" height="40" rx="6" fill="#334155" stroke="#f87171" stroke-width="1.5"/>
  <text x="436" y="139" fill="#e2e8f0" font-size="10" text-anchor="middle">规则：来源 IP</text>

  <rect x="510" y="162" width="88" height="40" rx="6" fill="#334155" stroke="#94a3b8" stroke-width="1.5"/>
  <text x="554" y="178" fill="#e2e8f0" font-size="10" text-anchor="middle">LAN 口</text>
  <text x="554" y="193" fill="#94a3b8" font-size="8" text-anchor="middle">ip 10.0.0.1</text>

  <rect x="305" y="162" width="86" height="40" rx="6" fill="#334155" stroke="#94a3b8" stroke-width="1.5"/>
  <text x="348" y="182" fill="#e2e8f0" font-size="11" text-anchor="middle">WAN1 真实</text>

  <rect x="400" y="162" width="82" height="40" rx="6" fill="#fbbf24" stroke="#d97706" stroke-width="1.5"/>
  <text x="441" y="182" fill="#000" font-size="11" text-anchor="middle">WAN2</text>
  <text x="441" y="198" fill="#000" font-size="10" text-anchor="middle">ip 10.0.0.2</text>

  <!-- 旁路由 -->
  <rect x="690" y="245" width="230" height="180" rx="10" fill="#1e293b" stroke="#7dd3fc" stroke-width="2"/>
  <text x="805" y="275" fill="#7dd3fc" font-size="14" text-anchor="middle">旁路由</text>
  <text x="805" y="291" fill="#94a3b8" font-size="11" text-anchor="middle">双网口、跨网段</text>

  <rect x="705" y="306" width="200" height="44" rx="6" fill="#334155" stroke="#7dd3fc" stroke-width="1.5"/>
  <text x="805" y="324" fill="#7dd3fc" font-size="11" text-anchor="middle">WAN 口 ip 192.168.1.2</text>
  <text x="805" y="341" fill="#94a3b8" font-size="10" text-anchor="middle">网关 192.168.1.1（接爱快 LAN, 出网）</text>

  <rect x="705" y="362" width="200" height="44" rx="6" fill="#334155" stroke="#7dd3fc" stroke-width="1.5"/>
  <text x="805" y="380" fill="#7dd3fc" font-size="11" text-anchor="middle">LAN 口 ip 10.0.0.1</text>
  <text x="805" y="397" fill="#94a3b8" font-size="10" text-anchor="middle">（接爱快 WAN2）</text>

  <!-- 光猫 -->
  <rect x="290" y="410" width="210" height="60" rx="10" fill="#1e293b" stroke="#a78bfa" stroke-width="2"/>
  <text x="395" y="446" fill="#c4b5fd" font-size="14" text-anchor="middle">光猫</text>

  <!-- 绿：客户端 -> LAN 口 -> 规则（线、箭头、小球均为绿色） -->
  <path d="M805 158 L554 182 L571 84" stroke="#22c55e" stroke-width="2" fill="none"/>
  <polygon points="571,84 579,79 579,89" fill="#22c55e"/>
  <path id="pg2" d="M805 158 L554 182 L571 84" stroke="none" fill="none"/>
  <circle r="7" fill="#22c55e">
    <animateMotion dur="2s" repeatCount="indefinite"><mpath xlink:href="#pg2"/></animateMotion>
  </circle>

  <!-- 蓝：规则 -> WAN1 -> 直连出网（独立蓝线到光猫，规则到 WAN1 直线连接） -->
  <path d="M395 102 L300 182 L338 408" stroke="#3b82f6" stroke-width="2" fill="none"/>
  <polygon points="338,408 333,397 343,397" fill="#3b82f6"/>
  <path id="pb2" d="M395 102 L300 182 L338 408" stroke="none" fill="none"/>
  <circle r="7" fill="#3b82f6">
    <animateMotion dur="6s" repeatCount="indefinite"><mpath xlink:href="#pb2"/></animateMotion>
  </circle>

  <!-- 红：规则 -> WAN2 -> 旁路由LAN -> 旁路由WAN -> 主路由LAN口 -> 来源IP -> WAN1 -> 光猫（独立红线到光猫，规则到 WAN2 走右侧空隙、回程经 LAN口->来源IP->WAN1） -->
  <path d="M395 102 L395 106 L490 106 L490 182 L441 182 L441 210 L608 210 L805 384 L805 328 L554 182 L436 134 L348 182 L362 408" stroke="#ef4444" stroke-width="2" fill="none"/>
  <polygon points="362,408 357,397 367,397" fill="#ef4444"/>
  <path id="pr2" d="M395 102 L395 106 L490 106 L490 182 L441 182 L441 210 L608 210 L805 384 L805 328 L554 182 L436 134 L348 182 L362 408" stroke="none" fill="none"/>
  <circle r="7" fill="#ef4444">
    <animateMotion dur="6s" repeatCount="indefinite"><mpath xlink:href="#pr2"/></animateMotion>
  </circle>

  <!-- 方向箭头：仅放在直线段中点，朝运动方向；拐弯处不加箭头 -->
  <path d="M684.5 169.5 L674.5 170.5" stroke="#22c55e" stroke-width="1" marker-end="url(#arrG2)"/>
  <path d="M561.6 137.9 L563.4 128.1" stroke="#22c55e" stroke-width="1" marker-end="url(#arrG2)"/>
  <path d="M351.3 138.8 L343.7 145.2" stroke="#3b82f6" stroke-width="1" marker-end="url(#arrB2)"/>
  <path d="M318.2 290.1 L319.8 299.9" stroke="#3b82f6" stroke-width="1" marker-end="url(#arrB2)"/>
  <path d="M490 139 L490 149" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M519.5 210 L529.5 210" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M683.8 257.5 L675.2 252.5" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M396.4 160.4 L387.6 155.6" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  </g>
</svg>
