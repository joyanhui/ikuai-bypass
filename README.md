## iKuai Bypass
ikuai 可以通过分流规则 把openwrt或者其他路由作为爱快的上级虚拟运营商，同时作为ikuai的下级路由，再把openwrt的出口流量绑回到爱快实际的运营商，实现无感分流：国内网站访问速度更好、不用单独配置网关、openwrt炸了不影响正常上网、openwrt恢复后网络自愈。[查看具体实现方式](https://dev.leiyanhui.com/route/ikuai-bypass-joyanhui/) 或者查看  [恩山eezz](https://www.right.com.cn/forum/thread-8252571-1-1.html) 或者下文 [分流模式的简单说明](https://github.com/joyanhui/ikuai-bypass?tab=readme-ov-file#%E5%88%86%E6%B5%81%E6%A8%A1%E5%BC%8F%E7%AE%80%E5%8D%95%E8%AF%B4%E6%98%8E)。
这种方式比传统用openwrt的作为旁路由的指定网关的方案，或者only openwrt的方案更加稳定，速度更好。   
但是因为大家喜闻乐见的分流规则数据可能几万条，在ikuai上维护更新比较麻烦，这个工具就是为了自动从订阅地址更新爱快的分流规则的域名分流和运营商分流。  
> 如有问题或建议都可以提[issues](https://github.com/joyanhui/ikuai-bypass/issues)，我会尽快处理。给个star我会很开心。
## 分流模式简单说明
ikuai可以是物理机也可以是虚拟机。openwrt同样可以是物理机也可以是虚拟机，也可以是lxc/docker也可以部署到爱快内。   
ikuai需要分配3个网口（分别绑定到wan1 wan2 lan1），openwrt需要2个（wan和lan）。可以是物理网卡也可以是虚拟网卡。  
<details>
<summary>点击这里展开查看详细图文说明</summary>
<img src="assets/img.png" >
</details>

## 主要修改点
- 两个协程并发处理运营商/IP分流和域名分流,更新速度更快。  
- 更新成功后再删除旧规则,原版会先删除,如果更新失败就全部丢了，这也是自己下手修改的主要原因。   
- 支持无docker运行，当然也支持docker运行。    
- 编译了 linux  macos windows freebsd 的二进制，当然也支持openwrt、老毛子和有shell权限其他系统。   
- 编译了 arm5-7 arm64  mipsle mips64le ppc64le amd64 386 等架构并用upx压缩，覆盖多数路由器和os系统。    
- 支持清理模式，单次更新模式，先更新一次再等计划任务触发模式，等待计划任务触发模式。

## 参数说明
- `-c` : 配置文件路径  默认为`config.yml` 可用相对路径或者绝对路径
- `-r` : 运行模式 默认为`cron`
    - `cron` : 先运行一次 而后等待计划任务触发
    - `nocron` 或 `once`或 `1`: 忽略配置文件的cron定时配置配置 运行一次然后就退出结束
    - `cronAft` : 先不运行等计划任务触发
    - `clean` : 清理模式 默认可选附加参数为 `-tag cleanAll`
- `-tag` : 备注信息 清理模式下的附加参数 
    - 默认为cleanAll(即清理所有备注中包含`IKUAI_BYPASS`字符的规则) 
    - 单独指定备注的关键词 可以不添写`IKUAI_BYPASS_`前缀 例如`-r clean -tag ipcn` 或 `-r clean -tag IKUAI_BYPASS_ipcn`

## 更新日志
- 2024-03-07 openwrt服务安装脚本增加无代理环境安装  
- 2024-02-25 增加去广告功能演示规则  [[参考]](https://github.com/joyanhui/ikuai-bypass/blob/main/config_example.yml)
- 2024-02-7 添加一个openwrt下开机自动运行 [[参考脚本]](https://github.com/joyanhui/ikuai-bypass/blob/main/script-example/AddOpenwrtService.sh)
- 2024-02-1 优化清理模式的提示信息，增加`once`或 `1`模式等同于nocron模式
- 2024-02-1 某一分组规则更新失败导致相关的旧规则被删除的bug  [[#3]](https://github.com/joyanhui/ikuai-bypass/issues/3)   
- 2024-02-1 清理模式增加附加参数`-tag` 可以清理全部备注名包含`IKUAI_BYPASS`的分流规则，或者指定备注名全程或者后缀名的分流规则   
- 旧的更新记录没啥价值也未单独记，小工具代码简单，请参考commit记录
## 简要使用说明
从 Releases [[下载]](https://github.com/joyanhui/ikuai-bypass/releases)  解压后得到一个可执行文件`ikuai-bypass`和一个通用配置文件 `config.yml` [[参考]](https://github.com/joyanhui/ikuai-bypass/blob/main/config_example.yml)      
编辑一下`config.yml`里面ikuai的地址用户名和密码，然后在可以访问到ikuai的设备上执行命令格式如下: ` ./ikuai-bypass -c /配置文件路径/config.yml -r 运行模式`即可。      
example:     
`./ikuai-bypass` 等同 `./ikuai-bypass -c config.yml -r cron`: 将根据配置文件的内容更新分流规则更新成功后删除旧的分流规则 并在配置文件的cron的时间按照计划任务 重新更新。    
`./ikuai-bypas -r clean` 等同 `./ikuai-bypass -c config.yml -r clean -tag  cleanAll` :   删除所有备注包含 `IKUAI_BYPASS`的规则   
`./ikuai-bypas -r clean  -tag IKUAI_BYPASS_ipcn` 等同 `./ikuai-bypas -r clean  -tag ipcn`:  删除备注为 `IKUAI_BYPASS_ipcn` 的分流规则   

## 不同平台下
### linux(推荐openwrt内直接运行)
下载 linux-xxx.zip,unzip 后在shell运行。 建议把ikuai-bypass作为服务安装到openwrt [[参考安装脚本]](https://github.com/joyanhui/ikuai-bypass/blob/main/script-example/AddOpenwrtService.sh) 
### docker
下载linux版本，参考命令如下
```sh
mkdir ~/ikuai-bypass/ && cd ~/ikuai-bypass
# 下载amd64版本，如arm版本自行修改
wget -c https://github.com/joyanhui/ikuai-bypass/releases/download/v0.2.2/ikuai-bypass-linux-amd64.zip
unzip ikuai-bypass-linux-amd64.zip
# 编辑默认的 config.yml  略
# 创建容器 docker/podman
docker run -itd  --name ikuai-bypass  --privileged=true --restart=always   \
    -v  ~/ikuai-bypass/:/opt/ikuai-bypass/   \
    alpine:3.18.4  /opt/ikuai-bypass/ikuai-bypass -c  /opt/ikuai-bypass/config.yml -r cron
```
### ikuai docker下
因为ikuai 无法直接执行shell命令,实在懒得给这种小工具打包镜像，尤其是基于的golang根本没有外部依赖只是一个可执行文件。
如果您要在ikuai的docker内运行。请自行下载 linux版本。解压后 上传可执行文件和配置文件 到ikuai数据盘。例如/data0/ikuai-bypass/ikuai-bypass  /data0/ikuai-bypass/config.yml
而后在ikuai的docker中随便下载一个通用的linux镜像,例如 alpine:3.18.4 。创建docker 目录挂载 `/data0/ikuai-bypass/` 到容器内 `/opt/ikuai-bypass/`
入口命令修改为:
```sh
chmod +x /opt/ikuai-bypass/ikuai-bypass  && /opt/ikuai-bypass/ikuai-bypass -r cron -c  /opt/ikuai-bypass/config.yml
```
###  windows
请在 releases 里面点击 `show all xx assets` 可以看到windows的包 下载解压 cmd下cd到解压后的目录运行里面的exe程序
### macos下
下载 darwin-arm64.zip 或者darwin-amd64.zip,unzip 后 在shell运行
## v0.1.15 升级0.2版
v0.2.x 以后规则的备注不再只有字符`IKUAI_BYPASS`，会根据tag添加指定的后缀，所以升级到0.2.x后最好清理掉旧的分流规则重新添加。
另外新版配置文件中每条规则都多了一个 `tag: 备注后缀` 用于区分不同的规则 [[参考]](https://github.com/joyanhui/ikuai-bypass/blob/main/config_example.yml)
```sh
./ikuai-bypass -c /路径/config.yml -r clean -tag cleanAll # 清理所有备注名包含`IKUAI_BYPASS`的分流规则
./ikuai-bypass -c /路径/config.yml -r cron #先运行一次 而后等待计划任务触发 
```

## 其他相关说明
[https://dev.leiyanhui.com/route/ikuai-bypass-joyanhui/](https://dev.leiyanhui.com/route/ikuai-bypass-joyanhui/)

## 致谢
- [恩山ztc1997](https://github.com/ztc1997/ikuai-bypass/): 本项目fork自ztc1997, 感谢原作者实现的核心功能，我只是简单缝补了几个小地方。
- [JetBrains](https://jb.gg/OpenSourceSupport): 为本项目提供免费的开源许可证
- [neovim](https://neovim.io/),[NvChad](https://github.com/NvChad/NvChad),[elulcao](https://github.com/elulcao/NvChad-custom)
