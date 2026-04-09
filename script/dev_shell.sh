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

# https://github.com/joyanhui/ikuai-bypass/blob/main/.github/ikuai.qcow2.7z
# -drive file=/home/y/Downloads/iiKuai8_x64_4.0.110_beta_Build202603051848.iso,index=1,media=cdrom \

qemu-system-x86_64 -M q35,usb=on,acpi=on,hpet=off -m 4G -smp cores=4 -accel kvm \
  -drive file=/home/y/kvm/ikuai.qcow2,if=virtio \
  -device usb-tablet -device VGA,vgamem_mb=256 -monitor stdio \
  -nic tap,ifname=tap0,script=no,downscript=no,model=e1000,mac=98:6A:5E:44:59:11 \
  -nic user,model=e1000,mac=A4:99:14:44:14:22 \
  -nic user,model=e1000,mac=50:00:F8:6A:F0:33 \
  -nic user,model=e1000,mac=B0:48:32:04:AC:44

lsof -ti:19000,19001,19021,19222 | xargs kill -9 2>/dev/null
