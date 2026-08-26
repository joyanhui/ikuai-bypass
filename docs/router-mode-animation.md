---
title: 分流模式 SVG 动画对比
type: docs
weight: 11
---
  
## ip分组和端口分流

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
  <rect x="430" y="160" width="190" height="110" rx="10" fill="#14532d" stroke="#4ade80" stroke-width="2"/>

  <!-- 爱快路由 -->
  <rect x="90" y="40" width="310" height="205" rx="10" fill="#1e293b" stroke="#2563eb" stroke-width="2"/>
  <rect x="105" y="105" width="160" height="38" rx="6" fill="#334155" stroke="#a855f7" stroke-width="1.5"/>
  <rect x="200" y="150" width="54" height="20" rx="6" fill="#38bdf8" stroke="#0284c7" stroke-width="1.5"/>
  <rect x="105" y="188" width="80" height="46" rx="6" fill="#334155" stroke="#94a3b8" stroke-width="1.5"/>
  <rect x="193" y="188" width="125" height="44" rx="6" fill="#334155" stroke="#38bdf8" stroke-width="1.5"/>

  <!-- 旁路由 -->
  <rect x="430" y="280" width="190" height="110" rx="10" fill="#0c4a6e" stroke="#38bdf8" stroke-width="2"/>


  <!-- 光猫 -->
  <rect x="90" y="360" width="190" height="60" rx="10" fill="#1e293b" stroke="#a78bfa" stroke-width="2"/>

  <!-- 绿：终端设备 -> LAN口 -> 规则 -->
  <path d="M430 270 L310 270 L310 210 L350 210 L350 136 L265 136" stroke="#4ade80" stroke-width="4" marker-end="url(#arrG)" fill="none"/>
  <path d="M430 270 L368 270" stroke="none" fill="none" marker-end="url(#arrG)"/>
  <path d="M310 210 L340 210" stroke="none" fill="none" marker-end="url(#arrG)"/>
  <path d="M350 210 L350 173" stroke="none" fill="none" marker-end="url(#arrG)"/>
  <path d="M350 136 L300 136" stroke="none" fill="none" marker-end="url(#arrG)"/>
  <path id="pg" d="M430 270 L310 270 L310 210 L350 210 L350 136 L265 136" stroke="none" fill="none"/>
  <circle r="7" fill="url(#dotG)">
    <animateMotion dur="2s" repeatCount="indefinite"><mpath xlink:href="#pg"/></animateMotion>
  </circle>
  <circle r="7" fill="url(#dotG)">
    <animateMotion dur="2s" begin="1s" repeatCount="indefinite"><mpath xlink:href="#pg"/></animateMotion>
  </circle>

  <!-- 红：规则 -> 下一跳 -> LAN口左侧 -> 垂出 -> 旁路由 -> LAN口左下 -> Wan1右下 -> 光猫 -->
  <path d="M185 143 L202 160 L213 210 L213 290 L440 290 L440 334 L200 334 L200 226 L120 226 L120 358" stroke="#f87171" stroke-width="2" marker-end="url(#arrR)" fill="none"/>
  <path d="M213 235 L213 272" stroke="none" fill="none" marker-end="url(#arrR)"/>
  <path d="M230 290 L320 290" stroke="none" fill="none" marker-end="url(#arrR)"/>
  <path d="M440 290 L440 317" stroke="none" fill="none" marker-end="url(#arrR)"/>
  <path d="M200 334 L200 285" stroke="none" fill="none" marker-end="url(#arrR)"/>
  <path d="M120 226 L120 287" stroke="none" fill="none" marker-end="url(#arrR)"/>
  <path id="pr" d="M185 143 L202 160 L213 210 L213 290 L440 290 L440 334 L200 334 L200 226 L120 226 L120 356" stroke="none" fill="none"/>
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
  <text x="525" y="192" fill="#e2e8f0" font-size="15" text-anchor="middle">终端设备</text>
  <text x="525" y="215" fill="#7dd3fc" font-size="12" text-anchor="middle">ip 192.168.1.100</text>
  <text x="525" y="234" fill="#94a3b8" font-size="11" text-anchor="middle">网关 192.168.1.1</text>
  <text x="525" y="252" fill="#94a3b8" font-size="11" text-anchor="middle">网口连爱快 LAN</text>
  <text x="245" y="70" fill="#fca5a5" font-size="15" text-anchor="middle">爱快主路由 iKuai</text>
  <text x="245" y="90" fill="#94a3b8" font-size="11" text-anchor="middle">LAN ip 192.168.1.1 / WAN1 pppoe</text>
  <text x="185" y="128" fill="#e2e8f0" font-size="12" text-anchor="middle">规则：目标 IP 匹配分组</text>
  <text x="227" y="164" fill="#0f172a" font-size="9" text-anchor="middle">下一跳</text>
  <text x="145" y="208" fill="#e2e8f0" font-size="11" text-anchor="middle">WAN1(PPPOE)</text>
  <text x="255.5" y="208" fill="#e2e8f0" font-size="11" text-anchor="middle">LAN口</text>
  <text x="255.5" y="224" fill="#7dd3fc" font-size="10" text-anchor="middle">ip 192.168.1.1</text>
  <text x="525" y="310" fill="#4ade80" font-size="15" text-anchor="middle">旁路由 </text>
  <text x="525" y="335" fill="#94a3b8" font-size="11" text-anchor="middle">单网口单臂/多网口都可以、同网段</text>
  <text x="525" y="354" fill="#86efac" font-size="12" text-anchor="middle">ip 192.168.1.2</text>
  <text x="185" y="396" fill="#c4b5fd" font-size="15" text-anchor="middle">光猫 </text>
</svg>

### isp+ 域名分流

<svg viewBox="0 0 760 520" width="760" height="520" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" style="width:100%;height:auto;background:#0f172a;border-radius:8px;">
  <defs>
    <marker id="arrG2" markerWidth="8" markerHeight="8" refX="6" refY="4" orient="auto"><path d="M0,0 L8,4 L0,8 z" fill="#22c55e"/></marker>
    <marker id="arrB2" markerWidth="8" markerHeight="8" refX="6" refY="4" orient="auto"><path d="M0,0 L8,4 L0,8 z" fill="#3b82f6"/></marker>
    <marker id="arrR2" markerWidth="8" markerHeight="8" refX="6" refY="4" orient="auto"><path d="M0,0 L8,4 L0,8 z" fill="#ef4444"/></marker>
  </defs>
  <g transform="translate(-260,0)">

  <!-- 终端设备 -->
  <rect x="665" y="165" width="180" height="86" rx="10" fill="#1e293b" stroke="#22c55e" stroke-width="2"/>

  <!-- 爱快 -->
  <rect x="283" y="20" width="355" height="200" rx="10" fill="#1e293b" stroke="#2563eb" stroke-width="2"/>

  <rect x="297" y="66" width="274" height="36" rx="6" fill="#334155" stroke="#a855f7" stroke-width="1.5"/>

  <rect x="450" y="114" width="92" height="40" rx="6" fill="#334155" stroke="#a855f7" stroke-width="1.5"/>

  <rect x="510" y="162" width="88" height="40" rx="6" fill="#334155" stroke="#94a3b8" stroke-width="1.5"/>

  <rect x="295" y="162" width="96" height="40" rx="6" fill="#334155" stroke="#94a3b8" stroke-width="1.5"/>

  <rect x="400" y="162" width="82" height="40" rx="6" fill="#fbbf24" stroke="#d97706" stroke-width="1.5"/>

  <!-- 旁路由 -->
  <rect x="640" y="260" width="230" height="154" rx="10" fill="#1e293b" stroke="#7dd3fc" stroke-width="2"/>

  <rect x="655" y="268" width="200" height="44" rx="6" fill="#334155" stroke="#7dd3fc" stroke-width="1.5"/>

  <rect x="655" y="320" width="200" height="44" rx="6" fill="#334155" stroke="#7dd3fc" stroke-width="1.5"/>

  <!-- 光猫 -->
  <rect x="290" y="440" width="210" height="60" rx="10" fill="#1e293b" stroke="#a78bfa" stroke-width="2"/>

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
  <path d="M395 102 A 90 90 0 0 0 313 182 L313 438" stroke="#3b82f6" stroke-width="2" fill="none" marker-end="url(#arrB2)"/>
  <path id="pb2" d="M395 102 A 90 90 0 0 0 313 182 L313 438" stroke="none" fill="none"/>
  <circle r="7" fill="#3b82f6">
    <animateMotion dur="2s" repeatCount="indefinite" begin="2s"><mpath xlink:href="#pb2"/></animateMotion>
  </circle>
  <circle r="7" fill="#3b82f6">
    <animateMotion dur="2s" repeatCount="indefinite" begin="3s"><mpath xlink:href="#pb2"/></animateMotion>
  </circle>

  <!-- 红：规则 -> WAN2 -> 旁路由LAN -> 旁路由WAN -> 主路由LAN口 -> 来源IP -> WAN1 -> 光猫（独立红线到光猫，规则到 WAN2 走右侧空隙、回程经 LAN口->来源IP->WAN1） -->
  <path d="M395 102 L415 182 L415 360 L665 360 L665 308 L520 308 L520 182 L458 134 L416.8 157.8 Q409.9 146.2 403.0 165.8 L375 182 L375 438" stroke="#ef4444" stroke-width="2" fill="none" marker-end="url(#arrR2)"/>
  <path id="pr2" d="M395 102 L415 182 L415 360 L665 360 L665 308 L520 308 L520 182 L458 134 L416.8 157.8 Q409.9 146.2 403.0 165.8 L375 182 L375 438" stroke="none" fill="none"/>
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
  <path d="M343.2 123.8 L336.0 130.8" stroke="#3b82f6" stroke-width="1" marker-end="url(#arrB2)"/>
  <path d="M313 305 L313 315" stroke="#3b82f6" stroke-width="1" marker-end="url(#arrB2)"/>
  <path d="M398.8 117.2 L401.2 126.9" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M415 266 L415 276" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M535 360 L545 360" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M665 339 L665 329" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M597.5 308 L587.5 308" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M520 250 L520 240" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M493.0 161.1 L485.0 154.9" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M441.7 143.4 L433.1 148.4" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <path d="M375 305 L375 315" stroke="#ef4444" stroke-width="1" marker-end="url(#arrR2)"/>
  <text x="755" y="198" fill="#e2e8f0" font-size="14" text-anchor="middle">终端设备</text>
  <text x="755" y="222" fill="#4ade80" font-size="12" text-anchor="middle">ip 192.168.1.100</text>
  <text x="755" y="241" fill="#94a3b8" font-size="11" text-anchor="middle">网关 192.168.1.1（爱快 LAN）</text>
  <text x="445" y="52" fill="#fca5a5" font-size="14" text-anchor="middle">爱快主路由 iKuai</text>
  <text x="434" y="90" fill="#e2e8f0" font-size="9" text-anchor="middle">规则：IP匹配自定义虚拟运营商或者域名匹配</text>
  <text x="496" y="139" fill="#e2e8f0" font-size="10" text-anchor="middle">规则：来源 IP</text>
  <text x="554" y="178" fill="#e2e8f0" font-size="10" text-anchor="middle">LAN 口</text>
  <text x="554" y="193" fill="#94a3b8" font-size="8" text-anchor="middle">ip 10.0.0.1</text>
  <text x="343" y="182" fill="#e2e8f0" font-size="11" text-anchor="middle">WAN1(PPPOE)</text>
  <text x="441" y="182" fill="#000" font-size="9" text-anchor="middle">WAN2(虚拟运营商)</text>
  <text x="441" y="198" fill="#000" font-size="10" text-anchor="middle">ip 10.0.0.2</text>
  <text x="755" y="386" fill="#7dd3fc" font-size="14" text-anchor="middle">旁路由</text>
  <text x="755" y="402" fill="#94a3b8" font-size="11" text-anchor="middle">双网口、跨网段</text>
  <text x="755" y="286" fill="#7dd3fc" font-size="11" text-anchor="middle">WAN 口 ip 192.168.1.2</text>
  <text x="755" y="303" fill="#94a3b8" font-size="10" text-anchor="middle">网关 192.168.1.1（接爱快 LAN, 出网）</text>
  <text x="755" y="338" fill="#7dd3fc" font-size="11" text-anchor="middle">LAN 口 ip 10.0.0.1</text>
  <text x="755" y="355" fill="#94a3b8" font-size="10" text-anchor="middle">（接爱快 WAN2）</text>
  <text x="395" y="476" fill="#c4b5fd" font-size="14" text-anchor="middle">光猫</text>
  </g>
</svg>
