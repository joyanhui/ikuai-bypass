#!/bin/sh
# Test passed under openwrt5.15.137 x64 在openwrt 5.15.137 x64 测试通过

# 更新或者下载最新版到 /opt/注意修改版本号CPU架构以及路径  =============== start
# 最好逐行运行
# opkg install wget unzip 
mkdir -p /opt/ && cd  /opt/
wget https://github.com/joyanhui/ikuai-bypass/releases/download/v0.2.2/ikuai-bypass-linux-amd64.zip
unzip ikuai-bypass-linux-amd64.zip && rm -rf ikuai-bypass-linux-amd64.zip && rm -rf README.md
mv config.yml  ikuai-bypass.yml 
# 更新或者下载最新版到 /opt/注意修改版本号CPU架构以及路径  =============== end

# 创建服务脚本，这段代码请整体复制后粘贴  ================================= start
# 这段代码整体复制到ssh执行，注意 fish-shell 不兼容cat EOF的写法，请使用openwrt自带的ash或sh/bash/zsh，或者用nano vim 手动编辑
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
# 创建服务脚本，这段代码请整体复制后粘贴  ================================= end

chmod +x /etc/init.d/ikuai-bypass # 添加执行权限

# 服务设定为开机启动
service ikuai-bypass enable
# 手动启动 并查看进程是否存在
service ikuai-bypass start && ps |grep ikuai-bypass
# 手动停止
# service ikuai-bypass stop && ps |grep ikuai-bypass
# 手动执行一次 检查执行结果
#   /opt/ikuai-bypass -r 1 -c /opt/ikuai-bypass.yml
# 卸载 清理
# service ikuai-bypass stop
# service ikuai-bypass disable
# rm -rf /etc/init.d/ikuai-bypass 
# rm -rf /opt/ikuai-bypass
# rm -rf /opt/ikuai-bypass.yml


