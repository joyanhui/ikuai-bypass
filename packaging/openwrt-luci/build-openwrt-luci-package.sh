#!/usr/bin/env bash

# Why/为什么: nfpm 对 OpenWrt IPK 的生成存在兼容性问题，改用纯 tar 手动打包。
# 手动打包的 IPK 格式完全符合 opkg 规范，且无需额外依赖。
# English: manual tar-based IPK builder — no external tools beyond POSIX tar and gzip.

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

template_root="${repo_root}/packaging/openwrt-luci/luci-app-ikuai-bypass"
stage_root="$(mktemp -d)"

cleanup() {
  rm -rf "${stage_root}"
}
trap cleanup EXIT

# ── 1. prepare data.tar.gz ──
data_dir="${stage_root}/data"
mkdir -p "${data_dir}"
cp -R "${template_root}/root/." "${data_dir}/"

# fix permissions
chmod 0755 "${data_dir}/usr/libexec/ikuai-bypass-openwrt"

# build data.tar.gz
tar -czf "${stage_root}/data.tar.gz" -C "${data_dir}" .

# ── 2. prepare control.tar.gz ──
control_dir="${stage_root}/control"
mkdir -p "${control_dir}"

# control file
cat > "${control_dir}/control" <<EOF
Package: luci-app-ikuai-bypass
Version: ${version}
Architecture: ${package_arch}
Maintainer: joyanhui <noreply@users.noreply.github.com>
Section: luci
Priority: optional
Description: iKuai Bypass — LuCI interface. Delegates CLI lifecycle actions to the remote install.sh.
Depends: luci-base, curl, ca-bundle
EOF

# postinst
cp "${template_root}/scripts/postinstall.sh" "${control_dir}/postinst"
chmod 0755 "${control_dir}/postinst"

tar -czf "${stage_root}/control.tar.gz" -C "${control_dir}" .

# ── 3. debian-binary ──
printf '2.0\n' > "${stage_root}/debian-binary"

# ── 4. assemble .ipk ──
mkdir -p "${output_dir}"
tar -czf "${output_dir}/${artifact_base}.ipk" \
  -C "${stage_root}" \
  debian-binary control.tar.gz data.tar.gz

printf 'Built: %s/%s.ipk\n' "${output_dir}" "${artifact_base}"
ls -lh "${output_dir}/${artifact_base}.ipk"
