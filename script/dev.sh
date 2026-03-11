#!/usr/bin/env bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "${SCRIPT_DIR}")"

prefix_output() {
    local tag="$1"
    while IFS= read -r line; do
        echo "[${tag}] ${line}"
    done
}

cleanup() {
    jobs -p | xargs -r kill 2>/dev/null || true
    exit 0
}

trap cleanup SIGINT SIGTERM

cli_dev() {
    cd "${PROJECT_ROOT}"
    go run ./apps/cli 2>&1 | prefix_output "CLI" &
    wait
}

gui_dev() {
    cd "${PROJECT_ROOT}"
    go run ./apps/gui 2>&1 | prefix_output "GUI" &
    wait
}

print_usage() {
    echo "Usage: bash script/dev.sh <command>"
    echo "Commands:"
    echo "  cli:dev"
    echo "  gui:dev"
}

case "${1:-}" in
    cli:dev)
        cli_dev
        ;;
    gui:dev)
        gui_dev
        ;;
    -h|--help|help)
        print_usage
        ;;
    *)
        print_usage
        exit 1
        ;;
esac
