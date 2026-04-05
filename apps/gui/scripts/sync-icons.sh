#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GUI_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
REPO_ROOT="$(cd "${GUI_DIR}/../.." && pwd)"
SOURCE_ICON="${GUI_DIR}/icons/icon.png"
IPKG_ICON="${REPO_ROOT}/packaging/ikuai-ipkg/ikuai-bypass/ui/ico/app.png"
WEB_FAVICON="${REPO_ROOT}/frontends/app/public/favicon.png"
WEB_APPLE_TOUCH_ICON="${REPO_ROOT}/frontends/app/public/apple-touch-icon.png"
MODE="${1:-all}"

run_tauri_icon() {
  # Why/为什么: 统一从单一 PNG 源图派生所有桌面/移动端图标，避免不同平台图标漂移。
  # English: Generate all GUI platform icons from the single source PNG to keep branding consistent.
  if command -v tauri >/dev/null 2>&1; then
    (cd "${GUI_DIR}" && tauri icon "icons/icon.png")
    return
  fi

  if command -v cargo-tauri >/dev/null 2>&1; then
    (cd "${GUI_DIR}" && cargo-tauri icon "icons/icon.png")
    return
  fi

  echo "missing tauri icon generator (need 'tauri' or 'cargo-tauri')" >&2
  exit 1
}

sync_ipkg_icon() {
  # Why/为什么: 爱快应用市场图标也必须跟 GUI 主图标保持同源，避免包内外品牌不一致。
  # English: Keep the iKuai package icon derived from the same GUI source icon.
  install -D -m 0644 "${SOURCE_ICON}" "${IPKG_ICON}"
}

sync_web_icons() {
  # Why/为什么: CLI WebUI 浏览器标签页和收藏夹图标也必须与 GUI 主图标同源。
  # English: Keep the CLI WebUI favicon and touch icon derived from the same source icon.
  install -D -m 0644 "${SOURCE_ICON}" "${WEB_FAVICON}"
  install -D -m 0644 "${SOURCE_ICON}" "${WEB_APPLE_TOUCH_ICON}"
}

case "${MODE}" in
  all)
    run_tauri_icon
    sync_ipkg_icon
    sync_web_icons
    ;;
  tauri-only)
    run_tauri_icon
    ;;
  ipkg-only)
    sync_ipkg_icon
    ;;
  web-only)
    sync_web_icons
    ;;
  *)
    echo "usage: $0 [all|tauri-only|ipkg-only|web-only]" >&2
    exit 1
    ;;
esac
