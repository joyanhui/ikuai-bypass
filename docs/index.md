---
title: 首页
permalink: /
nav_order: 1
---

# iKuai Bypass

爱快路由器专用的分流规则自动同步工具。

[![GitHub release](https://img.shields.io/github/v/release/joyanhui/ikuai-bypass?style=flat-square&logo=github&label=最新版本)](https://github.com/joyanhui/ikuai-bypass/releases)
[![GitHub stars](https://img.shields.io/github/stars/joyanhui/ikuai-bypass?style=flat-square&logo=github&label=Stars)](https://github.com/joyanhui/ikuai-bypass)
[![GitHub downloads](https://img.shields.io/github/downloads/joyanhui/ikuai-bypass/total?style=flat-square&logo=github&label=下载总量)](https://github.com/joyanhui/ikuai-bypass/releases)

从订阅地址自动同步域名分流、IP 分组、自定义运营商等规则到爱快，实现 OpenWRT 旁路无感分流——国内网站访问更快、旁路故障不影响基础上网、恢复后网络自愈。

## 特性

- **安全优先** — 远程资源下载失败时跳过更新，不清理旧规则（Safe-Before）
- **原地更新** — 匹配则编辑、不匹配则新增，爱快内部 ID 保持稳定
- **多平台** — 支持 Docker / CLI / OpenWRT ipkg / PVE / Unraid 等部署方式
- **WebUI + GUI** — 浏览器管理界面与桌面/手机 App 双入口
- **规则兼容** — 支持自定义运营商、域名分流、端口分流、广告屏蔽

## 🚀 入门

- [快速上手]({{ '/quickstart/' | relative_url }}) — 下载、配置、运行定时任务
- [爱快两种分流模式解析]({{ '/router-mode/' | relative_url }}) — 自定义运营商 / IP 分组模式
- [CLI 参数说明]({{ '/cli-params/' | relative_url }}) — 命令行参数、运行模式、分流模式
- [FAQ - 常见问题]({{ '/faq/' | relative_url }}) — 故障排查与使用疑问

## 📦 部署方案

- [部署方案总览]({{ '/deployment/' | relative_url }}) — Docker、CLI、ipkg、PVE、Unraid 全场景
- [OpenWRT 安装为系统服务]({{ '/openwrt-service-install/' | relative_url }}) — 开机自启配置
- [iKuai v4 应用市场 ipkg 安装]({{ '/ikuai-v4-ipkg-install-guide/' | relative_url }}) — 直接在爱快中安装

## 📖 参考

- [更新日志]({{ '/updatelog/' | relative_url }}) — 版本历史与变更记录
- [配置详解与升级指南]({{ '/v4.4.13-update-to-v4.4.10x/' | relative_url }}) — 配置文件参数说明与升级
- [proxy 与 github-proxy 的区别]({{ '/proxy-vs-github-proxy-guide/' | relative_url }}) — 两种代理配置说明
- [爱快 4.0.210+ 端口分流参数错误]({{ '/ikuai-firmware-4.0.210-port-streaming-mode-fix/' | relative_url }}) — mode 5/6 兼容性
- [v4.2.x 注意事项]({{ '/v4.2.x-notes/' | relative_url }}) — 旧版兼容性与配置说明

> **版本提示**：爱快 v3.7x 请用 [v4.2.0](https://github.com/joyanhui/ikuai-bypass/releases/tag/v4.2.0)。旧版 Go 归档至 [v4.4.13](https://github.com/joyanhui/ikuai-bypass/releases/tag/v4.4.13)，升级前阅读[说明]({{ '/v4.4.13-update-to-v4.4.10x/' | relative_url }})。  
> **DNS 分流**：可搭配 [ADGuard Home 规则](https://github.com/joyanhui/adguardhome-rules) 自建 DNS 分流解析。

> 如果本项目对你有帮助，请点个 Star！有问题或建议欢迎提交 [GitHub Issues](https://github.com/joyanhui/ikuai-bypass/issues)。
