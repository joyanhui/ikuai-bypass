## iKuai Bypass

fork 自 https://github.com/ztc1997/ikuai-bypass/
## 功能
自动或手动爱快分流规则,让部分域名或者ip走指定的wan口(通常分流到openwrt)，实现无感不影响他人的分流。
## 主要修改点
- 并发处理 运营商/IP分流 和 域名分流  
- 更新成功后再删除旧规则  
- 支持支持linux macos windows 无docker运行 ,当然也支持docker运行
- 支持单次运行参数`-r nocron`: 忽略配置文件的cron定时配置配置
- 支持单独清理模式`-r clean` :清理本工具添加的备注为`IKUAI_BYPASS`的分流规则
- 支持cron运行参数`-r cron` `-r cronAft` : 先运行一次 而后等待计划任务触发 or 等计划任务时间
## 简要使用说明
需要两个文件 1、可执行程序[下载](https://github.com/joyanhui/ikuai-bypass/releases)  2、配置文件 config.yml [参考](https://github.com/joyanhui/ikuai-bypass/blob/main/config_example.yml)
命令格式: ` ./ikuai-bypass -c /路径/config.yml -r 运行模式`
例如: ` ./ikuai-bypass -c config.yml -r cron` : 将根据配置文件的内容更新分流规则更新成功后删除旧的分流规则 在配置文件的cron的时间 重新更新。

## 不同平台下
###  windows下
请在 releases 里面点击 `show all xx assets` 可以看到windows的包 下载解压 cmd下cd到解压后的目录运行里面的exe程序
### macos下
下载 darwin-arm64.zip 或者darwin-amd64.zip,unzip 后 在shell运行
### linux 下
下载 linux-xxx.zip,unzip 后在shell运行
### docker
下载linux版本
```sh
mkdir ~/ikuai-bypass/ && cd ~/ikuai-bypass
# 下载amd64版本，如arm版本自行修改
wget -c https://github.com/joyanhui/ikuai-bypass/releases/download/v0.1.15/ikuai-bypass-linux-amd64.zip
unzip ikuai-bypass-linux-amd64.zip
# 编辑默认的 config.yml 
docker run -itd  --name ikuai-bypass  --privileged=true --restart=always   \
    -v  ~/ikuai-bypass/:/opt/ikuai-bypass/   \
    alpine:3.18.4  /opt/ikuai-bypass/ikuai-bypass -c  /opt/ikuai-bypass/config.yml -r cron
```
### ikuai docker下
因为ikuai 无法直接执行shell命令,所以自行下载 linux版本。解压后 上传到ikuai数据盘。例如/data0/ikuai-bypass/ikuai-bypass  /data0/ikuai-bypass/config.yml
而后在ikuai的docker中随便下载一个通用的linux镜像,例如 alpine:3.18.4 。创建docker 目录挂载 `/data0/ikuai-bypass/` 到 `/data0/ikuai-bypass/`
入口命令修改为:
```sh
chmod +x /opt/ikuai-bypass/ikuai-bypass  && /opt/ikuai-bypass/ikuai-bypass -c  /opt/ikuai-bypass/config.yml

```
## 详细说明
[https://dev.leiyanhui.com/route/ikuai-bypass-joyanhui/](https://dev.leiyanhui.com/route/ikuai-bypass-joyanhui/)


