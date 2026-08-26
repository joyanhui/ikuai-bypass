#!/usr/bin/env bash
# 文档站构建/预览编排脚本（Jekyll -> Hugo 迁移）。
#
# 原始 Markdown 源文件始终位于 docs/*.md，由 Hugo 直接以 docs/ 为内容目录读取，
# 不再做格式转换/复制。Hugo 工程（配置与自定义模板）位于 docs/hugo/，主题在
# 构建时拉取到仓库根目录的 .hugo-themes/（不纳入 git），以避免内容目录扫描到主题文件。
#
# 用法：
#   bash script/pages.sh build   构建静态站点到 docs/hugo/public
#   bash script/pages.sh serve   本地开发预览（hugo server，http://127.0.0.1:4000）
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "${SCRIPT_DIR}")"
DOCS_DIR="${REPO_ROOT}/docs"
HUGO_DIR="${DOCS_DIR}/hugo"
THEME_DIR="${REPO_ROOT}/.hugo-themes/hextra"
HEXTRA_COMMIT="38d18a5a25d9700dc88888b6e906f1bccf4631b0"

fetch_theme() {
  if [[ -d "${THEME_DIR}/layouts" ]]; then
    return 0
  fi
  echo "==> Fetching Hextra theme (${HEXTRA_COMMIT})"
  rm -rf "${THEME_DIR}"
  git clone --depth 1 https://github.com/imfing/hextra.git "${THEME_DIR}"
  git -C "${THEME_DIR}" fetch --depth 1 origin "${HEXTRA_COMMIT}"
  git -C "${THEME_DIR}" checkout "${HEXTRA_COMMIT}"
}

stage_static() {
  echo "==> Staging static assets (favicon / install.sh / install-file)"
  mkdir -p "${HUGO_DIR}/static"
  cp -f "${DOCS_DIR}/favicon.ico" "${HUGO_DIR}/static/favicon.ico"
  cp -f "${DOCS_DIR}/install.sh" "${HUGO_DIR}/static/install.sh"
  rm -rf "${HUGO_DIR}/static/install-file"
  cp -r "${DOCS_DIR}/install-file" "${HUGO_DIR}/static/install-file"
}

cmd_build() {
  fetch_theme
  stage_static
  echo "==> Building Hugo site"
  hugo --gc --minify -s "${HUGO_DIR}"
}

cmd_serve() {
  fetch_theme
  stage_static
  echo "==> Starting Hugo dev server at http://127.0.0.1:4000"
  exec hugo server \
    -s "${HUGO_DIR}" \
    --bind 127.0.0.1 \
    --port 4000 \
    --baseURL "http://localhost:4000/"
}

case "${1:-}" in
  build) cmd_build ;;
  serve) cmd_serve ;;
  *)
    echo "Usage: bash script/pages.sh {build|serve}"
    exit 1
    ;;
esac
