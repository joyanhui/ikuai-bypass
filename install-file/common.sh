# ──────────────────────────────────────
# iKuai Bypass Install Script — Common Library
# ikuai-bypass 安装脚本 — 共享函数库
# ──────────────────────────────────────

# 默认路径 / Default paths
SERVICE_NAME="ikuai-bypass"
INSTALL_DIR="/opt/ikuai-bypass"
BIN_PATH="${INSTALL_DIR}/ikuai-bypass"
CONFIG_PATH="${INSTALL_DIR}/config.yml"
VERSION_FILE="${INSTALL_DIR}/.version"
LOG_PATH="/dev/null"

# ── 根据系统配置路径 / Configure OS-specific paths ──
configure_paths() {
    INSTALL_DIR="/opt/ikuai-bypass"
    BIN_PATH="${INSTALL_DIR}/ikuai-bypass"
    CONFIG_PATH="${INSTALL_DIR}/config.yml"
    VERSION_FILE="${INSTALL_DIR}/.version"
    LOG_PATH="/dev/null"
}

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
        armv5*|arm926*)
            printf "arm5"
            ;;
        armv6*|arm1176*)
            printf "arm6"
            ;;
        armv7*|armv7l|armhf*)
            printf "arm7"
            ;;
        mips64el*)
            printf "mips64le"
            ;;
        mips64*)
            printf "mips64"
            ;;
        mipsel*|mipsle*)
            printf "mipsle"
            ;;
        mips*)
            printf "mips"
            ;;
        riscv64*)
            printf "riscv64gc"
            ;;
        ppc64le*|powerpc64le*)
            printf "ppc64le"
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
        ikb_error "root" "Please run as root" "install.sh must run as root (uid 0)"
        exit 1
    }
    return 0
}

# ── 工具命令检测 / Ensure tool command ──
ensure_cmd() {
    local cmd="$1"
    local pkg="${2:-$1}"
    if ! command -v "${cmd}" >/dev/null 2>&1; then
        if command -v apt >/dev/null 2>&1; then
            apt-get update -qq && apt-get install -y -qq "${pkg}" ca-certificates
        elif command -v pacman >/dev/null 2>&1; then
            pacman -Sy --noconfirm "${pkg}"
        elif command -v opkg >/dev/null 2>&1; then
            opkg update >/dev/null 2>&1 && opkg install "${pkg}" ca-bundle
        fi
        if ! command -v "${cmd}" >/dev/null 2>&1; then
            ikb_error "deps_missing" "Failed to install required command: ${cmd}" "package=${pkg}"
            exit 1
        fi
    fi
}

# ── 安装基础依赖 / Install required dependencies ──
ensure_base_deps() {
    ensure_cmd curl curl
    ensure_cmd unzip unzip
}

# ── 写文件到 overlay 前先落盘到 /tmp / Stage writes through /tmp for overlay safety ──
install_file_atomic() {
    local src="$1"
    local dst="$2"
    local mode="${3:-0644}"
    local tmp="/tmp/ikuai-bypass-install.$$.$(basename "${dst}")"
    mkdir -p "$(dirname "${dst}")" || return 1
    cp "${src}" "${tmp}" || return 1
    chmod "${mode}" "${tmp}" 2>/dev/null || true
    mv "${tmp}" "${dst}" || return 1
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

# ── 结构化错误输出 / Structured error output ──
# Why/为什么: 安装脚本的错误信息需要可被上层(helper/LuCI/前端)机器解析。
# 统一输出 key=value 行(status/error_code/message/error_detail), 与 LuCI parse_key_value_lines 兼容。
# English: emit machine-parsable key=value error lines (status/error_code/message/error_detail)
# compatible with LuCI parse_key_value_lines, so callers can report the real reason.
#
# 用法 / Usage:
#   ikb_error "download" "Download failed" "curl: (28) SSL timeout"
#   ikb_fail  "download" "Download failed" "curl: (28) SSL timeout"   # 返回 1
#   ikb_die   "root" "Please run as root"                              # 退出 1
#
# 错误码 / Error codes:
#   root              非 root 运行
#   os_unsupported    不支持的操作系统
#   arch_unsupported  不支持的架构
#   deps_missing      依赖命令缺失或安装失败
#   version_fetch     版本检测失败
#   download          下载失败(detail 含 curl/wget 错误)
#   extract           解压失败
#   archive_invalid   压缩包内容不符合预期
#   install_write     写盘/安装失败
#   service_start     服务启动失败
#   service_stop      服务停止失败
#   service_enable    开机自启设置失败
#   config_write      配置写入失败
ikb_error() {
    local code="$1"
    local msg="$2"
    printf 'status=error\n'
    printf 'error_code=%s\n' "${code}"
    printf 'message=%s\n' "${msg}"
    [ $# -gt 2 ] && printf 'error_detail=%s\n' "$3"
}

# 函数级失败: 输出结构化错误并返回非零 / Function-level failure: emit structured error, return non-zero
ikb_fail() {
    ikb_error "$@"
    return 1
}

# 脚本级致命错误: 输出结构化错误并退出 / Fatal: emit structured error, exit 1
ikb_die() {
    ikb_error "$@"
    exit 1
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
        version="$(curl -fsSL --connect-timeout 15 --max-time 30 -A "ikuai-bypass-install/1.0" "${api_url}" 2>/dev/null | \
            grep '"tag_name"' | head -1 | sed 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/')"
    elif command -v wget >/dev/null 2>&1; then
        version="$(wget -qO- --timeout=30 --header="User-Agent: ikuai-bypass-install/1.0" "${api_url}" 2>/dev/null | \
            grep '"tag_name"' | head -1 | sed 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/')"
    fi

    # tag is like "ikuai-bypass-v0.1.0", extract version part
    case "${version}" in
        ikuai-bypass-v*)  printf "%s" "${version#ikuai-bypass-v}" ;;
        v*)               printf "%s" "${version#v}" ;;
        "")               printf "" ;;
        *)                printf "%s" "${version}" ;;
    esac
}

# ── 获取最新预发行版本 / Get latest pre-release version ──
get_prerelease_version() {
    local api_url="https://api.github.com/repos/joyanhui/ikuai-bypass/releases?per_page=30"
    local version=""

    if command -v curl >/dev/null 2>&1; then
        version="$(curl -fsSL --connect-timeout 15 --max-time 30 -A "ikuai-bypass-install/1.0" "${api_url}" 2>/dev/null | \
            grep -E '"tag_name"|"prerelease"|"draft"' | paste - - - | \
            grep '"prerelease": true' | grep '"draft": false' | \
            head -1 | sed 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/')"
    elif command -v wget >/dev/null 2>&1; then
        version="$(wget -qO- --timeout=30 --header="User-Agent: ikuai-bypass-install/1.0" "${api_url}" 2>/dev/null | \
            grep -E '"tag_name"|"prerelease"|"draft"' | paste - - - | \
            grep '"prerelease": true' | grep '"draft": false' | \
            head -1 | sed 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/')"
    fi

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
        if [ "${IKB_PRERELEASE:-0}" = "1" ]; then
            version="$(get_prerelease_version)"
        else
            version="$(get_latest_version)"
        fi
        if [ -z "${version}" ]; then
            ikb_error "version_fetch" "Failed to detect latest version"
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
    tmp_dir="$(mktemp -d 2>/dev/null || mktemp -d -t ikb-install)"
    zip_file="${tmp_dir}/ikuai-bypass-${arch}.zip"
    unpack_dir="${tmp_dir}/unpack"

    print_msg "MSG_DOWNLOADING" "ikuai-bypass-cli-linux-${arch}.zip"
    if command -v curl >/dev/null 2>&1; then
        curl_err="$(mktemp 2>/dev/null || printf '/tmp/ikb-curl-err.%s' "$$")"
        curl -fsSL --connect-timeout 15 --max-time 300 -o "${zip_file}" "${url}" 2>"${curl_err}" || {
            local detail
            detail="$(head -1 "${curl_err}" 2>/dev/null || true)"
            rm -f "${curl_err}"
            ikb_error "download" "Download failed" "URL: ${url}${detail:+ | ${detail}}"
            rm -rf "${tmp_dir}"
            return 1
        }
        rm -f "${curl_err}"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "${zip_file}" "${url}" --timeout=300 2>/tmp/ikb-wget-err.$$ || {
            local detail
            detail="$(head -1 /tmp/ikb-wget-err.$$ 2>/dev/null || true)"
            rm -f /tmp/ikb-wget-err.$$
            ikb_error "download" "Download failed" "URL: ${url}${detail:+ | ${detail}}"
            rm -rf "${tmp_dir}"
            return 1
        }
        rm -f /tmp/ikb-wget-err.$$
    else
        ikb_error "deps_missing" "curl or wget is required"
        return 1
    fi

    print_msg "MSG_EXTRACTING"
    mkdir -p "${unpack_dir}" || {
        ikb_error "install_write" "Failed to create extract dir" "${unpack_dir}"
        rm -rf "${tmp_dir}"
        return 1
    }

    if command -v unzip >/dev/null 2>&1; then
        unzip -qo "${zip_file}" -d "${unpack_dir}" 2>/tmp/ikb-unzip-err.$$ || {
            local detail
            detail="$(head -1 /tmp/ikb-unzip-err.$$ 2>/dev/null || true)"
            rm -f /tmp/ikb-unzip-err.$$
            ikb_error "extract" "Failed to extract archive" "${detail:-unzip exit $?}"
            rm -rf "${tmp_dir}"
            return 1
        }
        rm -f /tmp/ikb-unzip-err.$$
    else
        if command -v python3 >/dev/null 2>&1; then
            python3 -c "import zipfile,sys; zipfile.ZipFile(sys.argv[1]).extractall(sys.argv[2])" \
                "${zip_file}" "${unpack_dir}" || {
                ikb_error "extract" "Failed to extract archive" "python3 zipfile failed"
                rm -rf "${tmp_dir}"
                return 1
            }
        elif command -v python >/dev/null 2>&1; then
            python -c "import zipfile,sys; zipfile.ZipFile(sys.argv[1]).extractall(sys.argv[2])" \
                "${zip_file}" "${unpack_dir}" || {
                ikb_error "extract" "Failed to extract archive" "python zipfile failed"
                rm -rf "${tmp_dir}"
                return 1
            }
        else
            ikb_error "deps_missing" "unzip is required" "install unzip (opkg install unzip)"
            rm -rf "${tmp_dir}"
            return 1
        fi
    fi

    if [ ! -f "${unpack_dir}/ikuai-bypass" ]; then
        ikb_error "archive_invalid" "Archive does not contain ikuai-bypass" "${zip_file}"
        rm -rf "${tmp_dir}"
        return 1
    fi

    install_file_atomic "${unpack_dir}/ikuai-bypass" "${BIN_PATH}" 0755 || {
        ikb_error "install_write" "Failed to install binary" "${BIN_PATH}"
        rm -rf "${tmp_dir}"
        return 1
    }

    chmod +x "${BIN_PATH}"

    # 如果 config.yml 不存在，从 sample 复制
    if [ ! -f "${CONFIG_PATH}" ]; then
        if [ -f "${unpack_dir}/config.yml" ]; then
            install_file_atomic "${unpack_dir}/config.yml" "${CONFIG_PATH}" 0644 || {
                ikb_error "config_write" "Failed to write config.yml" "${CONFIG_PATH}"
                rm -rf "${tmp_dir}"
                return 1
            }
        fi
    fi

    # 写入版本
    mkdir -p "$(dirname "${VERSION_FILE}")" || {
        ikb_error "install_write" "Failed to create version dir" "$(dirname "${VERSION_FILE}")"
        rm -rf "${tmp_dir}"
        return 1
    }
    printf "%s" "${version}" > "/tmp/ikuai-bypass-version.$$" || {
        ikb_error "install_write" "Failed to write version file" "/tmp/ikuai-bypass-version.$$"
        rm -rf "${tmp_dir}"
        return 1
    }
    mv "/tmp/ikuai-bypass-version.$$" "${VERSION_FILE}" || {
        ikb_error "install_write" "Failed to move version file" "${VERSION_FILE}"
        rm -rf "${tmp_dir}"
        return 1
    }
    rm -rf "${tmp_dir}"

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
            systemctl start "${SERVICE_NAME}" 2>/tmp/ikb-systemctl-err.$$ || {
                local detail
                detail="$(head -1 /tmp/ikb-systemctl-err.$$ 2>/dev/null || true)"
                rm -f /tmp/ikb-systemctl-err.$$
                ikb_error "service_start" "Failed to start service" "systemctl: ${detail:-unknown error}"
                return 1
            }
            rm -f /tmp/ikb-systemctl-err.$$
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
        ikb_error "service_start" "Failed to start service" "process not running (check binary/config)"
        print_msg "MSG_START_FAIL"
        return 1
    fi
}

# ── 停止服务 / Stop service ──
stop_service() {
    local os_type
    os_type="$(detect_os)"

    print_msg "MSG_STOPPING"
    case "${os_type}" in
        debian|arch)
            # 仅当服务实际存在且激活时才报错; 服务不存在/已停止时 systemctl 返回非0属正常
            systemctl stop "${SERVICE_NAME}" 2>/dev/null || {
                systemctl is-active "${SERVICE_NAME}" >/dev/null 2>&1 && ikb_error "service_stop" "Failed to stop service" "systemctl stop ${SERVICE_NAME}"
            }
            ;;
        openwrt)
            local pid
            if command -v pidof >/dev/null 2>&1; then
                pid="$(pidof ikuai-bypass 2>/dev/null || true)"
            else
                pid="$(pgrep ikuai-bypass 2>/dev/null || true)"
            fi
            if [ -n "${pid}" ]; then
                kill -TERM "${pid}" 2>/dev/null || ikb_error "service_stop" "Failed to signal process" "kill -TERM ${pid}"
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
                [ -n "${pid}" ] && kill -KILL "${pid}" 2>/dev/null || ikb_error "service_stop" "Failed to force-kill process" "kill -KILL ${pid}"
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
            systemctl enable "${SERVICE_NAME}" 2>/dev/null || {
                systemctl is-enabled "${SERVICE_NAME}" >/dev/null 2>&1 || ikb_error "service_enable" "Failed to enable auto-start" "systemctl enable ${SERVICE_NAME}"
            }
            systemctl daemon-reload 2>/dev/null || true
            ;;
        openwrt)
            # 仅当 init.d 脚本存在时才执行 enable; 服务已卸载时静默(避免误报)
            if [ -f /etc/rc.common ] && [ -f "/etc/init.d/${SERVICE_NAME}" ]; then
                /etc/init.d/${SERVICE_NAME} enable 2>/dev/null || ikb_error "service_enable" "Failed to enable auto-start" "/etc/init.d/${SERVICE_NAME} enable"
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
            systemctl disable "${SERVICE_NAME}" 2>/dev/null || {
                systemctl is-enabled "${SERVICE_NAME}" >/dev/null 2>&1 && ikb_error "service_enable" "Failed to disable auto-start" "systemctl disable ${SERVICE_NAME}"
            }
            ;;
        openwrt)
            # 仅当 init.d 脚本存在时才执行 disable; 服务已卸载时静默(避免误报)
            if [ -f /etc/rc.common ] && [ -f "/etc/init.d/${SERVICE_NAME}" ]; then
                /etc/init.d/${SERVICE_NAME} disable 2>/dev/null || ikb_error "service_enable" "Failed to disable auto-start" "/etc/init.d/${SERVICE_NAME} disable"
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
            if [ "${LOG_PATH}" != "/dev/null" ] && [ -f "${LOG_PATH}" ]; then
                tail -f "${LOG_PATH}" 2>/dev/null
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

    if [ "${IKB_UNINSTALL_SERVICE_ONLY:-0}" = "1" ]; then
        print_msg "MSG_KEEP_CONF"
        print_msg "MSG_UNINSTALL_DONE"
        return 0
    fi

    if [ "${IKB_UNINSTALL_REMOVE_CONFIG:-0}" = "1" ]; then
        rm -rf "${INSTALL_DIR}"
        print_msg "MSG_RM_ALL"
    elif [ "${IKB_NONINTERACTIVE:-0}" = "1" ]; then
        rm -f "${BIN_PATH}"
        rm -f "${VERSION_FILE}"
        rm -f "${INSTALL_DIR}/README.md"
        print_msg "MSG_KEEP_CONF"
    else
        # 询问是否删除配置
        print_msg "MSG_UNINSTALL_CONF"
        read < /dev/tty rm_conf
        case "${rm_conf}" in
            y|Y|yes|YES)
                rm -rf "${INSTALL_DIR}"
                print_msg "MSG_RM_ALL"
                ;;
            *)
                rm -f "${BIN_PATH}"
                rm -f "${VERSION_FILE}"
                rm -f "${INSTALL_DIR}/README.md"
                print_msg "MSG_KEEP_CONF"
                ;;
        esac
    fi

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
            if [ -f "${src_dir}/install-file/ikuai-bypass.service" ]; then
                install_file_atomic "${src_dir}/install-file/ikuai-bypass.service" "/etc/systemd/system/${SERVICE_NAME}.service" 0644
            else
                write_systemd_service "/etc/systemd/system/${SERVICE_NAME}.service"
            fi
            systemctl daemon-reload 2>/dev/null || true
            ;;
        openwrt)
            if [ -f "${src_dir}/install-file/ikuai-bypass.initd" ]; then
                install_file_atomic "${src_dir}/install-file/ikuai-bypass.initd" "/etc/init.d/${SERVICE_NAME}" 0755
            else
                write_openwrt_init "/etc/init.d/${SERVICE_NAME}"
            fi
            ;;
    esac
}

write_openwrt_init() {
    local dst="$1"
    local tmp="/tmp/ikuai-bypass-init.$$"
    cat > "${tmp}" <<EOF
#!/bin/sh /etc/rc.common

START=99
STOP=10

start() {
    ${BIN_PATH} -c ${CONFIG_PATH} > /dev/null 2>&1 &
}

stop() {
    killall -q -9 ikuai-bypass 2>/dev/null
}

restart() {
    stop
    sleep 1
    start
}
EOF
    install_file_atomic "${tmp}" "${dst}" 0755
    rm -f "${tmp}"
}

write_systemd_service() {
    local dst="$1"
    local tmp="/tmp/ikuai-bypass-service.$$"
    cat > "${tmp}" <<EOF
[Unit]
Description=iKuai Bypass CLI Service
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=${BIN_PATH} -c ${CONFIG_PATH}
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF
    install_file_atomic "${tmp}" "${dst}" 0644
    rm -f "${tmp}"
}

print_status_kv() {
    printf 'binary_path=%s\n' "${BIN_PATH}"
    if [ -x "${BIN_PATH}" ]; then
        printf 'binary_exists=1\n'
        printf 'binary_version=%s\n' "$("${BIN_PATH}" --version 2>/dev/null | sed -n '1p' || true)"
    else
        printf 'binary_exists=0\n'
        printf 'binary_version=\n'
    fi
    printf 'config_path=%s\n' "${CONFIG_PATH}"
    [ -f "${CONFIG_PATH}" ] && printf 'config_exists=1\n' || printf 'config_exists=0\n'
    if [ -f "/etc/init.d/${SERVICE_NAME}" ] || [ -f "/etc/systemd/system/${SERVICE_NAME}.service" ]; then
        printf 'service_installed=1\n'
    else
        printf 'service_installed=0\n'
    fi
    check_process && printf 'running=1\n' || printf 'running=0\n'
    [ -f "${VERSION_FILE}" ] && printf 'version=%s\n' "$(cat "${VERSION_FILE}")" || printf 'version=\n'
    printf 'arch=%s\n' "$(detect_arch 2>/dev/null || true)"
    printf 'mode=%s\n' "$(sed -n '/^mode: /{s/^mode: *//p;q}' "${CONFIG_PATH}" 2>/dev/null)"
    printf 'run_mode=%s\n' "$(grep '^run-mode:' "${CONFIG_PATH}" 2>/dev/null | head -1 | sed 's/^run-mode: *//')"
}

print_latest_kv() {
    local latest=""
    local current=""
    latest="$(get_latest_version)"
    [ -f "${VERSION_FILE}" ] && current="$(cat "${VERSION_FILE}")"
    printf 'latest_version=%s\n' "${latest}"
    printf 'current_version=%s\n' "${current}"
    if [ -n "${latest}" ] && [ "${latest}" != "${current}" ]; then
        printf 'update_available=1\n'
    else
        printf 'update_available=0\n'
    fi
}

print_prerelease_kv() {
    local latest=""
    local current=""
    latest="$(get_prerelease_version)"
    [ -f "${VERSION_FILE}" ] && current="$(cat "${VERSION_FILE}")"
    printf 'latest_version=%s\n' "${latest}"
    printf 'current_version=%s\n' "${current}"
    if [ -n "${latest}" ] && [ "${latest}" != "${current}" ]; then
        printf 'update_available=1\n'
    else
        printf 'update_available=0\n'
    fi
}

set_config_field() {
    local key="$1" val="$2" file="${CONFIG_PATH}"
    [ -n "${val}" ] || return 0
    if [ -f "${file}" ]; then
        if grep -q "^${key}:" "${file}" 2>/dev/null; then
            sed -i "s|^${key}:.*|${key}: ${val}|" "${file}"
        else
            sed -i "/^AddWait:/a\\${key}: ${val}" "${file}"
        fi
    fi
}

run_action() {
    local action="${1:-}"
    local version="${2:-}"
    IKB_NONINTERACTIVE=1
    export IKB_NONINTERACTIVE
    case "${action}" in
        install)
            ensure_base_deps
            if install_app "${version}"; then
                set_config_field "mode" "${IKB_MODE:-}"
                set_config_field "run-mode" "${IKB_RUN_MODE:-}"
                install_service_file; enable_autostart; start_service
            else
                # Why/为什么: install_app 失败时必须返回非0, 供上层(helper/LuCI)判断真实结果。
                # 否则空 if 分支会让整个 install 伪装成功, 上层误报"服务配置成功"。
                # English: return non-zero when install_app fails so the caller (helper/LuCI)
                # can report the real result instead of faking success.
                return 1
            fi
            ;;
        update)
            ensure_base_deps
            if install_app "${version}"; then
                set_config_field "mode" "${IKB_MODE:-}"
                set_config_field "run-mode" "${IKB_RUN_MODE:-}"
                install_service_file; restart_service
            else
                return 1
            fi
            ;;
        uninstall)
            case "${2:-}" in
                --remove-config|--full) IKB_UNINSTALL_REMOVE_CONFIG=1; export IKB_UNINSTALL_REMOVE_CONFIG ;;
                --service-only) IKB_UNINSTALL_SERVICE_ONLY=1; export IKB_UNINSTALL_SERVICE_ONLY ;;
            esac
            uninstall_app
            ;;
        start) start_service ;;
        stop) stop_service ;;
        restart) restart_service ;;
        enable) enable_autostart ;;
        disable) disable_autostart ;;
        status|inspect) print_status_kv ;;
        latest)
            if [ "${IKB_PRERELEASE:-0}" = "1" ]; then
                print_prerelease_kv
            else
                print_latest_kv
            fi
            ;;
        log)
            if [ "${LOG_PATH}" != "/dev/null" ] && [ -f "${LOG_PATH}" ]; then tail -n 80 "${LOG_PATH}"; else print_msg "MSG_NO_LOG"; fi
            ;;
        *)
            printf 'Unsupported action: %s\n' "${action}"
            return 2
            ;;
    esac
}
