---
title: Proxy 与 ghproxy 配置
nav_order: 11
---
## 配置项中 关于 proxy 与 github-proxy 的区别

```yaml
# 全局 HTTP 代理（推荐按需开启）
proxy:
  mode: smart           # custom / system / smart
  url: ""
  user: ""
  pass: ""

# ghproxy（可选，仅用于 GitHub Raw 下载加速）
github-proxy: "https://gh-proxy.com/"
```

### proxy

proxy：代理模式（custom/system/smart）。

- **custom**：所有外部 HTTP 请求走自定义代理（url + 可选 user/pass），并禁用 github-proxy
- **system**：使用系统/环境代理
- **smart**：规则下载/远程载入优先使用 github-proxy 直连，未配置时回退到自定义代理/系统代理

GitHub API 查询新版优先使用自定义代理。

### github-proxy

github-proxy：ghproxy URL 前缀重写，仅对 raw.githubusercontent.com / github.com 生效，主要用于"规则文件下载加速"。它不是网络层代理。

> **提示**：完整配置示例请参考 [config.yml](https://github.com/joyanhui/ikuai-bypass/blob/main/config.yml)，里面有详细注释。GUI 版本可以在界面里直接配置。

