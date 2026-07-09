---
title: FAQ - 常见问题
nav_order: 4
---

# FAQ - 常见问题

## 目录

- [代理与下载加速配置](#代理与下载加速配置)
- [爱快固件兼容性：端口分流参数变更](#爱快固件兼容性端口分流参数变更)
- [配置升级：v4.4.13 迁移](#配置升级v4413-迁移)

---

## 代理与下载加速配置

代理模式（`proxy`）与 GitHub 下载加速（`github-proxy`）的区别与配置方法。

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

- **custom**：所有外部 HTTP 请求走自定义代理（url + 可选 user/pass），并禁用 github-proxy
- **system**：使用系统/环境代理
- **smart**：规则下载/远程载入优先使用 github-proxy 直连，未配置时回退到自定义代理/系统代理

GitHub API 查询新版优先使用自定义代理。

### github-proxy

ghproxy URL 前缀重写，仅对 `raw.githubusercontent.com` / `github.com` 生效，主要用于规则文件下载加速。它不是网络层代理。

> 完整配置示例请参考 [config.yml](https://github.com/joyanhui/ikuai-bypass/blob/main/config.yml)，里面有详细注释。

---

## 爱快固件兼容性：端口分流参数变更

### 问题现象

执行更新时，端口分流（STREAM:端口分流）报错：

```
[STREAM:端口分流] [UPDATE:更新失败] [1/1] 国内流量: failed: api error: 请求参数不合法
```

### 原因

爱快固件从 `4.0.210` 起，端口分流（stream_ipport）的负载模式中，**`mode: 5`（旧版主备模式）已被移除**，由 `mode: 6`（新版主备模式）替代。

如果配置文件中仍使用 `mode: 5`，iKuai API 会拒绝请求并返回"请求参数不合法"。

### 解决方法

将端口分流配置项的 `mode` 值从 `5` 改为 `6`：

```yaml
stream-ipport:
  - type: "0"
    interface: wan1
    ip-group: 国内
    mode: 6
    ifaceband: 0
```

### 模式兼容性

| mode | 说明 | 兼容固件版本 |
|------|------|-------------|
| 0 | 新建连接数 | 全部 |
| 1 | 源IP | 全部 |
| 2 | 源IP+源端口 | 全部 |
| 3 | 源IP+目的IP | 全部 |
| 4 | 源IP+目的IP+目的端口 | 全部 |
| 5 | 主备模式 | **仅 ≤ 4.0.120** |
| 6 | 主备模式 | **≥ 4.0.210** |

> 固件 ≤ 4.0.120 只能用 `mode: 5`；≥ 4.0.210 只能用 `mode: 6`。

### 相关链接

- [更新日志](updatelog.md) — v4.4.104 对此问题的兼容性修复

---

## 配置升级：v4.4.13 迁移

除了使用 Rust 重构并支持桌面端和手机 app 之外，有以下主要更新：

- 删除了配置项 `enable_update`，在 CLI 端只要是 WebUI 启用，那么就可以在线更新和控制
- CLI 端的 WebUI 支持控制功能：停止和启动服务、查看日志等
- CLI 端的 `-r clean` 模式要求必须强制输入 tags 名字，不会默认 cleanAll
- CLI 端的 `-r web` 参数取消，现在在 `webui.enable=true` 的情况下，`-r cron/cronAft` 都会启动 WebUI
- 除了原 ghproxy 方式之外，新版支持使用传统代理：系统代理/http(s)代理。并有智能模式：更新规则和分组的时候使用 ghproxy，检查更新或其他联网功能使用传统代理。
