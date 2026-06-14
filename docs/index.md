---
title: 首页
nav_order: 1
permalink: /
---

# iKuai Bypass

爱快路由器专用的分流规则自动同步工具。

[![Download](https://img.shields.io/badge/Download-%E4%B8%8B%E8%BD%BD%20Release-blue?style=for-the-badge&logo=github)](https://github.com/joyanhui/ikuai-bypass/releases)
[![GitHub](https://img.shields.io/badge/GitHub-仓库-brightgreen?style=for-the-badge&logo=github)](https://github.com/joyanhui/ikuai-bypass)
[![更新日志](https://img.shields.io/badge/更新日志-Changelog-orange?style=for-the-badge)](updatelog.md)

> **版本注意**：爱快 v3.7x 请用 [v4.2.0](https://github.com/joyanhui/ikuai-bypass/releases/tag/v4.2.0)。爱快 4.x 尚处于 beta 且 API 频繁变更。旧版 Go 已归档至 [v4.4.13](https://github.com/joyanhui/ikuai-bypass/releases/tag/v4.4.13)，老用户升级前请先阅读[升级说明](v4.4.13-update-to-v4.4.10x)。如果本项目对你有帮助，请点个 Star！
>
> **DNS 分流建议**：可搭配 [ADGuard Home 规则](https://github.com/joyanhui/adguardhome-rules) 自建 DNS 分流解析。

---

## 🚀 入门

- [快速上手](quickstart.md) — 下载、配置、运行定时任务
- [爱快两种分流模式解析](router-mode.md) — 自定义运营商 / IP 分组模式
- [WebUI 与 GUI](webui-gui.md) — 网页管理界面与桌面/手机 App
- [CLI 参数说明](cli-params.md) — 命令行参数、运行模式、分流模式

## 📦 部署

- [部署方案总览](deployment.md) — Docker、CLI、ipkg、Unraid 全场景
- [OpenWRT 安装为系统服务](openwrt-service-install) — 开机自启配置
- [iKuai v4 应用市场 ipkg 安装](ikuai-v4-ipkg-install-guide) — 直接在爱快中安装

## 📖 参考

- [更新日志](updatelog.md) — 版本历史与变更记录
- [配置详解与升级指南](v4.4.13-update-to-v4.4.10x) — 配置文件、参数说明、v4.4.13→v4.4.10x 升级
- [proxy 与 github-proxy 的区别](proxy-vs-github-proxy-guide) — 两种代理配置说明
- [注意事项](https://github.com/joyanhui/ikuai-bypass/blob/main/README.md#注意事项) — 规则命名、清理安全、端口配置
