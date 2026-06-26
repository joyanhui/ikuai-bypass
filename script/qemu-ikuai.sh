#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "${SCRIPT_DIR}")"
QCOW2="${PROJECT_ROOT}/packaging/ikuai.qcow2"
ISO="/mnt_auto/autofs/DATA128G/Core_File/ISO/iKuai8_x64_4.0.302_Build202606161528.iso"

ISO_ARG=()
if [[ "${1:-}" == "--install" ]]; then
    ISO_ARG=(-drive file="${ISO}",index=1,media=cdrom)
fi

exec qemu-system-x86_64 \
    -M q35,usb=on,acpi=on,hpet=off \
    -m 4G -smp cores=4 -accel kvm \
    -drive file="${QCOW2}",if=virtio \
    "${ISO_ARG[@]}" \
    -device usb-tablet -device VGA,vgamem_mb=256 -monitor stdio \
    -nic user,model=e1000,mac=52:54:00:11:11:11,hostfwd=tcp::8080-:80 \
    -nic user,model=e1000,mac=52:54:00:22:22:22 \
    -nic user,model=e1000,mac=52:54:00:33:33:33 \
    -nic user,model=e1000,mac=52:54:00:44:44:44
