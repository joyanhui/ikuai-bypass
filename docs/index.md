**iKuai Bypass** 是爱快路由器专用的分流规则自动同步工具。

[GitHub](https://github.com/joyanhui/ikuai-bypass) — [Releases](https://github.com/joyanhui/ikuai-bypass/releases) — [README](https://github.com/joyanhui/ikuai-bypass/blob/main/README.md)

> **版本注意**：爱快 v3.7x 请用 [v4.2.0](https://github.com/joyanhui/ikuai-bypass/releases/tag/v4.2.0)。爱快 4.x 尚处于 beta 且 API 频繁变更。旧版 Go 已归档至 [v4.4.13](https://github.com/joyanhui/ikuai-bypass/releases/tag/v4.4.13)，老用户升级前请先阅读[升级说明](v4.4.13-update-to-v4.4.10x)。如果本项目对你有帮助，请点个 Star！
>
> **DNS 分流建议**：可搭配 [ADGuard Home 规则](https://github.com/joyanhui/adguardhome-rules) 自建 DNS 分流解析。

---

## 文档

### 🚀 入门
- [快速上手](quickstart.md) — 下载、配置、运行定时任务
- [爱快两种分流模式解析](router-mode.md) — 自定义运营商 / IP 分组模式
- [WebUI 与 GUI](webui-gui.md) — 网页管理界面与桌面/手机 App
- [CLI 参数说明](https://github.com/joyanhui/ikuai-bypass/blob/main/README.md#cli-参数说明) — 命令行参数、运行模式、分流模式

### 📦 部署
- [部署方案总览](deployment.md) — Docker、CLI、ipkg、Unraid 全场景
- [OpenWRT 安装为系统服务](openwrt-service-install) — 开机自启配置
- [iKuai v4 应用市场 ipkg 安装](ikuai-v4-ipkg-install-guide) — 直接在爱快中安装

### 📖 参考
- [配置详解与升级指南](v4.4.13-update-to-v4.4.10x) — 配置文件、参数说明、v4.4.13→v4.4.10x 升级
- [proxy 与 github-proxy 的区别](proxy-vs-github-proxy-guide) — 两种代理配置说明
- [注意事项](https://github.com/joyanhui/ikuai-bypass/blob/main/README.md#注意事项) — 规则命名、清理安全、端口配置

---

💬 [Discussions](https://github.com/joyanhui/ikuai-bypass/discussions) · [Telegram](https://t.me/+cosAS1HgFOtlMTc1) · 🐛 [Issues](https://github.com/joyanhui/ikuai-bypass/issues)

致谢：感谢 [ztc1997](https://github.com/ztc1997/ikuai-bypass/) 的初始版本思路，以及所有 PR 贡献者。欢迎 PR，Rust/TS 代码须遵循零 Clone、零隐式 Panic、零 Any 原则。
