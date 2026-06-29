# OpenWrt LuCI IPK 构建与说明

## 概述

`luci-app-ikuai-bypass` 是一个纯 LuCI 界面包，**不内置 CLI 二进制，也不内置完整安装逻辑**。它只提供 WebUI 和一个很薄的 `/usr/libexec/ikuai-bypass-openwrt` 包装脚本，实际安装、卸载、状态、服务控制统一委托给远程 `docs/install.sh`。

IPK 安装命令：[快速上手 - OpenWrt LuCI 面板](../docs/quickstart.md#openwrt-luci-面板)。

安装后，LuCI 页面的 Install/Update 按钮在后台实际执行：

```sh
curl -fsSL https://joyanhui.github.io/ikuai-bypass/install.sh | sh -s -- install
```

## 工作原理

```
安装 IPK → 访问 LuCI 页面 → 可选填写代理（仅保存到浏览器 localStorage）
                                  ↓
                         点击 Install / Update
                                  ↓
              LuCI POST 当前代理到 /usr/libexec/ikuai-bypass-openwrt
                                  ↓
              wrapper 临时 export http_proxy/https_proxy（不写入路由器）
                                  ↓
              下载远程 install.sh 并执行 sh -s -- install
                                  ↓
              install.sh 下载 CLI 到 /tmp，解压校验后写入 /opt/ikuai-bypass/
                                  ↓
              安装 /etc/init.d/ikuai-bypass，启用并启动服务
```

## install.sh 非交互命令

`docs/install.sh` 现在支持双模式：

| 调用方式 | 行为 |
|---|---|
| `sh install.sh` | 进入交互式菜单 |
| `sh install.sh install [version]` | 安装指定版本或最新版 |
| `sh install.sh update [version]` | 更新指定版本或最新版 |
| `sh install.sh status` | 输出 `key=value` 状态 |
| `sh install.sh start` | 启动服务 |
| `sh install.sh stop` | 停止服务 |
| `sh install.sh restart` | 重启服务 |
| `sh install.sh enable` | 设置开机自启 |
| `sh install.sh disable` | 关闭开机自启 |
| `sh install.sh uninstall --keep-config` | 卸载并保留配置 |
| `sh install.sh uninstall --remove-config` | 卸载并删除配置 |
| `sh install.sh log` | 输出最近日志 |

非交互模式不会读取 `/dev/tty`，便于 LuCI、CI 和管道调用。

## 路径约定

所有 Linux 发行版统一使用 `/opt/ikuai-bypass/`，避免二进制、配置和版本文件分散到多个目录。

| 项目 | 路径 |
|---|---|
| 二进制 | `/opt/ikuai-bypass/ikuai-bypass` |
| 配置 | `/opt/ikuai-bypass/config.yml` |
| 版本文件 | `/opt/ikuai-bypass/.version` |
| init 脚本 | `/etc/init.d/ikuai-bypass` |
| OpenWrt 日志 | 不写入磁盘，服务输出重定向到 `/dev/null` |

Debian/Arch 的 systemd 服务也使用同一套 `/opt/ikuai-bypass/` 路径。

## 卸载语义

LuCI 页面提供两个卸载按钮：

| 按钮 | 行为 |
|---|---|
| Uninstall Service Only | 停止服务、删除开机自启和服务文件；保留 `/opt/ikuai-bypass/` 下的二进制、配置、版本文件 |
| Full Uninstall | 停止服务、删除服务文件，并彻底删除整个 `/opt/ikuai-bypass/` 目录 |

## 代理设计

LuCI 页面提供 HTTP/HTTPS 代理输入框，例如：

```text
http://192.168.1.101:7890
```

代理只保存到当前浏览器的 `localStorage`，不会写入 OpenWrt 文件系统。点击安装时，前端把代理作为 POST 参数传给 Lua 控制器，控制器以 `IKB_PROXY` 环境变量传给 wrapper；wrapper 仅在本次命令中导出：

```sh
http_proxy
https_proxy
HTTP_PROXY
HTTPS_PROXY
```

因此代理配置不会持久化到路由器，也不会影响其他系统命令。

## overlay 兼容策略

OpenWrt 常见为 squashfs root + overlay 可写层。为减少 overlay 污染和半写入文件，安装脚本采用：

1. 下载到 `/tmp/ikuai-bypass-install.*`
2. 在 `/tmp` 中解压和校验
3. 目标目录存在时再写入
4. 文件先写 `/tmp`，再移动到最终路径
5. 配置文件只在不存在时创建

## 包结构

```
luci-app-ikuai-bypass.ipk
├── debian-binary
├── control.tar.gz
│   ├── control
│   └── postinst
└── data.tar.gz
    └── usr/
        ├── lib/lua/luci/
        │   ├── controller/ikuai_bypass.lua
        │   ├── model/cbi/ikuai_bypass.lua
        │   └── view/ikuai_bypass/status.htm
        └── libexec/ikuai-bypass-openwrt
```

## 依赖

IPK 依赖：

```text
luci-base, curl, ca-bundle
```

`unzip` 由 `install.sh` 在运行时通过 `opkg update && opkg install unzip` 自动安装。

## 构建方法

```bash
bash packaging/openwrt-luci/build-openwrt-luci-package.sh \
  <repo-root> \
  <output-dir> \
  <version> \
  <package-arch> \
  <artifact-base>
```

示例：

```bash
bash packaging/openwrt-luci/build-openwrt-luci-package.sh \
  "$PWD" \
  ./release \
  "4.4.107-alpha.3" \
  "all" \
  "luci-app-ikuai-bypass_4.4.107-alpha.3_all"
```

构建脚本使用纯 `tar` 手动打包 IPK，不需要 nfpm。

## 本地验证

```bash
sh -n docs/install.sh
sh -n docs/install-file/common.sh
sh -n packaging/openwrt-luci/luci-app-ikuai-bypass/root/usr/libexec/ikuai-bypass-openwrt

bash packaging/openwrt-luci/build-openwrt-luci-package.sh \
  "$PWD" /tmp/ikb-luci-test 0.0.0-test all luci-app-ikuai-bypass-test
```

OpenWrt KVM 验证：

```bash
bash /home/y/myws/os-config/kvm-config/qemu-openwrt.sh
scp /tmp/ikb-luci-test/*.ipk root@10.0.0.1:/tmp/
ssh root@10.0.0.1 "opkg install /tmp/*.ipk"
ssh root@10.0.0.1 "/usr/libexec/ikuai-bypass-openwrt status"
ssh root@10.0.0.1 "IKB_PROXY=http://192.168.1.101:7890 /usr/libexec/ikuai-bypass-openwrt install"
```

## 变更记录

| 日期 | 变更 |
|---|---|
| 2026-06-27 | 初始版本：简化 IPK 设计，移除 nfpm 依赖，helper 脚本内嵌 init 模板，LuCI 页面简化为状态展示 + 一键安装 |
| 2026-06-28 | IPK helper 改为远程 install.sh 薄包装；install.sh 支持非交互命令；代理仅保存浏览器 localStorage；日志区域改为深色背景；所有 Linux 路径统一到 `/opt/ikuai-bypass/` |
