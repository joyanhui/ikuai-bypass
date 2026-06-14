---
title: CLI 参数说明
nav_order: 12
---

# CLI 参数说明

## 常用参数

| 参数 | 说明 |
| :--- | :--- |
| `-c` | 配置文件路径 |
| `-r` | 运行模式（见下表） |
| `-m` | 分流模式（默认 `ispdomain`，一般不用改） |
| `-tag` | 清理模式必填，指定要清理的规则名 |
| `-login` | 覆盖配置文件登录信息（格式：`http://IP,username,password`） |
| `-exportPath` | 域名分流规则列表导出目录（用于调试/人工检查，默认 `/tmp`） |
| `-isIpGroupNameAddRandomSuff` | IP 分组名称是否增加随机后缀（`1` 开启 / `0` 关闭；默认开启） |

## 运行模式 (`-r`)

| 模式 | 说明 | 使用场景 |
| :--- | :--- | :--- |
| `cron` | 定时运行 | **最常用**，执行依次更新然后切换到任务计划模式等待定时再次触发 |
| `cronAft` | 定时运行 | 暂时不执行，直接进入计划任务模式 |
| `once` | 只运行一次 | 测试配置、手动更新 |
| `clean` | 清理规则 | 删掉所有规则和分组（新备注为 `IkuaiBypass`，兼容旧备注 `joyanhui/ikuai-bypass` / `IKUAI_BYPASS`） |
| `exportDomainSteamToTxt` | 导出域名分流 TXT | 下载 `stream-domain` 的域名列表并导出到 `-exportPath` 目录 |

## 分流模式 (`-m`)

一般使用默认的 `ispdomain` 即可，特殊情况才需要改：

| 模式 | 说明 |
| :--- | :--- |
| `ispdomain` | 运营商+域名分流（默认，推荐） |
| `ipgroup` | IPv4分组模式 |
| `ipv6group` | IPv6分组模式 |
| `ii` | 运营商和域名分流+IPv4分组混合模式 |
| `ip` | IPv4 + IPv6 分组 |
| `iip` | 完整混合模式 ips+domain+ipv4+ipv6 |
