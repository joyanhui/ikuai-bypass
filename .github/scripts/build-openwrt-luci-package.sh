#!/usr/bin/env bash

set -euo pipefail

if [ "$#" -ne 5 ]; then
  printf 'Usage: %s <repo-root> <output-dir> <version> <package-arch> <artifact-base>\n' "$0" >&2
  exit 1
fi

repo_root="$1"
output_dir="$2"
version="$3"
package_arch="$4"
artifact_base="$5"

template_root="${repo_root}/.github/openwrt/luci-app-ikuai-bypass"
stage_root="$(mktemp -d)"

cleanup() {
  rm -rf "${stage_root}"
}

trap cleanup EXIT

mkdir -p "${stage_root}/pkgroot"
cp -R "${template_root}/root/." "${stage_root}/pkgroot/"

chmod 0755 \
  "${stage_root}/pkgroot/usr/libexec/ikuai-bypass-openwrt" \
  "${template_root}/scripts/postinstall.sh"

nfpm_config="${stage_root}/nfpm-luci-app-ikuai-bypass.yaml"
cat > "${nfpm_config}" <<EOF
name: luci-app-ikuai-bypass
arch: ${package_arch}
platform: linux
version: ${version}
section: luci
priority: optional
maintainer: joyanhui <noreply@users.noreply.github.com>
description: Generic OpenWrt LuCI app for discovering and installing iKuai Bypass CLI from GitHub releases
homepage: https://github.com/joyanhui/ikuai-bypass
license: MIT
depends:
  - luci-base
  - ca-bundle
  - uclient-fetch
  - unzip
contents:
  - src: ${stage_root}/pkgroot/
    dst: /
    type: tree
scripts:
  postinstall: ${template_root}/scripts/postinstall.sh
EOF

mkdir -p "${output_dir}"
nfpm package --packager ipk --config "${nfpm_config}" --target "${output_dir}/"

package_path="$(find "${output_dir}" -maxdepth 1 -type f -name 'luci-app-ikuai-bypass*.ipk' | head -1 || true)"
if [ -z "${package_path}" ] || [ ! -f "${package_path}" ]; then
  printf 'Failed to locate generated LuCI package in %s\n' "${output_dir}" >&2
  exit 1
fi

mv "${package_path}" "${output_dir}/${artifact_base}.ipk"
