#!/bin/sh
# =============================================================================
# iKuai Bypass — 一键安装脚本 / One-click install script
# 仓库 / Repo: https://github.com/joyanhui/ikuai-bypass
# 说明 / Usage: sh install.sh
# =============================================================================

set -e

# ──────────────────────────────────────
# 语言选择 / Language selection
# ──────────────────────────────────────
LANG_CHOICE=""
# clear
printf "Please select your language / 请选择语言:\n"
printf "1. English\n"
printf "2. 中文\n"
printf "[1-2]: "
read < /dev/tty LANG_CHOICE

case "${LANG_CHOICE}" in
    1|2) ;;
    *)
        printf "Invalid choice / 无效选择\n"
        exit 1
        ;;
esac

# ──────────────────────────────────────
# 加载函数库 / Load library
# ──────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=install-file/common.sh
. "${SCRIPT_DIR}/install-file/common.sh"

# ──────────────────────────────────────
# 前置检查 / Pre-checks
# ──────────────────────────────────────
check_root

OS_TYPE="$(detect_os)"
case "${OS_TYPE}" in
    openwrt|debian|arch) ;;
    nixos)
        print_msg "MSG_NIXOS"
        exit 1
        ;;
    *)
        print_msg "ERR_OS"
        exit 1
        ;;
esac

ARCH="$(detect_arch)"
case "${ARCH}" in
    unsupported:*)
        print_msg "ERR_ARCH" "${ARCH#unsupported:}"
        exit 1
        ;;
esac

# ──────────────────────────────────────
# 函数：安装 / Install
# ──────────────────────────────────────
menu_install() {
    ensure_cmd curl || ensure_cmd wget
    ensure_cmd unzip

    print_msg "MSG_VERSION_INPUT"
    read < /dev/tty input_version

    if install_app "${input_version}"; then
        install_service_file
        start_service
    fi
}

# ──────────────────────────────────────
# 函数：更新 / Update
# ──────────────────────────────────────
menu_update() {
    ensure_cmd curl || ensure_cmd wget
    ensure_cmd unzip

    local current_ver=""
    [ -f "${VERSION_FILE}" ] && current_ver="$(cat "${VERSION_FILE}")"

    if [ -n "${current_ver}" ]; then
        if [ "${LANG_CHOICE}" = "1" ]; then
            printf "Current version: %s\n" "${current_ver}"
        else
            printf "当前版本：%s\n" "${current_ver}"
        fi
    fi

    print_msg "MSG_VERSION_INPUT"
    read < /dev/tty input_version
    [ -z "${input_version}" ] && input_version="${current_ver}"

    if install_app "${input_version}"; then
        restart_service
    fi
}

# ──────────────────────────────────────
# 菜单循环 / Main menu loop
# ──────────────────────────────────────
while :; do
    print_menu
    print_msg "MSG_ENTER_CHOICE"
    read < /dev/tty menu_choice

    case "${menu_choice}" in
        1) menu_install ;;
        2) menu_update ;;
        3) start_service ;;
        4) stop_service ;;
        5) restart_service ;;
        6) enable_autostart ;;
        7) disable_autostart ;;
        8) status_service ;;
        9) view_log ;;
        0)
            clear
            exit 0
            ;;
        *)
            print_msg "ERR_INVALID"
            ;;
    esac

    if [ "${menu_choice}" != "0" ]; then
        printf "\n"
        if [ "${LANG_CHOICE}" = "1" ]; then
            printf "Press Enter to continue..."
        else
            printf "按 Enter 键继续..."
        fi
        read < /dev/tty dummy
    fi
done
