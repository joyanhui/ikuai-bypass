# 快速上手

### 1. 下载

从 [Releases](https://github.com/joyanhui/ikuai-bypass/releases) 下载适合你系统的版本。

**选哪个版本？**

| 你的系统 | 推荐下载 |
| :--- | :--- |
| Windows 桌面 | `ikuai-bypass-gui-windows-x86_64.exe.zip` 解压即可 |
| macOS 桌面 | `ikuai-bypass-gui-macos-aarch64.dmg`（M芯片）或 `x86_64.dmg`（Intel） |
| Linux 桌面 | `ikuai-bypass-gui-linux-x86_64.zip` 解压后运行（需系统已安装 WebKitGTK/GTK3） |
| Android 手机 | `.apk` |
| iOS 手机 | `.ipa`（仅支持自签名或越狱设备） |
| 服务器/路由器/容器 | CLI 版本 `ikuai-bypass-cli-linux-xxx.zip` |
| LXC/PVE CT 容器 | `ikuai-bypass-lxc-alpine-musl-x86_64.tar.gz` |
| Docker | [joyanhui/ikuai-bypass](https://hub.docker.com/r/joyanhui/ikuai-bypass/tags) |
| iKuai v4 应用市场 | `ikuai-bypass-x86_64.ipkg`，在爱快"高级应用 -> 应用市场 -> 本地安装"上传 |

> **新手建议**：如果你在电脑上使用，直接下载 GUI 版本即可；Windows / Linux 解压后运行，macOS 打开 DMG 安装，无需命令行。

### 2. 配置

编辑 `config.yml` 文件，填写以下基本信息：

```yaml
# 爱快路由器地址和登录信息
ikuai-url: http://192.168.9.1   # 改成你的爱快地址
username: admin                   # 登录用户名
password: your_password           # 登录密码

# 定时更新（每天早上7点）
cron: "0 7 * * *"

# 要同步的规则列表
custom-isp:
  - tag: "国内IP"
    url: "https://example.com/cn-ip.txt"
```

> **提示**：完整配置示例请参考 [config.yml](../config.yml)，里面有详细注释。GUI 版本可以在界面里直接配置。
> 关于 proxy 与 github-proxy 的区别 [查看文档](https://joyanhui.github.io/ikuai-bypass/proxy-vs-github-proxy-guide)
### 3. 运行

**GUI 用户**：双击打开应用，在界面里配置即可，无需命令行。

**CLI 用户**：

```bash
# 最常用：定时自动更新（推荐）
./ikuai-bypass -r cron -c ./config.yml
# 只运行一次就退出
./ikuai-bypass -r once -c ./config.yml
# 导出域名分流列表到 TXT（不连接 iKuai，仅用于调试/人工导入）
./ikuai-bypass -r exportDomainSteamToTxt -c ./config.yml -exportPath /tmp
# 清理所有规则（慎用）
./ikuai-bypass -r clean -tag cleanAll -c ./config.yml

# WebUI：cron / cronAft 启动后，若配置中启用 WebUI（webui.enable=true），
# 可直接访问 http://你的IP:19001 查看状态、修改配置、停止定时任务
```
