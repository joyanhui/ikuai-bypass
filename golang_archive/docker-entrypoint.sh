#!/bin/sh
set -eu

DEFAULT_CONFIG_PATH="/opt/ikuai-bypass/config.yml"
TARGET_CONFIG_PATH="${IKB_CONFIG_PATH:-/etc/ikuai-bypass/config.yml}"

mkdir -p "$(dirname "$TARGET_CONFIG_PATH")"
if [ ! -f "$TARGET_CONFIG_PATH" ]; then
    cp "$DEFAULT_CONFIG_PATH" "$TARGET_CONFIG_PATH"
fi

if [ "$#" -gt 0 ]; then
    first_arg="$1"
    case "$first_arg" in
        -*)
            set -- ikuai-bypass "$@"
            ;;
    esac
fi

if [ "$#" -eq 0 ]; then
    set -- ikuai-bypass -r cron
fi

if [ "${1:-}" = "ikuai-bypass" ]; then
    set -- "$@" -c "$TARGET_CONFIG_PATH"
fi

exec "$@"
