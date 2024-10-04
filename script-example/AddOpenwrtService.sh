#!/bin/sh
# Test passed under openwrt5.15.137 x64 在openwrt 5.15.137 x64 测试通过
# 下面的脚本只支持ash sh bash zsh 不兼容fishshell 
# 更新或者下载最新版到 /opt/注意修改版本号CPU架构以及路径  =================================== start
# 最好逐行运行
# opkg update
# opkg install wget unzip 
export GhProxy=https://mirror.ghproxy.com/  # 配置github代理 如果不可用请自行更换如果已经有直连github环境也可以去掉这行
mkdir -p /opt/ && cd  /opt/
wget ${GhProxy}https://github.com/joyanhui/ikuai-bypass/releases/download/v1.0.0-beta2/ikuai-bypass-linux-amd64.zip
unzip ikuai-bypass-linux-amd64.zip && rm -rf ikuai-bypass-linux-amd64.zip && rm -rf README.md
# 使用版本内的配置文件
mv config.yml  ikuai-bypass.yml 
# 或者用最新的演示配置
rm -rf ikuai-bypass.yml && rm -rf config.yml
wget ${GhProxy}https://raw.githubusercontent.com/joyanhui/ikuai-bypass/main/config.yml -O ikuai-bypass.yml
# 更新或者下载最新版到 /opt/注意修改版本号CPU架构以及路径  =================================== end
# 手动执行一次 检查执行结果
#   /opt/ikuai-bypass -r 1 -c /opt/ikuai-bypass.yml
# 创建服务脚本，这段代码请整体复制后粘贴，或者使用vim nano编辑  ================================= start
cat > /etc/init.d/ikuai-bypass << \EOF
#!/bin/sh /etc/rc.common
START=99
start(){
        /opt/ikuai-bypass -r cronAft  -c /opt/ikuai-bypass.yml > /dev/null 2>&1 &
        echo "ikuai-bypass  is start"
}
 
stop(){
       killall -q -9  ikuai-bypass
       echo "ikuai-bypass  is stop"
}
EOF
# 创建服务脚本，这段代码请整体复制后粘贴，或者使用vim nano编辑  ================================= end

chmod +x /etc/init.d/ikuai-bypass # 添加执行权限

# 服务设定为开机启动
service ikuai-bypass enable
# 手动启动 并查看进程是否存在
service ikuai-bypass start && ps |grep ikuai-bypass
# 手动停止
# service ikuai-bypass stop && ps |grep ikuai-bypass

# 卸载 清理
# service ikuai-bypass stop
# service ikuai-bypass disable
# rm -rf /etc/init.d/ikuai-bypass 
# rm -rf /opt/ikuai-bypass
# rm -rf /opt/ikuai-bypass.yml


