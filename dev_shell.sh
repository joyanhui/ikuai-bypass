go run *.go  -r clean -c  config.yml

go run *.go  -r 1 -c config.yml -m ii


go run *.go  -r 1 -c config.yml -m ipgroup -delOldRule before






# kvm  /home/y/Downloads/iKuai8_x64_3.7.21_Build202508211345.iso

mkdir -p /home/y/kvm
qemu-img create -f qcow2 /home/y/kvm/ikuai.qcow2 32G

qemu-system-x86_64 -M q35,usb=on,acpi=on,hpet=off -m 4G -smp cores=4 -accel kvm \
    -drive file=/home/y/kvm/ikuai.qcow2,if=virtio \
    -drive file=/home/y/Downloads/iKuai8_x64_3.7.21_Build202508211345.iso,index=1,media=cdrom \
    -device usb-tablet -device VGA,vgamem_mb=256 -monitor stdio \
    -nic user,model=e1000,mac=52:54:00:11:11:11,hostfwd=tcp::8080-:80 \
    -nic user,model=e1000,mac=52:54:00:22:22:22,net=10.0.3.0/24 \
    -nic user,model=e1000,mac=52:54:00:33:33:33 \
    -nic user,model=e1000,mac=52:54:00:44:44:44

sudo ip tuntap add dev tap0 mode tap user $(whoami)
sudo ip link set tap0 up
sudo ip addr add 192.168.10.1/24 dev tap0
sudo ip addr del 192.168.10.1/24 dev tap0
sudo ip addr add 192.168.9.2/24 dev tap0


qemu-system-x86_64 -M q35,usb=on,acpi=on,hpet=off -m 4G -smp cores=4 -accel kvm \
    -boot order=dc,menu=on \
    -drive file=/home/y/kvm/ikuai.qcow2,if=virtio \
    -drive file=/home/y/Downloads/iKuai8_x64_4.0.0_Build202512241218.iso,index=1,media=cdrom \
    -device usb-tablet -device VGA,vgamem_mb=256 -monitor stdio \
    -nic tap,ifname=tap0,script=no,downscript=no,model=e1000,mac=52:54:00:11:11:11 \
    -nic user,model=e1000,mac=52:54:00:22:22:22 \
    -nic user,model=e1000,mac=52:54:00:33:33:33 \
    -nic user,model=e1000,mac=52:54:00:44:44:44



qemu-system-x86_64 -M q35,usb=on,acpi=on,hpet=off -m 4G -smp cores=4 -accel kvm \
    -drive file=/home/y/kvm/ikuai.qcow2,if=virtio \
    -drive file=/home/y/Downloads/iKuai8_x64_4.0.0_Build202512241218.iso,index=1,media=cdrom \
    -device usb-tablet -device VGA,vgamem_mb=256 -monitor stdio \
    -nic tap,ifname=tap0,script=no,downscript=no,model=e1000,mac=52:54:00:11:11:11 \
    -nic user,model=e1000,mac=52:54:00:22:22:22 \
    -nic user,model=e1000,mac=52:54:00:33:33:33 \
    -nic user,model=e1000,mac=52:54:00:44:44:44




lsof -ti:19000,19001,19021,19222 | xargs kill -9 2>/dev/null

go run *.go  -r 1 -c  /home/yh/workspace/ikuai-bypass/config_example.yml -login http://10.1.1.1,admin,123



go run *.go  -r 1 -m ipgroup -c  /home/yh/workspace/ikuai-bypass/config_example.yml -login http://10.1.1.1,admin,123


go run *.go  -r 1 -m ispdomain -c  /home/yh/workspace/ikuai-bypass/config_example.yml -login http://10.1.1.1,admin,123


go run *.go  -r clean -c  /home/yh/workspace/ikuai-bypass/config_example.yml  -login http://10.1.1.1,admin,123


 git tag -a v4.4.1-pre -m "v4.4.1-pre

   主要更新：
   1. 完成爱快 v4 内测版适配 (#103)
      - 统一使用 v4 API，移除 ikuai_api3 支持
      - IPv4/IPv6 分组统一使用 route_object API
      - 规则标识从备注改为名字前缀 IKB（v4 接口备注无返回）
      - 因ikuaiv4不支持同名强制使用 Safe-Before 更新模式
      - 支持按 IP 分组名称自动搜索来源 IP（src-addr-opt-ipgroup）
      - 删除配置项中的 name等混乱名字 统一为tag

   2. 新增 iip 混合模式 (#104)
      - 支持 ispgroup、ipv4group、ipv6group 三分流模块一起使用

   3. 其他优化：
      - 去掉并发处理，改为顺序执行，避免 API 失败
      - 重构日志系统，增加智能高亮、彩色输出和跨平台支持
      - 因ikuaiv4 移除 delOldRule 参数
      - 删除 exportDomainSteamToTxt 功能
   "
