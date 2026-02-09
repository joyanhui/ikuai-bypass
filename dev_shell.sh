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


 git tag -a v4.4.2-Pre -m "v4.4.2-Pre

   ### 主要更新(内测版本)：
   > 因为爱快v4也在内侧中，爱快的api变动较多，未来可能也有变化，此版本仅为尝鲜体验
   #### 1. 完成爱快 v4 内测版适配 (#103)
      - [重要] 统一使用 v4 API，移除 ikuai_api3 支持,v4.4以后版本不再支持爱快v3.x版本，如果你是爱快v3.7以下版本请使用ikuai-bypass v4.1
      - [重要] 规则标识从备注改为名字前缀 IKB（v4 接口备注无返回）,不再使用备注作为规则区分，备注仅仅只有备注功能了没有实际功能意义。
      - [重要] 删除配置项中的 name等混乱名字 统一为tag。你需要参考 [config.yml](https://github.com/joyanhui/ikuai-bypass/blob/main/config.yml) 更新一下你的配置文件

   #### 2. 新增 iip 混合模式 (#104)
      - [新增]支持 ispgroup、ipv4group、ipv6group 三分流模块一起使用

   #### 3. 其他优化：
      - 去掉并发处理，改为顺序执行，避免 API 失败和日志乱序
      - 重构日志系统，增加智能高亮、彩色输出和跨平台支持
      - [重要] 因ikuaiv4的api限制了同名规则，所以 移除 delOldRule 参数，改为统一逻辑为先同步后查询然后删除最后添加
      - 删除 exportDomainSteamToTxt 功能
      - [重要] 清理模式下不再默认为CleanAll 避免误删除, 必须显示配置tag参数。同时清理模式只可以清理名字为IKB开头的规则。为了兼容旧版也支持清理 备注中包含字符IKUAI_BYPASSS和joyanhui/ikuai-bypass的规则。不可以清理名字不包含IKB备注不包含前述字符的规则和分组。
   "
