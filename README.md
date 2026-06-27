# iKuai Bypass

![iKuai](https://img.shields.io/badge/Router-iKuai-brightgreen) ![License](https://img.shields.io/badge/License-AGPL%203.0-blue.svg) ![Rust](https://img.shields.io/badge/Language-Rust-orange)

**iKuai Bypass** 是一款爱快路由器专用的分流规则自动同步工具。它可以自动从网上下载 IP/域名列表并同步到你的路由器，让你的流量自动走正确的线路。比如：主流网站和国内ip流量通过光猫直连、特殊流量走旁路由/网关，可以自动同步更新并兼容手动维护的其他分流规则。

提供两种安装方式：
- **GUI**：图形化工具支持桌面和手机 App，支持 Windows / macOS / Linux 和 Android / iOS
- **CLI**：命令行 + 可选 WebUI，适合 服务器/OpenWrt / NAS / PVE / Docker 等部署

> **版本选择**：爱快 v3.7x 请用 [v4.2.0](https://github.com/joyanhui/ikuai-bypass/releases/tag/v4.2.0)。旧版 Go 已归档至 [v4.4.13](https://github.com/joyanhui/ikuai-bypass/releases/tag/v4.4.13)。老用户升级请阅读[升级指南](https://joyanhui.github.io/ikuai-bypass/v4.4.13-update-to-v4.4.10x)。

---
<img src="screenshot/index.gif" alt="">

## 快速导航

| 分类 | 文档链接 |
|:---|:---|
| 🚀 **入门指南** | [功能特性、下载配置、CLI 运行、WebUI/GUI](https://joyanhui.github.io/ikuai-bypass/quickstart) |
| 🔀 **分流模式** | [自定义运营商 vs IP 分组模式详解](docs/router-mode.md) |
| ⚡ **CLI 参数** | [运行模式、分流模式、清理参数](docs/cli-params.md) |
| 📦 **部署方式** | [Docker / CLI / OpenWRT / ipkg 全场景覆盖](docs/deployment.md) |
| 📖 **更新日志** | [版本历史与变更记录](docs/updatelog.md) |
| 📚 **完整文档** | [文档首页](https://joyanhui.github.io/ikuai-bypass/) |

## 快速开始

```bash
# Linux / OpenWrt 一键安装
curl -fsSL https://joyanhui.github.io/ikuai-bypass/install.sh | sh

# Docker
docker run -itd --name ikuai-bypass --restart=always \
  -e APP_RUN_MODE=ispdomain -p 19001:19001 \
  -v ./data:/etc/ikuai-bypass joyanhui/ikuai-bypass:latest
```

> 📖 详细教程：[下载、配置、运行、部署全指南](https://joyanhui.github.io/ikuai-bypass/quickstart.html)

## 赞助支持

虽世道艰难也一直在坚持维护这个项目，您的一份心意能让我更有动力继续完善这个工具，非常感谢您的支持！
- TRX (Tron TRC20) 钱包地址：`TLiv9F6i38uZEGdp8VoB5qLxJx43aV9XSZ`
- 在 GitHub 上给项目点一个 Star 也是莫大的支持！

## Star History

<a href="https://www.star-history.com/?repos=joyanhui%2Fikuai-bypass&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/image?repos=joyanhui/ikuai-bypass&type=date&theme=dark&legend=top-left" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/image?repos=joyanhui/ikuai-bypass&type=date&legend=top-left" />
   <img alt="Star History Chart" src="https://api.star-history.com/image?repos=joyanhui/ikuai-bypass&type=date&legend=top-left" />
 </picture>
</a>

## 交流与反馈

- 交流讨论：[GitHub Discussions](https://github.com/joyanhui/ikuai-bypass/discussions) · [Telegram 电报群](https://t.me/+cosAS1HgFOtlMTc1)
- Bug 反馈：[GitHub Issues](https://github.com/joyanhui/ikuai-bypass/issues)
- 致谢：感谢 [ztc1997](https://github.com/ztc1997/ikuai-bypass/) 的初始版本思路，以及所有 PR 贡献者。
- 欢迎 PR，Rust/TS 代码请须严格遵循零 Clone、零隐式 Panic 及零 Any 原则。
