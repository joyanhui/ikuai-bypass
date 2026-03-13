
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
    -drive file=/home/y/Downloads/iiKuai8_x64_4.0.110_beta_Build202603051848.iso,index=1,media=cdrom \
    -device usb-tablet -device VGA,vgamem_mb=256 -monitor stdio \
    -nic tap,ifname=tap0,script=no,downscript=no,model=e1000,mac=52:54:00:11:11:11 \
    -nic user,model=e1000,mac=52:54:00:22:22:22 \
    -nic user,model=e1000,mac=52:54:00:33:33:33 \
    -nic user,model=e1000,mac=52:54:00:44:44:44



qemu-system-x86_64 -M q35,usb=on,acpi=on,hpet=off -m 4G -smp cores=4 -accel kvm \
    -drive file=/home/y/kvm/ikuai.qcow2,if=virtio \
    -drive file=/home/y/Downloads/iKuai8_x64_4.0.110_beta_Build202603051848.iso,index=1,media=cdrom \
    -device usb-tablet -device VGA,vgamem_mb=256 -monitor stdio \
    -nic tap,ifname=tap0,script=no,downscript=no,model=e1000,mac=52:54:00:11:11:11 \
    -nic user,model=e1000,mac=52:54:00:22:22:22 \
    -nic user,model=e1000,mac=52:54:00:33:33:33 \
    -nic user,model=e1000,mac=52:54:00:44:44:44



lsof -ti:19000,19001,19021,19222 | xargs kill -9 2>/dev/null




cargo release 4.4.100-alpha1 --execute --no-publish --no-push


git push origin main
git push origin fileuni-v0.0.1-alpha5
git push --tags

cargo release 4.4.100-alpha3 --execute
