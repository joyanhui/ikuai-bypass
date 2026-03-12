#!/usr/bin/env bash
set -euo pipefail

# Why: buildx needs a deterministic docker/bin layout / 为什么：buildx 需要固定的 docker/bin 目录结构才能稳定挑选目标二进制

release_dir="${1:-release}"
context_dir="${2:-.docker-release/multiarch}"
matrix_json="${DOCKER_CLI_JSON:?DOCKER_CLI_JSON is required}"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
source "${script_dir}/arch-helpers.sh"

rm -rf "${context_dir}"
mkdir -p "${context_dir}/docker/app/frontend" "${context_dir}/docker/bin"

cp Dockerfile docker-entrypoint.sh config.yml "${context_dir}/"
cp -R app/frontend/dist "${context_dir}/docker/app/frontend/dist"
chmod +x "${context_dir}/docker-entrypoint.sh"

while IFS= read -r item; do
  label="$(jq -r '.label' <<<"${item}")"
  target="$(jq -r '.target' <<<"${item}")"
  archive_kind="$(jq -r '.archive' <<<"${item}")"
  package_base="$(ikb_cli_base "${target}")"
  unpack_dir="${context_dir}/unpack-${label}"

  mkdir -p "${context_dir}/docker/bin/${label}" "${unpack_dir}"

  case "${archive_kind}" in
    tar.gz)
      tar -xzf "${release_dir}/${package_base}.tar.gz" -C "${unpack_dir}"
      ;;
    zip)
      unzip -q "${release_dir}/${package_base}.zip" -d "${unpack_dir}"
      ;;
    *)
      printf 'Unsupported archive kind: %s\n' "${archive_kind}" >&2
      exit 1
      ;;
  esac

  cp "${unpack_dir}/ikuai-bypass" "${context_dir}/docker/bin/${label}/ikuai-bypass"
  chmod +x "${context_dir}/docker/bin/${label}/ikuai-bypass"
done < <(printf '%s' "${matrix_json}" | jq -c '.[]')
