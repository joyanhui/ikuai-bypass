#!/bin/sh
# example for openrc ( openwrt  or alpine ) 
# Test passed under openwrt5.15.137 x64
mkdir -p /opt/ && cd  /opt/
# 修改成最新版本地址
wget https://github.com/joyanhui/ikuai-bypass/releases/download/v0.2.2/ikuai-bypass-linux-amd64.zip
unzip ikuai-bypass-linux-amd64.zip && rm -rf ikuai-bypass-linux-amd64.zip && rm -rf README.md
mv config.yml  ikuai-bypass.yml 


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

service ikuai-bypass enable

service ikuai-bypass start && ps |grep ikuai-bypass

service ikuai-bypass stop && ps |grep ikuai-bypass

# 卸载 清理
service ikuai-bypass disable
rm -rf /etc/init.d/ikuai-bypass 
rm -rf /opt/ikuai-bypass
rm -rf /opt/ikuai-bypass.yml
