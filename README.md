# iKuai Bypass
fork 自 https://github.com/ztc1997/ikuai-bypass/

## 功能
ip分流规则和域名分流规则添加到爱快的自定义运营商和域名分流，并自动更新规则文件。  
## 更新
- 并发处理 运营商/IP分流 和 域名分流
- 更新成功后再删除旧规则  
- 支持无docker运行
### 通常使用方法 
建议在pve/esxi/unraid/群晖/docker/lxc/podman中运行爱快+openwrt，当然也可以物理设备。    
爱快3个或者3个以上网卡/虚拟网卡 作为主路由   
openwrt/其他linux 双网卡/虚拟网卡，作为下级路由（非旁路由）    
#### 物理网络/虚拟网卡配置 举例
爱快 lan1 绑定到 eth0  开DHCP dhcp范围 10.1.1.5-10.1.1.254   
爱快 wan1 绑定到 eth1  pppoe拨号    wan1先设置为默认线路
爱快 wan2 绑定到 eth2  静态ip指定 10.0.0.2 网关10.0.0.1   

openwrt  wan 绑定到eth1 DHCP客户端，连接到爱快lan1 从爱快静态ip绑定为10.1.1.3
openwrt  lan 绑定套eth0 关闭DHCP服务，ip地址配置 10.1.1.1

#### 避免死循环
爱快添加一个分流规则 流控分流 > 分流设置 > 端口分流   
分流方式：外网线路   线路：wan1  源地址 ip/mac分组 添加一个 10.1.1.3 点加入  保存   
此时 openwrt出来的流量 走wan1 不会再到wan2回到openwrt 导致死循环。  
此时openwrt 可以正常访问外部网络。  
#### 配置  wan2 为默认线路
此时所有对外网访问 都会 经过wan2 到openwrt,然后根据你的深度学习软件的分流情况决定是否加密，再回到爱快的lan1 再流向外网。  
此时你的上网速度和传统的旁路由模式完全一样，略慢。
#### 配置国内域名强制走wan1直连
流控分流 > 分流设置 > 域名分流，选择wan1 ，输入几个域名，输入客户端ip,建议 10.1.1.5-10.1.1.254 ，此时10.1.1.5-10.1.1.254 的设备访问这个域名会直接走wan1 , 不经过openwrt，速度飞快。   
当然你自己输入太麻烦了。ikuai-bypass 可以自动帮你维护这个域名列表。  
#### 配置某些域名强制走wan2 交给openwrt处理
目的是啥，你肯定知道。配置和上面一样，同样 ikuai-bypass 可以自动帮你维护这个域名列表。 
#### 配置国内ip强制走wan1
因为域名清单里面的域名不能覆盖所有网站，还有一些是没有域名直接ip连接情况。所以需要维护一个根据ip分流的规则。   
流控分流 > 分流设置 > 多线负载  自定义运营商  添加运营商  名称：`国内ip地址` 目的地址：输入几个国内的ip,备注：`自定义`  
返回  流控分流 > 分流设置 > 多线负载  点右侧添加 运营商：选择你刚刚添加的名称，点wan1 后面的启用，保存。  
此时 你访问这个ip上的网站或者其他东西，会强制走wan1 不经过openwrt，速度飞快。     
同样 ikuai-bypass 可以自动帮你维护这个IP地址列表。   
#### 配置默认线路 确保网络100%可用性 
把wan2 配置为默认线路，网络设置 > 内外网设置  wan2 同时打开 
-  默认网关：设此条线路为默认网关   
-  自动切换：掉线自动切换    
-  线路检测：HTTP  www.google.com   
 
此时，访问不在国内域名清单的域名，并且ip不再自定义运营商的`国内ip地址`的服务器，会默认走wan2。如果openwrt死机或者google连不上，会自动临时禁用wan2 默认走wan1。

#### 使用ikuai-bypass 自动维护ip和域名规则。
##### 配置文件
需要自定义一个配置文件 config.yml 格式如下。  
然后运行  ` ./ikuai-bypass -c /路径/config.yml ` 即可在启动时候 自动更新一次规则文件，并在 cron指定的时间内定时运行。  
下面的配置文件 会执行一下操作
1. 自动登陆到 网址是 http://10.1.1.1 的爱快 使用用户名admin 密码 admin888登陆，如果登陆成功。    
2. 运营商/IP分流规则  他会帮你添加 一个 `国内IP列表` 的运营商，你需要去爱快 流控分流 > 分流设置 > 多线负载  添加规则，选择 国内IP列表 启用wan1   
也会添加几个 telegram google 等ip地址，你可以删掉那几行，也可以添加规则 启用wan2     
3. 另外会添加4个域名分流规则 分别强制走wan1 和wan2 你可以在下面配置文件清晰看到。需要注意的是 china-list 和 gfw两个清单 都很长，添加/更新的时候会很慢。你可以根据你的需求决定是否要保留，添加后会增加你访问网址的速度，但是会增加ikuai的性能消耗（大概增加30-120M内存消耗，cpu负载也会增加一点），如果你的ikuai配置很烂，可以去掉其他规则 只使用  国内IP列表  的IP分流即可  。

文件内容参考  https://github.com/joyanhui/ikuai-bypass/blob/main/config_example.yml 

##### 部署ikuai-bypass
ikuai-bypass 只要部署在可以访问到 爱快路由器的地方即可。  
###### 直接运行
```sh
./ikuai-bypass_linux_amd64 -c  /opt/ikuai-bypass/config_example.yml
```
###### openwrt/alpine/rc-server
以openwrt为例
```sh
mkdir -p /root/ikuai-bypass/ && cd /root/ikuai-bypass/
wget -c https://github.com/joyanhui/ikuai-bypass/raw/main/ikuai-bypass_linux_amd64
chmod +x ./ikuai-bypass_linux_amd64
wget -c https://github.com/joyanhui/ikuai-bypass/raw/main/config_example.yml
# 测试
/root/ikuai-bypass/ikuai-bypass_linux_amd64 -c /root/ikuai-bypass/config_example.yml 
```
添加服务
```sh
cat >/etc/init.d/ikuai-bypass<< \EOF
#!/bin/sh /etc/rc.common
#service startup sequence
START=99
start() {
        #start your process with parameters in background
        /root/ikuai-bypass/ikuai-bypass_linux_amd64 -c /root/ikuai-bypass/config_example.yml  &
}
stop() {
           killall ikuai-bypass_linux_amd64
}
EOF
chmod +x /etc/init.d/ikuai-bypass

```
打开 openwrt webui ，系统>启动项，找到 ikuai-bypass 默认是禁止开机启动启动的，点`已禁用` 变成启用状态，然后启动一下。

重启 openwrt,然后shell运行 `ps |grep ikuai-bypass` 验证一下是否开机自动启动

###### docker
我没有打包docker镜像，因为完全没必要，你可以自己用下面的命令启动一个docker   
```sh
mkdir ~/ikuai-bypass/ && cd ～/ikuai-bypass_exe
wget -c https://github.com/joyanhui/ikuai-bypass/raw/main/ikuai-bypass_linux_amd64
chmod +x ./ikuai-bypass_linux_amd64
wget -c https://github.com/joyanhui/ikuai-bypass/raw/main/config_example.yml

docker run -itd  --name ikuai-bypass  --privileged=true --restart=always   \
    -v  ~/ikuai-bypass/:/opt/ikuai-bypass/   \
    alpine:3.18.4  /opt/ikuai-bypass/ikuai-bypass_linux_amd64 -c  /opt/ikuai-bypass/config_example.yml

```
如果你想部署到爱快内的docker里面  下载 alpine镜像，上传两个文件，然后入口 命令修改为 类似下面的命令即可
```sh
chmod +x /opt/ikuai-bypass/ikuai-bypass_linux_amd64  && /opt/ikuai-bypass/ikuai-bypass_linux_amd64 -c  /opt/ikuai-bypass/config_example.yml
```




#### 其他补充
##### 自定义规则和ikuai-bypass的规则
ikuai-bypass 自动维护的规则 都会添加备注 `IKUAI_BYPASS` ，只要你添加的自定义的规则备注不是这个即可。
##### 关于实例配置文件
实例配置文件使用了 https://mirror.ghproxy.com 作为github的代理方便可以在无科学环境更新规则，但是ghproxy有被gfw污染的先例，请自行更新更稳定的或者自建的github代理。  
##### 是否可以替代原版ikuai-bypass
可以直接替代，参考上面的docker配置