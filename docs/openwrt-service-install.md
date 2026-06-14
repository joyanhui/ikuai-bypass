---
title: OpenWRT 系统服务安装
nav_order: 7
---
## 在 OpenWRT 上安装 ikb (Rust 版) 为系统服务

### 环境要求

- OpenWRT x64 / ARM / MIPS 等架构（Linux 内核 5.4+ 测试通过）
- `ash` / `bash` / `zsh`（不兼容 `fish`）
- 已安装 `unzip` 和 `wget`：

```sh
opkg update && opkg install unzip wget
```

### 一、下载与安装

```sh
# 可选：GitHub 代理（如直连则跳过）
export GhProxy=https://ghp.ci/

# 自动检测架构，选择正确的发布包
case "$(uname -m)" in
  x86_64|amd64)     ARCH="x86_64" ;;
  i686|i386)        ARCH="x86_32" ;;
  aarch64|arm64)    ARCH="aarch64" ;;
  armv5*|arm926*)   ARCH="arm5" ;;
  armv6*|arm1176*)  ARCH="arm6" ;;
  armv7*|cortex-a*) ARCH="arm7" ;;
  mips)             ARCH="mips" ;;
  mipsel)           ARCH="mipsle" ;;
  riscv64)          ARCH="riscv64gc" ;;
  *)                echo "不支持的架构: $(uname -m)"; exit 1 ;;
esac

# 设置版本（可从 Releases 页面查看最新 tag）
# 自动获取最新版本（需 opkg install jq）：
# IKB_VERSION=$(wget -qO- https://api.github.com/repos/joyanhui/ikuai-bypass/releases/latest | grep '"tag_name"' | cut -d'"' -f4)
# 手动指定版本：
IKB_VERSION="4.4.105-alpha.5"

mkdir -p /opt/ && cd /opt/
wget "${GhProxy}https://github.com/joyanhui/ikuai-bypass/releases/download/${IKB_VERSION}/ikuai-bypass-cli-linux-${ARCH}.zip"
unzip ikuai-bypass-cli-linux-${ARCH}.zip && rm -f ikuai-bypass-cli-linux-${ARCH}.zip

# 使用仓库内的演示配置
rm -f config.yml
wget "${GhProxy}https://raw.githubusercontent.com/joyanhui/ikuai-bypass/main/config.yml" -O ikuai-bypass.yml

# 赋予执行权限
chmod +x /opt/ikuai-bypass
```

### 二、手动测试运行

```sh
# 先手动执行一次，确认配置正确
/opt/ikuai-bypass -r once -c /opt/ikuai-bypass.yml
```

### 三、创建开机服务

```sh
cat > /etc/init.d/ikuai-bypass << 'EOF'
#!/bin/sh /etc/rc.common

START=99
STOP=10

start() {
    /opt/ikuai-bypass -r cronAft -c /opt/ikuai-bypass.yml > /dev/null 2>&1 &
    echo "ikuai-bypass is start"
}

stop() {
    killall -q -9 ikuai-bypass 2>/dev/null
    echo "ikuai-bypass is stop"
}

restart() {
    stop
    sleep 1
    start
}
EOF

chmod +x /etc/init.d/ikuai-bypass
service ikuai-bypass enable
service ikuai-bypass start
ps | grep ikuai-bypass
```

### 四、管理命令

```sh
service ikuai-bypass start     # 启动
service ikuai-bypass stop      # 停止
service ikuai-bypass restart   # 重启
service ikuai-bypass enable    # 开机自启
service ikuai-bypass disable   # 关闭自启
```

### 五、卸载清理

```sh
service ikuai-bypass stop
service ikuai-bypass disable
rm -f /etc/init.d/ikuai-bypass
rm -rf /opt/ikuai-bypass
rm -f /opt/ikuai-bypass.yml
```

### 六、参数说明

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `-c` | 配置文件路径 | `config.yml` |
| `-r` | 运行模式：`once`/`cron`/`cronAft`/`clean` | `cron` |
| `-m` | 更新模块：`ispdomain` 等 | `ispdomain` |
| `--tag` | 清理模式目标标签 | - |

> **提示**：`-r cron` = 立即执行一次 + 按 cron 定时；`-r cronAft` = 仅定时模式，不立即执行。

