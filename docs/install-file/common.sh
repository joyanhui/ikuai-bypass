# ──────────────────────────────────────
# iKuai Bypass Install Script — Common Library
# ikuai-bypass 安装脚本 — 共享函数库
# ──────────────────────────────────────

# 默认路径
INSTALL_DIR="/opt/ikuai-bypass"
BIN_PATH="${INSTALL_DIR}/ikuai-bypass"
CONFIG_PATH="${INSTALL_DIR}/config.yml"
VERSION_FILE="${INSTALL_DIR}/.version"
SERVICE_NAME="ikuai-bypass"

# ── 架构检测 / Arch detection ──
detect_arch() {
    local umachine
    umachine="$(uname -m)"
    case "${umachine}" in
        x86_64|amd64)
            printf "x86_64"
            ;;
        i686|i386)
            printf "x86_32"
            ;;
        aarch64|arm64)
            printf "aarch64"
            ;;
        armv7*|armv7l)
            printf "arm7"
            ;;
        *)
            printf "unsupported:%s" "${umachine}"
            ;;
    esac
}

# ── OS 检测 / OS detection ──
detect_os() {
    if [ -f /etc/openwrt_release ]; then
        printf "openwrt"
    elif command -v nixos-version >/dev/null 2>&1 || [ -f /etc/nixos/configuration.nix ]; then
        printf "nixos"
    elif [ -f /etc/arch-release ] || command -v pacman >/dev/null 2>&1; then
        printf "arch"
    elif [ -f /etc/debian_version ] || command -v apt >/dev/null 2>&1; then
        printf "debian"
    else
        printf "unknown"
    fi
}

# ── 检查 root / Root check ──
check_root() {
    [ "$(id -u)" != "0" ] && {
        print_msg "ERR_ROOT"
        exit 1
    }
}

# ── 工具命令检测 / Ensure tool command ──
ensure_cmd() {
    local cmd="$1"
    local pkg="${2:-$1}"
    if ! command -v "${cmd}" >/dev/null 2>&1; then
        if command -v apt >/dev/null 2>&1; then
            apt-get update -qq && apt-get install -y -qq "${pkg}"
        elif command -v pacman >/dev/null 2>&1; then
            pacman -Sy --noconfirm "${pkg}"
        elif command -v opkg >/dev/null 2>&1; then
            opkg update >/dev/null 2>&1 && opkg install "${pkg}"
        fi
        if ! command -v "${cmd}" >/dev/null 2>&1; then
            print_msg "ERR_CMD" "${cmd}"
            exit 1
        fi
    fi
}

# ── 进程检查 / Check process ──
check_process() {
    if command -v pidof >/dev/null 2>&1; then
        pidof "ikuai-bypass" >/dev/null 2>&1
    elif command -v pgrep >/dev/null 2>&1; then
        pgrep "ikuai-bypass" >/dev/null 2>&1
    else
        ps 2>/dev/null | grep -v grep | grep "ikuai-bypass" >/dev/null 2>&1
    fi
}

# ── 双语消息 / Bilingual messages ──
# 使用: print_msg "KEY" [arg1] [arg2] ...
print_msg() {
    local key="$1"
    shift
    case "${LANG_CHOICE}" in
        1)
            case "${key}" in
                ERR_ROOT)         printf "Please run as root!\n" ;;
                ERR_ARCH)         printf "Unsupported architecture: %s\n" "$@" ;;
                ERR_OS)           printf "Unsupported OS\n" ;;
                ERR_CMD)          printf "Failed to install required command: %s\n" "$@" ;;
                ERR_INVALID)      printf "Invalid choice\n" ;;
                MSG_INSTALLING)   printf "Installing iKuai Bypass...\n" ;;
                MSG_INSTALL_OK)   printf "Installation completed\n" ;;
                MSG_UPDATE_DONE)  printf "Update completed\n" ;;
                MSG_STARTING)     printf "Starting service...\n" ;;
                MSG_ALREADY_RUN)  printf "Service is already running\n" ;;
                MSG_START_OK)     printf "Service started successfully\n" ;;
                MSG_START_FAIL)   printf "Failed to start service!\n" ;;
                MSG_STOPPING)     printf "Stopping service...\n" ;;
                MSG_STOPPED)      printf "Service stopped\n" ;;
                MSG_AUTO_ENABLE)  printf "Enabling auto-start...\n" ;;
                MSG_AUTO_DISABLE) printf "Disabling auto-start...\n" ;;
                MSG_AUTO_OK)      printf "Auto-start configured\n" ;;
                MSG_UNINSTALL)    printf "Uninstalling...\n" ;;
                MSG_UNINSTALL_DONE) printf "Uninstall completed\n" ;;
                MSG_UNINSTALL_CONF) printf "Remove configuration file (config.yml)? [y/N]: " ;;
                MSG_KEEP_CONF)    printf "Configuration file preserved\n" ;;
                MSG_RM_ALL)       printf "All files removed\n" ;;
                MSG_NIXOS)        printf "NixOS is not yet supported by this install script.\nPlease refer to the documentation for manual setup.\n" ;;
                MSG_VERSION_INPUT) printf "Enter version (leave empty for latest): " ;;
                MSG_FETCHING)     printf "Fetching latest version from GitHub API...\n" ;;
                MSG_LATEST_VER)   printf "Latest stable version: %s\n" "$@" ;;
                MSG_DOWNLOADING)  printf "Downloading %s ...\n" "$@" ;;
                MSG_DOWNLOAD_FAIL) printf "Download failed!\nURL: %s\n" "$@" ;;
                MSG_EXTRACTING)   printf "Extracting...\n" ;;
                MSG_PROC_FOUND)   printf "Found running process, stopping...\n" ;;
                MSG_LOG_HINT)     printf "Press Ctrl+C to exit log view\n" ;;
                MSG_NO_LOG)       printf "No log file found\n" ;;
                MSG_ENTER_CHOICE) printf "Enter your choice [0-9]: " ;;
                *)                printf "%s\n" "${key}" ;;
            esac
            ;;
        2)
            case "${key}" in
                ERR_ROOT)         printf "请以 root 用户运行此脚本！\n" ;;
                ERR_ARCH)         printf "不支持的架构：%s\n" "$@" ;;
                ERR_OS)           printf "不支持的操作系统\n" ;;
                ERR_CMD)          printf "无法安装必需命令：%s\n" "$@" ;;
                ERR_INVALID)      printf "输入错误\n" ;;
                MSG_INSTALLING)   printf "正在安装 iKuai Bypass...\n" ;;
                MSG_INSTALL_OK)   printf "安装完成\n" ;;
                MSG_UPDATE_DONE)  printf "更新完成\n" ;;
                MSG_STARTING)     printf "正在启动服务...\n" ;;
                MSG_ALREADY_RUN)  printf "服务已在运行中\n" ;;
                MSG_START_OK)     printf "服务启动成功\n" ;;
                MSG_START_FAIL)   printf "服务启动失败！\n" ;;
                MSG_STOPPING)     printf "正在停止服务...\n" ;;
                MSG_STOPPED)      printf "服务已停止\n" ;;
                MSG_AUTO_ENABLE)  printf "正在设置开机启动...\n" ;;
                MSG_AUTO_DISABLE) printf "正在关闭开机启动...\n" ;;
                MSG_AUTO_OK)      printf "开机启动设置完成\n" ;;
                MSG_UNINSTALL)    printf "正在卸载...\n" ;;
                MSG_UNINSTALL_DONE) printf "卸载完成\n" ;;
                MSG_UNINSTALL_CONF) printf "是否删除配置文件 (config.yml)？[y/N]：" ;;
                MSG_KEEP_CONF)    printf "配置文件已保留\n" ;;
                MSG_RM_ALL)       printf "所有文件已删除\n" ;;
                MSG_NIXOS)        printf "NixOS 暂不支持此安装脚本。\n请参考文档进行手动部署。\n" ;;
                MSG_VERSION_INPUT) printf "输入版本号（留空自动获取最新版）：" ;;
                MSG_FETCHING)     printf "正在从 GitHub API 获取最新版本...\n" ;;
                MSG_LATEST_VER)   printf "最新稳定版本：%s\n" "$@" ;;
                MSG_DOWNLOADING)  printf "正在下载 %s ...\n" "$@" ;;
                MSG_DOWNLOAD_FAIL) printf "下载失败！\nURL：%s\n" "$@" ;;
                MSG_EXTRACTING)   printf "正在解压...\n" ;;
                MSG_PROC_FOUND)   printf "发现正在运行的进程，正在停止...\n" ;;
                MSG_LOG_HINT)     printf "按 Ctrl+C 退出日志查看\n" ;;
                MSG_NO_LOG)       printf "未找到日志文件\n" ;;
                MSG_ENTER_CHOICE) printf "请输入选项 [0-9]：" ;;
                *)                printf "%s\n" "${key}" ;;
            esac
            ;;
    esac
}

# ── 菜单显示 / Menu display ──
print_menu() {
    clear
    printf "═══════════════════════════════════════\n"
    if [ "${LANG_CHOICE}" = "1" ]; then
        printf "   iKuai Bypass — Install & Manage\n"
    else
        printf "   iKuai Bypass — 安装管理脚本\n"
    fi
    printf "═══════════════════════════════════════\n"
    printf "1.  %s\n" "$(menu_str INSTALL)"
    printf "2.  %s\n" "$(menu_str UPDATE)"
    printf "3.  %s\n" "$(menu_str START)"
    printf "4.  %s\n" "$(menu_str STOP)"
    printf "5.  %s\n" "$(menu_str RESTART)"
    printf "6.  %s\n" "$(menu_str AUTO_ENABLE)"
    printf "7.  %s\n" "$(menu_str AUTO_DISABLE)"
    printf "8.  %s\n" "$(menu_str STATUS)"
    printf "9.  %s\n" "$(menu_str LOG)"
    printf "0.  %s\n" "$(menu_str EXIT)"
    printf "───────────────────────────────────────\n"
}

menu_str() {
    local key="$1"
    if [ "${LANG_CHOICE}" = "1" ]; then
        case "${key}" in
            INSTALL)      printf "Install" ;;
            UPDATE)       printf "Update" ;;
            START)        printf "Start" ;;
            STOP)         printf "Stop" ;;
            RESTART)      printf "Restart" ;;
            AUTO_ENABLE)  printf "Enable auto-start" ;;
            AUTO_DISABLE) printf "Disable auto-start" ;;
            STATUS)       printf "Status / Log" ;;
            LOG)          printf "View real-time log" ;;
            EXIT)         printf "Exit" ;;
        esac
    else
        case "${key}" in
            INSTALL)      printf "安装" ;;
            UPDATE)       printf "更新" ;;
            START)        printf "启动" ;;
            STOP)         printf "停止" ;;
            RESTART)      printf "重启" ;;
            AUTO_ENABLE)  printf "设置开机启动" ;;
            AUTO_DISABLE) printf "关闭开机启动" ;;
            STATUS)       printf "查看运行状态" ;;
            LOG)          printf "查看实时日志" ;;
            EXIT)         printf "退出" ;;
        esac
    fi
}

# ── 获取最新版本（无 pre-release）/ Get latest stable version ──
get_latest_version() {
    local api_url="https://api.github.com/repos/joyanhui/ikuai-bypass/releases/latest"
    local version=""

    if command -v curl >/dev/null 2>&1; then
        version="$(curl -fsSL "${api_url}" 2>/dev/null | \
            grep '"tag_name"' | head -1 | sed 's/.*"tag_name": "\(.*\)",.*/\1/')"
    elif command -v wget >/dev/null 2>&1; then
        version="$(wget -qO- "${api_url}" 2>/dev/null | \
            grep '"tag_name"' | head -1 | sed 's/.*"tag_name": "\(.*\)",.*/\1/')"
    fi

    # tag is like "ikuai-bypass-v0.1.0", extract version part
    case "${version}" in
        ikuai-bypass-v*)  printf "%s" "${version#ikuai-bypass-v}" ;;
        v*)               printf "%s" "${version#v}" ;;
        "")               printf "" ;;
        *)                printf "%s" "${version}" ;;
    esac
}

# ── 下载并安装 / Download and install ──
install_app() {
    local version="$1"
    local arch
    local os_type
    local url
    local zip_file

    arch="$(detect_arch)"
    os_type="$(detect_os)"

    if [ -z "${version}" ]; then
        print_msg "MSG_FETCHING"
        version="$(get_latest_version)"
        if [ -z "${version}" ]; then
            print_msg "MSG_DOWNLOAD_FAIL" "Failed to detect latest version"
            return 1
        fi
        print_msg "MSG_LATEST_VER" "${version}"
    fi

    # 如果已安装则询问是否覆盖
    if [ -f "${BIN_PATH}" ]; then
        print_msg "MSG_PROC_FOUND"
        stop_service
    fi

    url="https://github.com/joyanhui/ikuai-bypass/releases/download/ikuai-bypass-v${version}/ikuai-bypass-cli-linux-${arch}.zip"
    zip_file="/tmp/ikuai-bypass-${arch}.zip"

    print_msg "MSG_DOWNLOADING" "ikuai-bypass-cli-linux-${arch}.zip"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -o "${zip_file}" "${url}" || {
            print_msg "MSG_DOWNLOAD_FAIL" "${url}"
            rm -f "${zip_file}"
            return 1
        }
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "${zip_file}" "${url}" || {
            print_msg "MSG_DOWNLOAD_FAIL" "${url}"
            rm -f "${zip_file}"
            return 1
        }
    else
        print_msg "ERR_CMD" "curl/wget"
        return 1
    fi

    print_msg "MSG_EXTRACTING"
    mkdir -p "${INSTALL_DIR}"

    if command -v unzip >/dev/null 2>&1; then
        unzip -qo "${zip_file}" -d "${INSTALL_DIR}"
    else
        if command -v python3 >/dev/null 2>&1; then
            python3 -c "import zipfile,sys; zipfile.ZipFile(sys.argv[1]).extractall(sys.argv[2])" \
                "${zip_file}" "${INSTALL_DIR}"
        elif command -v python >/dev/null 2>&1; then
            python -c "import zipfile,sys; zipfile.ZipFile(sys.argv[1]).extractall(sys.argv[2])" \
                "${zip_file}" "${INSTALL_DIR}"
        else
            print_msg "ERR_CMD" "unzip"
            rm -f "${zip_file}"
            return 1
        fi
    fi
    rm -f "${zip_file}"

    chmod +x "${BIN_PATH}"

    # 如果 config.yml 不存在，从 sample 复制
    if [ ! -f "${CONFIG_PATH}" ]; then
        cp "${INSTALL_DIR}/config.yml" "${CONFIG_PATH}" 2>/dev/null || true
    fi

    # 写入版本
    printf "%s" "${version}" > "${VERSION_FILE}"

    print_msg "MSG_INSTALL_OK"
    return 0
}

# ── 启动服务 / Start service ──
start_service() {
    local os_type
    os_type="$(detect_os)"

    if check_process; then
        print_msg "MSG_ALREADY_RUN"
        return
    fi

    print_msg "MSG_STARTING"
    case "${os_type}" in
        debian|arch)
            systemctl start "${SERVICE_NAME}" 2>/dev/null || {
                print_msg "MSG_START_FAIL"
                return 1
            }
            ;;
        openwrt)
            "${BIN_PATH}" -r cronAft -c "${CONFIG_PATH}" > /dev/null 2>&1 &
            sleep 2
            ;;
    esac

    sleep 1
    if check_process; then
        print_msg "MSG_START_OK"
    else
        print_msg "MSG_START_FAIL"
    fi
}

# ── 停止服务 / Stop service ──
stop_service() {
    local os_type
    os_type="$(detect_os)"

    print_msg "MSG_STOPPING"
    case "${os_type}" in
        debian|arch)
            systemctl stop "${SERVICE_NAME}" 2>/dev/null || true
            ;;
        openwrt)
            local pid
            if command -v pidof >/dev/null 2>&1; then
                pid="$(pidof ikuai-bypass 2>/dev/null || true)"
            else
                pid="$(pgrep ikuai-bypass 2>/dev/null || true)"
            fi
            if [ -n "${pid}" ]; then
                kill -TERM "${pid}" 2>/dev/null || true
            fi
            local i=0
            while [ $i -lt 5 ]; do
                if ! check_process; then
                    break
                fi
                sleep 1
                i=$((i + 1))
            done
            if check_process; then
                if command -v pidof >/dev/null 2>&1; then
                    pid="$(pidof ikuai-bypass 2>/dev/null || true)"
                else
                    pid="$(pgrep ikuai-bypass 2>/dev/null || true)"
                fi
                [ -n "${pid}" ] && kill -KILL "${pid}" 2>/dev/null || true
                sleep 1
            fi
            ;;
    esac
    print_msg "MSG_STOPPED"
}

# ── 重启服务 / Restart service ──
restart_service() {
    stop_service
    start_service
}

# ── 设置开机启动 / Enable auto-start ──
enable_autostart() {
    local os_type
    os_type="$(detect_os)"

    print_msg "MSG_AUTO_ENABLE"
    case "${os_type}" in
        debian|arch)
            systemctl enable "${SERVICE_NAME}" 2>/dev/null || true
            systemctl daemon-reload 2>/dev/null || true
            ;;
        openwrt)
            if [ -f /etc/rc.common ]; then
                /etc/init.d/${SERVICE_NAME} enable 2>/dev/null || true
            fi
            ;;
    esac
    print_msg "MSG_AUTO_OK"
}

# ── 关闭开机启动 / Disable auto-start ──
disable_autostart() {
    local os_type
    os_type="$(detect_os)"

    print_msg "MSG_AUTO_DISABLE"
    case "${os_type}" in
        debian|arch)
            systemctl disable "${SERVICE_NAME}" 2>/dev/null || true
            ;;
        openwrt)
            if [ -f /etc/rc.common ]; then
                /etc/init.d/${SERVICE_NAME} disable 2>/dev/null || true
            fi
            ;;
    esac
    print_msg "MSG_AUTO_OK"
}

# ── 查看状态 / Show status ──
status_service() {
    local os_type
    os_type="$(detect_os)"

    printf "═══════════════════════════════════════\n"
    case "${os_type}" in
        debian|arch)
            systemctl status "${SERVICE_NAME}" 2>/dev/null || printf "Service not found\n"
            ;;
        openwrt)
            if check_process; then
                printf "PID: "
                ps 2>/dev/null | grep -v grep | grep "ikuai-bypass" | awk '{print $1}'
            else
                printf "ikuai-bypass: Not running\n"
            fi
            ;;
    esac
    if [ -f "${VERSION_FILE}" ]; then
        printf "Version: %s\n" "$(cat "${VERSION_FILE}")"
    fi
    printf "Config:  %s\n" "${CONFIG_PATH}"
}

# ── 查看日志 / View logs ──
view_log() {
    local os_type
    os_type="$(detect_os)"

    case "${os_type}" in
        debian|arch)
            print_msg "MSG_LOG_HINT"
            journalctl -u "${SERVICE_NAME}" -f -n 50 2>/dev/null || printf "No logs available\n"
            ;;
        openwrt)
            print_msg "MSG_LOG_HINT"
            if [ -f /tmp/ikuai-bypass.log ]; then
                tail -f /tmp/ikuai-bypass.log 2>/dev/null
            else
                print_msg "MSG_NO_LOG"
            fi
            ;;
    esac
}

# ── 卸载 / Uninstall ──
uninstall_app() {
    stop_service
    disable_autostart

    print_msg "MSG_UNINSTALL"

    # 删除服务文件
    local os_type
    os_type="$(detect_os)"
    case "${os_type}" in
        debian|arch)
            rm -f "/etc/systemd/system/${SERVICE_NAME}.service"
            systemctl daemon-reload 2>/dev/null || true
            ;;
        openwrt)
            rm -f "/etc/init.d/${SERVICE_NAME}"
            ;;
    esac

    # 删除二进制、版本文件和 README
    rm -f "${BIN_PATH}"
    rm -f "${VERSION_FILE}"
    rm -f "${INSTALL_DIR}/README.md"

    # 询问是否删除配置
    print_msg "MSG_UNINSTALL_CONF"
    read < /dev/tty rm_conf
    case "${rm_conf}" in
        y|Y|yes|YES)
            rm -f "${CONFIG_PATH}"
            print_msg "MSG_RM_ALL"
            ;;
        *)
            print_msg "MSG_KEEP_CONF"
            ;;
    esac

    # 如果目录为空则删除
    rmdir "${INSTALL_DIR}" 2>/dev/null || true

    print_msg "MSG_UNINSTALL_DONE"
}

# ── 安装服务文件 / Install service file ──
install_service_file() {
    local src_dir="${1:-$(dirname "$0")}"
    local os_type
    os_type="$(detect_os)"

    case "${os_type}" in
        debian|arch)
            mkdir -p /etc/systemd/system
            cp "${src_dir}/install-file/ikuai-bypass.service" "/etc/systemd/system/${SERVICE_NAME}.service"
            systemctl daemon-reload 2>/dev/null || true
            ;;
        openwrt)
            cp "${src_dir}/install-file/ikuai-bypass.initd" "/etc/init.d/${SERVICE_NAME}"
            chmod +x "/etc/init.d/${SERVICE_NAME}"
            ;;
    esac
}
