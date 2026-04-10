#!/usr/bin/env bash

# Why/为什么: iKuai ipkg 最终必须携带 `manifest.json`，但源码树里不应放一个会被 CI
# 和本地打包脚本原地改写的伪成品文件。这里统一从模板渲染 staging manifest。
# English: The final ipkg must ship `manifest.json`, but the repo should keep a
# template instead of mutating a checked-in artifact. Render the real manifest in staging.

set -euo pipefail

if [ "$#" -ne 3 ]; then
  printf 'Usage: %s <template> <output> <version>\n' "$0" >&2
  exit 1
fi

template="$1"
output="$2"
version="$3"
placeholder='__IKB_IPKG_VERSION__'

if [ ! -f "${template}" ]; then
  printf 'Template not found: %s\n' "${template}" >&2
  exit 1
fi

if [[ ! "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  printf 'Invalid ipkg version: %s\n' "${version}" >&2
  exit 1
fi

if ! grep -q "${placeholder}" "${template}"; then
  printf 'Template placeholder %s not found in %s\n' "${placeholder}" "${template}" >&2
  exit 1
fi

mkdir -p "$(dirname "${output}")"
sed "s/${placeholder}/${version}/g" "${template}" > "${output}"
