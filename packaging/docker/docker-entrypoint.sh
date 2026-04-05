#!/usr/bin/env sh

set -eu

CONFIG_DIR="/etc/ikuai-bypass"
CONFIG_PATH="${CONFIG_DIR}/config.yml"
TEMPLATE_PATH="/opt/ikuai-bypass/config.yml"

mkdir -p "${CONFIG_DIR}"

if [ ! -f "${CONFIG_PATH}" ]; then
  cp "${TEMPLATE_PATH}" "${CONFIG_PATH}"
fi

exec /usr/local/bin/ikuai-bypass -c "${CONFIG_PATH}" "$@"
