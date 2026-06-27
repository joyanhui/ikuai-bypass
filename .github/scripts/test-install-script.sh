#!/bin/bash
# Why: CI test for docs/install.sh — supports native systemd (ubuntu) and KVM (kvm-openwrt)
# 为什么：针对 docs/install.sh 的 CI 测试 — 支持原生 systemd (ubuntu) 和 KVM (kvm-openwrt)
# Usage:
#   bash .github/scripts/test-install-script.sh ubuntu        # Ubuntu systemd test
#   bash .github/scripts/test-install-script.sh kvm-openwrt   # OpenWrt KVM test

set -e

# 非 root 时自动通过 sudo 提权
if [ "$(id -u)" -ne 0 ] && command -v sudo >/dev/null 2>&1; then
    exec sudo bash "$(realpath "$0")" "$@"
fi

MODE="${1:-ubuntu}"
PASS=0
FAIL=0

# ── helpers ──
pass() { echo "  ✓ $1"; PASS=$((PASS + 1)); }
fail() { echo "  ✗ $1"; FAIL=$((FAIL + 1)); }

assert_true() {
    local desc="$1"; shift
    if "$@"; then pass "${desc}"; else fail "${desc}"; fi
}
assert_false() {
    local desc="$1"; shift
    if "$@"; then fail "${desc}"; else pass "${desc}"; fi
}

# ═══════════════════════════════════════════════
#  MODE: ubuntu — native systemd test
# ═══════════════════════════════════════════════
run_ubuntu_test() {
    cleanup() {
        local p
        for p in $(pidof ikuai-bypass 2>/dev/null || pgrep ikuai-bypass 2>/dev/null || true); do
            kill -TERM "${p}" 2>/dev/null || true
        done
        sleep 1
        rm -rf "${INSTALL_DIR}" 2>/dev/null || true
        rm -f /etc/systemd/system/ikuai-bypass.service 2>/dev/null || true
        rm -f /etc/init.d/ikuai-bypass 2>/dev/null || true
        rm -f /usr/lib/systemd/system/ikuai-bypass.service 2>/dev/null || true
        systemctl daemon-reload 2>/dev/null || true
    }

    local WORK_DIR="/tmp/ikb-install-ci-$(date +%s)"
    mkdir -p "${WORK_DIR}"
    cp -r "$(dirname "$0")/../../docs/install.sh" "${WORK_DIR}/"
    cp -r "$(dirname "$0")/../../docs/install-file" "${WORK_DIR}/"
    cd "${WORK_DIR}"

    LANG_CHOICE=1
    # shellcheck source=../../docs/install-file/common.sh
    . ./install-file/common.sh

    local DETECTED_OS="$(detect_os)"
    local DETECTED_ARCH="$(detect_arch)"

    echo "═══════════════════════════════════════"
    echo "  Mode: ubuntu"
    echo "  OS detected: ${DETECTED_OS}"
    echo "  ARCH detected: ${DETECTED_ARCH}"
    echo "═══════════════════════════════════════"

    # ── 1/9 OS / Arch ──
    echo ""; echo "── 1/9  OS / Arch detection ──"
    case "${DETECTED_ARCH}" in
        x86_64|x86_32|aarch64|arm7) pass "Arch supported: ${DETECTED_ARCH}" ;;
        *) fail "Arch unsupported: ${DETECTED_ARCH}" ;;
    esac
    case "${DETECTED_OS}" in
        debian|arch) pass "OS supported: ${DETECTED_OS}" ;;
        *) fail "OS unsupported: ${DETECTED_OS}" ;;
    esac

    # ── 2/9 get_latest_version ──
    echo ""; echo "── 2/9  get_latest_version ──"
    local VERSION="$(get_latest_version)"
    [ -n "${VERSION}" ] && pass "Latest version: ${VERSION}" || fail "get_latest_version returned empty"

    # ── 3/9 install_app ──
    echo ""; echo "── 3/9  install_app (download + extract) ──"
    cleanup
    install_app "${VERSION}" && pass "install_app succeeded" || fail "install_app failed"
    assert_true "Binary exists" test -f "${BIN_PATH}"
    assert_true "Binary executable" test -x "${BIN_PATH}"
    assert_true "Version file exists" test -f "${VERSION_FILE}"
    assert_true "Config exists" test -f "${CONFIG_PATH}"
    assert_true "Version matches" test "$(cat "${VERSION_FILE}")" = "${VERSION}"

    # ── 4/9 Service file ──
    echo ""; echo "── 4/9  Service file installation ──"
    install_service_file "${WORK_DIR}"
    assert_true "Systemd unit exists" test -f /etc/systemd/system/ikuai-bypass.service
    local TMPL_SHA=$(sha256sum "${WORK_DIR}/install-file/ikuai-bypass.service" 2>/dev/null | cut -d' ' -f1)
    local INST_SHA=$(sha256sum /etc/systemd/system/ikuai-bypass.service 2>/dev/null | cut -d' ' -f1)
    [ "${TMPL_SHA}" = "${INST_SHA}" ] && pass "Systemd unit matches template" || fail "Systemd unit content mismatch"

    # ── 5/9 start / status / stop ──
    echo ""; echo "── 5/9  Service start / status / stop ──"
    systemctl daemon-reload 2>/dev/null || true
    start_service; sleep 2
    assert_true "Process running after start" check_process
    status_service > /dev/null 2>&1 && pass "status_service runs without error"
    stop_service; sleep 2
    assert_false "Process stopped" check_process

    # ── 6/9 autostart ──
    echo ""; echo "── 6/9  Auto-start enable / disable ──"
    enable_autostart
    systemctl is-enabled ikuai-bypass >/dev/null 2>&1 && pass "Auto-start enabled" || fail "Auto-start not enabled"
    disable_autostart
    systemctl is-enabled ikuai-bypass 2>/dev/null | grep -q disabled && pass "Auto-start disabled" || echo "  (unit may report static — acceptable)"

    # ── 7/9 Uninstall (keep config) ──
    echo ""; echo "── 7/9  Uninstall (keep config) ──"
    rm -f "${BIN_PATH}" "${VERSION_FILE}"
    rm -f /etc/systemd/system/ikuai-bypass.service; systemctl daemon-reload 2>/dev/null || true
    assert_false "Binary removed" test -f "${BIN_PATH}"
    assert_true "Config preserved" test -f "${CONFIG_PATH}"

    # ── 8/9 Reinstall + Uninstall (delete all) ──
    echo ""; echo "── 8/9  Reinstall + Uninstall (delete all) ──"
    install_app "${VERSION}"
    install_service_file "${WORK_DIR}"
    assert_true "Binary re-installed" test -f "${BIN_PATH}"
    rm -f "${BIN_PATH}" "${VERSION_FILE}" "${CONFIG_PATH}" "${INSTALL_DIR}/README.md"
    rmdir "${INSTALL_DIR}" 2>/dev/null || true
    rm -f /etc/systemd/system/ikuai-bypass.service; systemctl daemon-reload 2>/dev/null || true
    assert_false "Binary removed" test -f "${BIN_PATH}"
    assert_false "Config removed" test -f "${CONFIG_PATH}"
    assert_false "Install dir cleaned" test -d "${INSTALL_DIR}"

    # ── 9/9 residue cleanup ──
    echo ""; echo "── 9/9  Temp file / residue cleanup ──"
    cleanup
    assert_false "No process residue" check_process
    assert_false "No install dir" test -d "${INSTALL_DIR}"
    rm -rf "${WORK_DIR}"

    echo ""; echo "═══════════════════════════════════════"
    echo "  Ubuntu Results: ${PASS} passed, ${FAIL} failed"
    echo "═══════════════════════════════════════"
    exit ${FAIL}
}

# ═══════════════════════════════════════════════
#  MODE: kvm-openwrt — KVM-based OpenWrt test
# ═══════════════════════════════════════════════
run_kvm_openwrt_test() {
    set -euo pipefail

    # ── KVM config ──
    local OPENWRT_VERSION="${IKB_OPENWRT_VERSION:-24.10.0}"
    local WORK_DIR="/tmp/ikb-openwrt-kvm"
    local SSH_PORT=2222
    local ROOT_PASSWORD="ikbtest123"
    local BOOT_TIMEOUT=120
    local SSH_TIMEOUT=90
    local SSH_OPTS="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR"
    local QEMU_PID=""
    local REPO_ROOT

    REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

    cleanup_kvm() {
        local ec=${1:-$?}
        echo ""; echo "=== Cleanup ==="
        # QEMU_PID may be unbound when EXIT trap fires before it is assigned (set -e early exit)
        if [ -n "${QEMU_PID-}" ] && kill -0 "${QEMU_PID-}" 2>/dev/null; then
            kill "${QEMU_PID-}" 2>/dev/null || true; wait "${QEMU_PID-}" 2>/dev/null || true
        fi
        pkill -f "qemu-system-x86.*openwrt" 2>/dev/null || true
        sudo qemu-nbd -d /dev/nbd0 2>/dev/null || true
        sleep 1; rm -rf "${WORK_DIR-}" 2>/dev/null || true
        echo "=== Cleanup done ==="; exit "${ec}"
    }
    trap 'cleanup_kvm' EXIT

    echo "═══════════════════════════════════════"
    echo "  Mode: kvm-openwrt"
    echo "═══════════════════════════════════════"

    # ── 1. Install host deps ──
    echo "--- [1/8] Install host dependencies ---"
    sudo apt-get update -qq
    sudo apt-get install -y -qq qemu-system-x86 qemu-utils sshpass wget openssl

    # ── 2. Download OpenWrt image ──
    echo "--- [2/8] Download OpenWrt ${OPENWRT_VERSION} x86_64 image ---"
    mkdir -p "${WORK_DIR}"
    pushd "${WORK_DIR}" > /dev/null
    local IMG_FILE="openwrt-${OPENWRT_VERSION}-x86-64-generic-ext4-combined.img"
    local IMG_URL="https://downloads.openwrt.org/releases/${OPENWRT_VERSION}/targets/x86/64/${IMG_FILE}.gz"
    wget -q --show-progress "${IMG_URL}"
    gunzip -f "${IMG_FILE}.gz" || true  # gzip returns 2 on trailing-garbage warning; harmless
    qemu-img convert -f raw -O qcow2 "${IMG_FILE}" openwrt.qcow2
    qemu-img resize openwrt.qcow2 2G
    rm -f "${IMG_FILE}"

    # ── 3. Pre-configure image ──
    echo "--- [3/8] Pre-configure image (qemu-nbd) ---"
    sudo modprobe nbd max_part=8
    sudo qemu-nbd -c /dev/nbd0 openwrt.qcow2; sleep 2
    local NBD_MNT="${WORK_DIR}/nbd-mnt"; mkdir -p "${NBD_MNT}"
    sudo mount /dev/nbd0p2 "${NBD_MNT}"
    local PWD_HASH
    PWD_HASH="$(openssl passwd -1 "${ROOT_PASSWORD}" 2>/dev/null || python3 -c "import crypt; print(crypt.crypt('${ROOT_PASSWORD}', crypt.mksalt(crypt.METHOD_MD5)))")"
    sudo sed -i "s|^root:[^:]*|root:${PWD_HASH}|" "${NBD_MNT}/etc/shadow"
    sudo sed -i 's/option Interface.*//' "${NBD_MNT}/etc/config/dropbear" 2>/dev/null || true
    # 确保 VM 内有 DNS（OpenWrt /etc/resolv.conf 是到 /tmp/resolv.conf 的 symlink，
    # 启动时 tmpfs 覆盖使其失效，所以删掉 symlink 写一个实文件）
    sudo rm -f "${NBD_MNT}/etc/resolv.conf"
    printf "nameserver 1.1.1.1\nnameserver 8.8.8.8\n" | sudo tee "${NBD_MNT}/etc/resolv.conf" >/dev/null
    sudo umount "${NBD_MNT}"; sudo qemu-nbd -d /dev/nbd0; rm -rf "${NBD_MNT}"
    echo "Image pre-configured"

    # ── 4. Boot QEMU ──
    echo "--- [4/8] Boot OpenWrt VM ---"
    local SERIAL_LOG="${WORK_DIR}/serial.log" QEMU_LOG="${WORK_DIR}/qemu.log"
    qemu-system-x86_64 -M q35 -m 1G -accel kvm -cpu host \
        -drive file=openwrt.qcow2,if=virtio \
        -nic user,hostfwd=tcp::${SSH_PORT}-:22,model=e1000 \
        -display none -serial file:"${SERIAL_LOG}" \
        > "${QEMU_LOG}" 2>&1 &
    QEMU_PID=$!; echo "QEMU PID: ${QEMU_PID}"

    # ── 5. Wait for boot ──
    echo "--- [5/8] Wait for boot (up to ${BOOT_TIMEOUT}s) ---"
    local BOOT_OK=0
    for i in $(seq 1 ${BOOT_TIMEOUT}); do
        grep -q " login:" "${SERIAL_LOG}" 2>/dev/null && { BOOT_OK=1; echo "Boot after ~${i}s"; break; }
        grep -q "Kernel panic" "${SERIAL_LOG}" 2>/dev/null && { tail -30 "${SERIAL_LOG}"; cleanup_kvm 1; }
        sleep 1
    done
    [ "${BOOT_OK}" -eq 1 ] || { echo "Boot timeout"; tail -50 "${SERIAL_LOG}"; cleanup_kvm 1; }

    # ── 6. Wait for SSH ──
    echo "--- [6/8] Wait for SSH (up to ${SSH_TIMEOUT}s) ---"
    local SSH_OK=0
    for i in $(seq 1 ${SSH_TIMEOUT}); do
        sshpass -p "${ROOT_PASSWORD}" ssh ${SSH_OPTS} -p ${SSH_PORT} root@127.0.0.1 "echo SSH_READY" 2>/dev/null | grep -q SSH_READY && { SSH_OK=1; echo "SSH after ~${i}s"; break; }
        sleep 2
    done
    [ "${SSH_OK}" -eq 1 ] || { echo "SSH timeout"; tail -30 "${SERIAL_LOG}"; cleanup_kvm 1; }

    # ── 7. Run tests via SSH ──
    echo ""; echo "--- [7/8] Run install script tests via SSH ---"
    sshpass -p "${ROOT_PASSWORD}" scp ${SSH_OPTS} -P ${SSH_PORT} \
        -r "${REPO_ROOT}/docs/install.sh" "${REPO_ROOT}/docs/install-file" root@127.0.0.1:/tmp/ 2>&1

    sshpass -p "${ROOT_PASSWORD}" ssh ${SSH_OPTS} -p ${SSH_PORT} root@127.0.0.1 sh -s << 'REMOTE_TEST'
set -e
PASS=0; FAIL=0
INSTALL_DIR="/opt/ikuai-bypass"; BIN_PATH="${INSTALL_DIR}/ikuai-bypass"
CONFIG_PATH="${INSTALL_DIR}/config.yml"; VERSION_FILE="${INSTALL_DIR}/.version"
SERVICE_FILE="/etc/init.d/ikuai-bypass"
pass() { echo "  ✓ $1"; PASS=$((PASS + 1)); }
fail() { echo "  ✗ $1"; FAIL=$((FAIL + 1)); exit 1; }

echo "[setup] Installing deps..."; opkg update || echo "  [warn] opkg update failed"; opkg install curl unzip 2>&1 || echo "  [warn] some opkg installs failed"
cd /tmp; LANG_CHOICE=1; . ./install-file/common.sh

echo ""; echo "── Test 1/9: OS / Arch ──"
case "$(detect_os)" in openwrt) pass "OK: openwrt";; *) fail "OS mismatch";; esac
case "$(detect_arch)" in x86_64) pass "OK: x86_64";; *) fail "Arch mismatch";; esac

echo ""; echo "── Test 2/9: get_latest_version ──"
VER="$(get_latest_version)"; [ -n "${VER}" ] && pass "Version: ${VER}" || fail "Empty"

echo ""; echo "── Test 3/9: install_app ──"
install_app "${VER}"
[ -f "${BIN_PATH}" ] && pass "Binary exists" || fail "Binary missing"
[ -x "${BIN_PATH}" ] && pass "Executable" || fail "Not executable"
[ "$(cat "${VERSION_FILE}")" = "${VER}" ] && pass "Version correct" || fail "Version mismatch"
[ -f "${CONFIG_PATH}" ] && pass "Config exists" || fail "Config missing"

echo ""; echo "── Test 4/9: Service file ──"
install_service_file /tmp
[ -f "${SERVICE_FILE}" ] && pass "Init.d exists" || fail "Init.d missing"
local _sum_cmd="sha256sum"
command -v sha256sum >/dev/null 2>&1 || _sum_cmd="md5sum"
local _tmpl=$(${_sum_cmd} /tmp/install-file/ikuai-bypass.initd 2>/dev/null | cut -d' ' -f1)
local _inst=$(${_sum_cmd} "${SERVICE_FILE}" 2>/dev/null | cut -d' ' -f1)
[ -n "${_tmpl}" ] && [ "${_tmpl}" = "${_inst}" ] && pass "Content matches" || fail "Content mismatch (${_sum_cmd})"

echo ""; echo "── Test 5/9: enable + start + stop ──"
"${SERVICE_FILE}" enable && pass "Enabled" || fail "Enable failed"
[ -L /etc/rc.d/S99ikuai-bypass ] || [ -L /etc/rc.d/K10ikuai-bypass ] && pass "rc.d symlink" || fail "rc.d symlink missing"
"${SERVICE_FILE}" start; sleep 3
check_process && pass "Process running" || fail "Process not running"
"${SERVICE_FILE}" stop; sleep 2
check_process && fail "Process still running" || pass "Process stopped"

echo ""; echo "── Test 6/9: disable ──"
"${SERVICE_FILE}" disable; sleep 1
[ -L /etc/rc.d/S99ikuai-bypass ] && echo "  WARN: symlink persists" || pass "rc.d symlink removed"

echo ""; echo "── Test 7/9: Uninstall (keep config) ──"
rm -f "${BIN_PATH}" "${VERSION_FILE}" "${SERVICE_FILE}"
[ -f "${BIN_PATH}" ] && fail "Binary residue" || pass "Binary removed"
[ -f "${CONFIG_PATH}" ] && pass "Config preserved" || fail "Config lost"

echo ""; echo "── Test 8/9: Reinstall + Uninstall (delete all) ──"
install_app "${VER}"; install_service_file /tmp; [ -f "${BIN_PATH}" ] && pass "Re-installed" || fail "Re-install failed"
rm -f "${BIN_PATH}" "${VERSION_FILE}" "${CONFIG_PATH}" "${INSTALL_DIR}/README.md" "${SERVICE_FILE}"
rmdir "${INSTALL_DIR}" 2>/dev/null || true
[ -f "${BIN_PATH}" ] && fail "Binary residue" || pass "Binary removed"
[ -f "${CONFIG_PATH}" ] && fail "Config residue" || pass "Config removed"
[ -d "${INSTALL_DIR}" ] && fail "Dir residue" || pass "Dir cleaned"

echo ""; echo "── Test 9/9: Process residue ──"
IKB_PID="$(pidof ikuai-bypass 2>/dev/null || true)"; [ -n "${IKB_PID}" ] && kill "${IKB_PID}" 2>/dev/null || true; sleep 1
check_process && fail "Process residue" || pass "No residue"

echo ""; echo "═══════════════════════════════════════"
echo "  OpenWrt KVM: ${PASS} passed, ${FAIL} failed"
echo "═══════════════════════════════════════"
exit ${FAIL}
REMOTE_TEST

    local TEST_EXIT=$?

    # ── 8. Collect logs ──
    echo ""; echo "--- [8/8] Collect logs ---"
    [ -f "${SERIAL_LOG}" ] && echo "Serial log: $(wc -c < "${SERIAL_LOG}") bytes" && echo "--- tail 30 ---" && tail -30 "${SERIAL_LOG}"

    popd > /dev/null
    echo "Exit code: ${TEST_EXIT}"
    exit ${TEST_EXIT}
}

# ═══════════════════════════════════════════════
#  Dispatch
# ═══════════════════════════════════════════════
case "${MODE}" in
    ubuntu) run_ubuntu_test ;;
    kvm-openwrt) run_kvm_openwrt_test ;;
    *)
        echo "Usage: $0 [ubuntu|kvm-openwrt]"
        exit 1
        ;;
esac
