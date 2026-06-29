---
title: 更新日志：版本历史
nav_order: 9999
---
# 更新日志

> 仅包含 Rust 版本（v4.4.100+）的更新记录。历史 Go 版本归档于 [v4.4.13](https://github.com/joyanhui/ikuai-bypass/releases/tag/v4.4.13)。

## v4.4.108 (2026-06-29)

- 新增 `run-mode` / `mode` 配置字段，支持三优先级解析（CLI 参数 > config.yml > 默认值）
- 修复 `spdomain` 拼写错误为 `ispdomain`（旧版 config.yml 中 `mode: spdomain` 会导致 CLI 启动失败）
- WebUI 运行时 chip 选择与 rawYaml 双向同步；标签切换时编辑器内容同步
- CLI `-r` / `-m` 改为 `Option<String>`，不再强设默认值，由配置文件和默认值共同决定
- LuCI IPK Config 子 tab 布局样式修复

## v4.4.107 (2026-06-29)

- OpenWrt LuCI IPK 重大升级：完整双语标签页界面、配置文件在线编辑器（备份/恢复）、插件自更新、自卸载（双重确认）、步骤进度+实时日志、代理配置模态框
- 新增一键安装脚本 `docs/install.sh`，支持 Ubuntu (systemd) 和 OpenWrt，附带 CI 测试覆盖安装/卸载全生命周期
- 文档重构：合并 quickstart/guide，新增一键安装指引

## v4.4.106 (2026-06-15)

- 修复 CLI 版本未能嵌入 WebUI 静态编译结果的 bug [#142](https://github.com/joyanhui/ikuai-bypass/issues/142)
- 修复 OpenWRT LuCI 对 `install` 命令的依赖 [#138](https://github.com/joyanhui/ikuai-bypass/issues/138)
- 增加 arm64 版本爱快 ipkg 插件支持
- iOS 端优化
- 增加对爱快≥ 4.0.210 端口分流的主备模式说明文案和默认值[aaa2c3c](https://github.com/joyanhui/ikuai-bypass/commit/aaa2c3c0b9d6d06086b2fc1b3558210327c8b2fd)
## v4.4.104 (2026-04-27)

- 针对爱快 v4.0.210beta 端口分流 mode:5 被弃用改为 6 的兼容性修复 [#130](https://github.com/joyanhui/ikuai-bypass/issues/130)
- 增加端口分流的优先级参数配置 `prio`（默认 0）[#128](https://github.com/joyanhui/ikuai-bypass/issues/128)

## v4.4.103 (2026-04-11)

- 修复 macOS/Windows 下 GUI 版本无法智能创建配置文件路径导致无法保存配置文件的 bug
- 完善远程加载配置文件的模态框，增加使用嵌入的 YAML 配置文件功能，增加直接使用 ghproxy 下载功能
- CLI 模式下增加自动创建配置文件能力（配置文件不存在时询问是否创建），GUI 模式在远程下载配置文件界面也有此功能

## v4.4.102 (2026-04-11)

- YAML 编辑器简化为直接使用多行输入框
- 修复 Windows 的 GUI 版本无法显示内置页面（提示页面未找到）的 bug [#127](https://github.com/joyanhui/ikuai-bypass/issues/127)

## v4.4.101 (2026-04-10)

- 备注信息改为 `IkuaiBypass`，避免爱快部分模块不支持特殊字符的困扰
- 支持端口分流配置的"反向匹配" [#119](https://github.com/joyanhui/ikuai-bypass/issues/119)
- 修复添加域名分流和端口分流时 IP 分组无法匹配非本系统维护的分组名的 bug [#125](https://github.com/joyanhui/ikuai-bypass/issues/125)
- Docker 和爱快应用市场支持环境变量 `APP_RUN_MODE` 控制运行模式
- 去掉爱快应用市场的其他环境变量配置，移步到 WebUI/配置文件内配置
- CI/CD 集成测试相关推进

## v4.4.100 (2026-04-05)

- 首个 Rust 版本，开始支持手机 App 和电脑直接可视化配置和使用
- 技术栈从 Go+HTML 迁移到 Rust + Tauri + Astro，引入集成测试和爱快 API 模拟器
- 功能和 v4.4.13 完全对齐
- 提供 PVE LXC/CT 部署方式（`ikuai-bypass-lxc-alpine-musl-x86_64.tar.gz`）
- 支持爱快 v4.0 应用市场 ipkg 安装

