#!/bin/sh
# Test passed under openwrt5.15.137 x64 在openwrt 5.15.137 x64 测试通过

# 更新或者下载最新版到 /opt/注意修改版本号和路径  =============== start
mkdir -p /opt/ && cd  /opt/
wget https://github.com/joyanhui/ikuai-bypass/releases/download/v0.2.2/ikuai-bypass-linux-amd64.zip
unzip ikuai-bypass-linux-amd64.zip && rm -rf ikuai-bypass-linux-amd64.zip && rm -rf README.md
mv config.yml  ikuai-bypass.yml 
# 更新或者下载最新版到 /opt/注意修改版本号和路径  =============== end


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

chmod +x /etc/init.d/ikuai-bypass

# 开机启动
service ikuai-bypass enable
# 手动启动
service ikuai-bypass start && ps |grep ikuai-bypass
# 手动停止
#service ikuai-bypass stop && ps |grep ikuai-bypass

# 卸载 清理
#service ikuai-bypass disable
#rm -rf /etc/init.d/ikuai-bypass 
#rm -rf /opt/ikuai-bypass
#rm -rf /opt/ikuai-bypass.yml


