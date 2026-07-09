---
title: 爱快应用市场 ipkg 安装
parent: 部署方案纵览
nav_order: 2
---

## iKuai v4 应用市场 ipkg 安装

### 1. 下载 ipkg 包

打开 [GitHub Releases](https://github.com/joyanhui/ikuai-bypass/releases)，选择对应版本下载 ipkg 包。

> 目前仅支持 **x86_64** 和 **arm64** 架构的爱快路由器。

### 2. 上传并安装

登录爱快路由器的 Web 管理界面，进入 **应用工具 > 应用管理 > 应用市场**，点击 **本地安装**，选择下载的 ipkg 文件上传，安装并启动服务。

> 另可参考 [PR #118](https://github.com/joyanhui/ikuai-bypass/pull/118) 的安装说明（注意：爱快新旧版本界面有差异，PR 中的截图可能已过时）。

### 3. 访问管理界面

安装启动后，通过浏览器访问：

```
http://<设备IP>:<WebUI端口>
```

默认端口：**19001**  
默认用户名：**admin**  
默认密码：**admin888**

### 4. 初始化配置

首次登录后，请先在配置中修改 **爱快路由器地址** 及 **管理员账号/密码**，否则 iKuai-Bypass 无法正常与爱快路由器通信。
