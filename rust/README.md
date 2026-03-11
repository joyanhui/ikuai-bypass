# Rust 版本（开发中）

本目录用于承载 Rust 重写实现（CLI + Tauri App），与仓库现有 Go 版本并存，确保迁移期间不破坏现有发布与使用方式。

- `core/`：纯业务核心库（配置、iKuai API、模块更新、日志、调度）
- `cli/`：Rust CLI 入口（对齐 Go flags/运行模式）
- `app/`：Tauri v2 + Bun + Astro 前端与 App 壳（后续落地）

对应的总体路线与验收标准见仓库根目录：`rust重构计划.md`。
