#!/usr/bin/env bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "${SCRIPT_DIR}")"

cleanup() {
    jobs -p | xargs -r kill 2>/dev/null || true
    exit 0
}

trap cleanup SIGINT SIGTERM

cli_dev() {
    cd "${PROJECT_ROOT}"
    cargo run --bin ikb-cli -- "$@"
}


frontend_dev() {
    cd "${PROJECT_ROOT}/app/frontend"
    bun run dev
}

frontend_build() {
    cd "${PROJECT_ROOT}/app/frontend"
    bun run build
}

gui_dev() {
    if [[ "${1:-}" == "--" ]]; then
        shift
    fi

    cd "${PROJECT_ROOT}/app/frontend"
    local log_file
    log_file="${PROJECT_ROOT}/app/frontend/.astro-dev.log"
    : > "${log_file}"
    bun run dev >"${log_file}" 2>&1 &
    local fe_pid=$!

    local port=""
    for i in $(seq 1 80); do
        if ! kill -0 "${fe_pid}" 2>/dev/null; then
            wait "${fe_pid}" || true
            echo "frontend dev server failed to start"
            exit 1
        fi
        if [[ -f "${log_file}" ]]; then
            port=$(grep -Eo 'http://localhost:[0-9]+' "${log_file}" | head -n 1 | awk -F: '{print $3}')
            if [[ -n "${port}" ]]; then
                break
            fi
        fi
        sleep 0.1
    done

    if [[ -z "${port}" ]]; then
        echo "failed to detect astro dev server port"
        cat "${log_file}" || true
        kill "${fe_pid}" 2>/dev/null || true
        exit 1
    fi

    cd "${PROJECT_ROOT}"
    bunx tauri dev --config app/src-tauri/tauri.conf.json --config "{\"build\":{\"beforeDevCommand\":\"\",\"devUrl\":\"http://localhost:${port}\"}}" "$@"

    kill "${fe_pid}" 2>/dev/null || true
}

print_usage() {
    echo "Usage: bash script/dev.sh <command> [args...]"
    echo "Commands:"
    echo "  cli:dev [-- <ikb-cli args...>]     运行 CLI（本体，完整功能）"
    echo "  gui:dev                           运行 GUI(App)（封装/复用 CLI 完整能力）"
    echo ""
    echo "  （可选附带能力）"
    echo "  webui:dev                         启动 Astro dev server（仅前端调试）"
    echo "  webui:build                       构建 Astro dist（供 CLI/Tauri 加载）"
}

case "${1:-}" in
    cli:dev)
        shift
        if [[ "${1:-}" == "--" ]]; then
            shift
        fi
        cli_dev "$@"
        ;;
    webui:dev)
        shift
        frontend_dev "$@"
        ;;
    webui:build)
        shift
        frontend_build "$@"
        ;;
    gui:dev)
        shift
        gui_dev "$@"
        ;;
    -h|--help|help)
        print_usage
        ;;
    *)
        print_usage
        exit 1
        ;;
esac
