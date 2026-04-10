#!/usr/bin/env sh

set -eu

CONFIG_DIR="/etc/ikuai-bypass"
CONFIG_PATH="${CONFIG_DIR}/config.yml"
TEMPLATE_PATH="/opt/ikuai-bypass/config.yml"
RUN_MODULE="${APP_RUN_MODE:-ispdomain}"

mkdir -p "${CONFIG_DIR}"

if [ ! -f "${CONFIG_PATH}" ]; then
  cp "${TEMPLATE_PATH}" "${CONFIG_PATH}"
fi

has_module_arg="0"

for arg in "$@"; do
  case "$arg" in
    -m|--m)
      has_module_arg="1"
      break
      ;;
  esac
done

if [ "${has_module_arg}" = "1" ]; then
  exec /usr/local/bin/ikuai-bypass -c "${CONFIG_PATH}" "$@"
fi

exec /usr/local/bin/ikuai-bypass -c "${CONFIG_PATH}" -m "${RUN_MODULE}" "$@"
