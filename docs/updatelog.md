---
title: 更新日志
parent: 📖 参考
nav_order: 1
---
# 更新日志

> 仅包含 Rust 版本（v4.4.100+）的更新记录。历史 Go 版本归档于 [v4.4.13](https://github.com/joyanhui/ikuai-bypass/releases/tag/v4.4.13)。

## v4.4.106-alpha.3 (2026-06-14)

- 修复 CLI 版本未能嵌入 WebUI 静态编译结果的 bug [#142](https://github.com/joyanhui/ikuai-bypass/issues/142)

## v4.4.105-alpha

- 修复 OpenWRT LuCI 对 `install` 命令的依赖 [#138](https://github.com/joyanhui/ikuai-bypass/issues/138)
- 增加 arm64 版本爱快 ipkg 插件支持
- iOS 端优化
- 其他代码清理

## v4.4.104 (2026-04-27)

- 针对爱快 v4.0.210beta 端口分流 mode:5 被弃用改为 6 的兼容性修复 [#130](https://github.com/joyanhui/ikuai-bypass/issues/130)
- 增加端口分流的优先级参数配置 `prio`（默认 0）[#128](https://github.com/joyanhui/ikuai-bypass/issues/128)

## v4.4.103 (2026-04-11)

- 修复 macOS/Windows 下 GUI 版本无法智能创建配置文件路径导致无法保存配置文件的 bug
- 完善远程加载配置文件的模态框，增加使用嵌入的 YAML 配置文件功能，增加直接使用 ghproxy 下载功能
- CLI 模式下增加自动创建配置文件能力（配置文件不存在时询问是否创建），GUI 模式在远程下载配置文件界面也有此功能

## v4.4.102

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

