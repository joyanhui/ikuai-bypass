---
title: 部署方案总览
parent: 📦 部署
nav_order: 1
---
# 部署方案

### 手机和电脑桌面用户（推荐新手）
只要选对正确格式的包就可以，过于简单，教程掠过
### 服务器 / 命令行版本

下载 CLI 版本，用命令行运行。建议配置成系统服务，开机自启动。

**OpenWrt 用户**：可以参考这个[服务脚本](https://github.com/joyanhui/ikuai-bypass/blob/main/golang_archive/example/script/AddOpenwrtService.sh)

### Docker 用户

适合 NAS、群晖、或者喜欢用容器的用户，详见下方 Docker 章节。 注意：docker镜像为 `joyanhui/ikuai-bypass` 其他docker可能是本项目的fork或者网友自治版。

```bash
# 运行（会自动创建配置文件）
docker run -itd --name ikuai-bypass --restart=always \
  -e APP_RUN_MODE=ispdomain \
  -p 19001:19001 -v ./data:/etc/ikuai-bypass \
  joyanhui/ikuai-bypass:latest
```

`APP_RUN_MODE` 用来映射 CLI 的 `-m` 分流模式参数；默认值是 `ispdomain`。可选值：`ispdomain`、`ipgroup`、`ipv6group`、`ii`、`ip`、`iip`。

如果你同时传了容器环境变量 `APP_RUN_MODE` 和命令行 `-m`，则以命令行 `-m` 为准。

启动后：
1. 用浏览器打开 `http://你的IP:19001`
2. 在网页界面里配置爱快地址和登录信息
3. 点击"运行一次"测试，成功后开启定时任务

### iKuai v4 应用市场 / ipkg

如果你打算直接在爱快 `高级应用 -> 应用市场 -> 本地安装` 中上传 `.ipkg` 包，安装流程、参数填写和界面截图请直接参考 [PR #118 使用说明](https://github.com/joyanhui/ikuai-bypass/pull/118)。

应用市场安装时也可以通过环境变量 `APP_RUN_MODE` 配置分流模式，界面字段名为"运行模式"，默认值同样是 `ispdomain`。

### Unraid / 群晖 / 爱快内docker 等部署

在群晖的 Docker 套件里：
1. 搜索 `joyanhui/ikuai-bypass` 并下载
2. 创建容器，映射端口 `19001`
3. 映射一个文件夹到 `/etc/ikuai-bypass` 存放配置
4. 启动后访问网页界面配置

