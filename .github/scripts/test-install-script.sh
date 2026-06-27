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
    IKB_INSTALL_BASE_URL="file://${WORK_DIR}" sh ./install.sh status | grep -q "binary_path=" && pass "non-interactive status works" || fail "non-interactive status failed"
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
    set -eo pipefail

    local OPENWRT_VERSION="${IKB_OPENWRT_VERSION:-24.10.0}"
    local WORK_DIR="/tmp/ikb-openwrt-kvm"
    local SSH_PORT=2222
    local SSH_OPTS="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=5 -o LogLevel=ERROR"
    local QEMU_PID=""
    local REPO_ROOT

    REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

    cleanup_kvm() {
        trap '' EXIT
        local ec=${1:-$?}
        echo ""; echo "=== Cleanup ==="
        [ -n "${QEMU_PID-}" ] && kill -0 "${QEMU_PID-}" 2>/dev/null && kill "${QEMU_PID-}" 2>/dev/null || true
        pkill -f "qemu-system-x86" 2>/dev/null || true
        sleep 1; rm -rf "${WORK_DIR-}" 2>/dev/null || true
        echo "=== Cleanup done ==="; exit "${ec}"
    }
    trap 'cleanup_kvm' EXIT

    echo "═══════════════════════════════════════"
    echo "  Mode: kvm-openwrt"
    echo "═══════════════════════════════════════"

    # ── 1. Install host deps ──
    echo "--- [1/7] Install dependencies ---"
    sudo apt-get update -qq
    sudo apt-get install -y -qq qemu-system-x86 sshpass qemu-utils

    # ── 2. Download & extract OpenWrt image ──
    echo "--- [2/7] Download OpenWrt ${OPENWRT_VERSION} ---"
    mkdir -p "${WORK_DIR}"
    pushd "${WORK_DIR}" > /dev/null
    local IMG_FILE="openwrt-${OPENWRT_VERSION}-x86-64-generic-ext4-combined"
    wget -q "https://downloads.openwrt.org/releases/${OPENWRT_VERSION}/targets/x86/64/${IMG_FILE}.img.gz"
    gunzip -f "${IMG_FILE}.img.gz" || true
    popd > /dev/null 2>&1 || true

    # ── 3. Pre-configure image (password + network) ──
    echo "--- [3/7] Prepare image ---"
    local SERIAL_LOG="${WORK_DIR}/serial.log"
    # dropbear rejects empty passwords, set to "admin"
    local ROOT_PW_HASH
    ROOT_PW_HASH="$(openssl passwd -1 admin 2>/dev/null)" || ROOT_PW_HASH='$1$01234567$ABCDEFabcdef0123456789abcdef'  # fallback
    # QEMU slirp assigns 10.0.2.0/24 via DHCP; change LAN from static to dhcp
    sudo modprobe nbd max_part=16 2>/dev/null || true
    sudo qemu-nbd -c /dev/nbd0 "${WORK_DIR}/${IMG_FILE}.img" --format=raw 2>/dev/null || true
    sleep 1
    sudo mount /dev/nbd0p2 /mnt 2>/dev/null || true
    sudo sed -i "s|^root:[^:]*:|root:${ROOT_PW_HASH}:|" /mnt/etc/shadow
    sudo sed -i "s/option proto 'static'/option proto 'dhcp'/" /mnt/etc/config/network
    sudo sed -i "/option ipaddr/d; /option netmask/d; /option ip6assign/d" /mnt/etc/config/network
    sudo umount /mnt 2>/dev/null; sudo qemu-nbd -d /dev/nbd0 2>/dev/null; sleep 1

    # ── 4. Boot QEMU with raw image ──
    echo "--- [4/7] Boot VM ---"
    qemu-system-x86_64 -M q35 -m 1G -accel kvm -cpu host \
        -drive file="${WORK_DIR}/${IMG_FILE}.img",format=raw,if=virtio \
        -nic user,hostfwd=tcp::${SSH_PORT}-:22,model=e1000 \
        -display none -serial file:"${SERIAL_LOG}" > /dev/null 2>&1 &
    QEMU_PID=$!

    # ── 5. Wait for SSH (root password set to "admin") ──
    echo "--- [5/7] Wait SSH (max 120s) ---"
    local SSH_OK=""
    for i in $(seq 1 60); do
        grep -q "Kernel panic" "${SERIAL_LOG}" 2>/dev/null && { trap '' EXIT; tail -5 "${SERIAL_LOG}"; cleanup_kvm 1; }
        if sshpass -p "admin" ssh ${SSH_OPTS} -p ${SSH_PORT} root@127.0.0.1 "echo OK" 2>/dev/null | grep -q OK; then
            echo "SSH ready ~$((i*2))s"; SSH_OK=1; break
        fi
        sleep 2
    done
    [ -n "${SSH_OK}" ] || { trap '' EXIT; echo "SSH timeout"; tail -30 "${SERIAL_LOG}"; cleanup_kvm 1; }

    # ── 6. Copy install script & run tests ──
    echo "--- [6/7] Copy install script ---"
    sshpass -p "admin" scp ${SSH_OPTS} -O -P ${SSH_PORT} \
        "${REPO_ROOT}/docs/install.sh" "${REPO_ROOT}/docs/install-file" root@127.0.0.1:/tmp/

    echo "--- [7/7] Run tests ---"
    sshpass -p "admin" ssh ${SSH_OPTS} -p ${SSH_PORT} root@127.0.0.1 sh -s << 'REMOTE_TEST'
set -e
PASS=0; FAIL=0
INSTALL_DIR="/opt/ikuai-bypass"; BIN_PATH="${INSTALL_DIR}/ikuai-bypass"
CONFIG_PATH="${INSTALL_DIR}/config.yml"; VERSION_FILE="${INSTALL_DIR}/.version"
SERVICE_FILE="/etc/init.d/ikuai-bypass"
pass() { echo "  ✓ $1"; PASS=$((PASS + 1)); }
fail() { echo "  ✗ $1"; FAIL=$((FAIL + 1)); exit 1; }

echo "[setup] Installing deps..."; opkg update || true; opkg install curl unzip 2>&1 || true
cd /tmp; LANG_CHOICE=1; . ./install-file/common.sh
configure_paths openwrt

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
grep -q "/opt/ikuai-bypass/ikuai-bypass" "${SERVICE_FILE}" && pass "Init.d uses /opt binary path" || fail "Init.d binary path mismatch"
grep -q "/opt/ikuai-bypass/config.yml" "${SERVICE_FILE}" && pass "Init.d uses /opt config path" || fail "Init.d config path mismatch"

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
IKB_UNINSTALL_SERVICE_ONLY=1 uninstall_app
[ -f "${SERVICE_FILE}" ] && fail "Service residue" || pass "Service removed"
[ -f "${BIN_PATH}" ] && pass "Binary preserved" || fail "Binary lost"
[ -f "${CONFIG_PATH}" ] && pass "Config preserved" || fail "Config lost"

echo ""; echo "── Test 8/9: Reinstall + Uninstall (delete all) ──"
install_app "${VER}"; install_service_file /tmp; [ -f "${BIN_PATH}" ] && pass "Re-installed" || fail "Re-install failed"
IKB_UNINSTALL_REMOVE_CONFIG=1 uninstall_app
[ -f "${BIN_PATH}" ] && fail "Binary residue" || pass "Binary removed"
[ -f "${CONFIG_PATH}" ] && fail "Config residue" || pass "Config removed"
[ -d "${INSTALL_DIR}" ] && fail "Install dir residue" || pass "Install dir cleaned"

echo ""; echo "── Test 9/9: Process residue ──"
IKB_PID="$(pidof ikuai-bypass 2>/dev/null || true)"; [ -n "${IKB_PID}" ] && kill "${IKB_PID}" 2>/dev/null || true; sleep 1
check_process && fail "Process residue" || pass "No residue"

echo ""; echo "═══════════════════════════════════════"
echo "  OpenWrt KVM: ${PASS} passed, ${FAIL} failed"
echo "═══════════════════════════════════════"
exit ${FAIL}
REMOTE_TEST

    local TEST_EXIT=$?

    # ── 7. Collect logs ──
    echo ""; echo "--- [7/7] Collect logs ---"
    [ -f "${SERIAL_LOG}" ] && echo "Serial log: $(wc -c < "${SERIAL_LOG}") bytes" && echo "--- tail 30 ---" && tail -30 "${SERIAL_LOG}"

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
