
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
    -drive file=/home/y/Downloads/iKuai8_x64_4.0.101_beta_Build202602111835.iso,index=1,media=cdrom \
    -device usb-tablet -device VGA,vgamem_mb=256 -monitor stdio \
    -nic tap,ifname=tap0,script=no,downscript=no,model=e1000,mac=52:54:00:11:11:11 \
    -nic user,model=e1000,mac=52:54:00:22:22:22 \
    -nic user,model=e1000,mac=52:54:00:33:33:33 \
    -nic user,model=e1000,mac=52:54:00:44:44:44



qemu-system-x86_64 -M q35,usb=on,acpi=on,hpet=off -m 4G -smp cores=4 -accel kvm \
    -drive file=/home/y/kvm/ikuai.qcow2,if=virtio \
    -drive file=/home/y/Downloads/iKuai8_x64_4.0.101_beta_Build202602111835.iso,index=1,media=cdrom \
    -device usb-tablet -device VGA,vgamem_mb=256 -monitor stdio \
    -nic tap,ifname=tap0,script=no,downscript=no,model=e1000,mac=52:54:00:11:11:11 \
    -nic user,model=e1000,mac=52:54:00:22:22:22 \
    -nic user,model=e1000,mac=52:54:00:33:33:33 \
    -nic user,model=e1000,mac=52:54:00:44:44:44



lsof -ti:19000,19001,19021,19222 | xargs kill -9 2>/dev/null

go run *.go  -r 1 -c  /home/yh/workspace/ikuai-bypass/config_example.yml -login http://10.1.1.1,admin,123



git tag -a v4.4.7 -F Update.md
