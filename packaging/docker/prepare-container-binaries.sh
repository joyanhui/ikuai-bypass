#!/usr/bin/env bash
set -euo pipefail

# Why: buildx needs a deterministic docker/bin layout / 为什么：buildx 需要固定的 docker/bin 目录结构才能稳定挑选目标二进制

release_dir="${1:-release}"
context_dir="${2:-.docker-release/multiarch}"
matrix_json="${DOCKER_CLI_JSON:?DOCKER_CLI_JSON is required}"

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=/dev/null
source "${repo_root}/.github/scripts/arch-helpers.sh"

rm -rf "${context_dir}"
mkdir -p "${context_dir}/docker/frontends/app" "${context_dir}/docker/bin"

cp "${repo_root}/packaging/docker/Dockerfile" "${repo_root}/packaging/docker/docker-entrypoint.sh" "${repo_root}/config.yml" "${context_dir}/"
cp -R "${repo_root}/frontends/app/dist" "${context_dir}/docker/frontends/app/dist"
chmod +x "${context_dir}/docker-entrypoint.sh"

while IFS= read -r item; do
  label="$(jq -r '.label' <<<"${item}")"
  target="$(jq -r '.target' <<<"${item}")"
  archive_kind="$(jq -r '.archive' <<<"${item}")"
  package_name="$(ikb_cli_zip_name "${target}")"
  unpack_dir="${context_dir}/unpack-${label}"
  package_path="${release_dir}/${package_name}"

  mkdir -p "${context_dir}/docker/bin/${label}" "${unpack_dir}"

  if [[ -f "${package_path}" ]]; then
    unzip -q "${package_path}" -d "${unpack_dir}"
  else
    printf 'Missing CLI package for docker target: %s (%s, declared archive=%s)\n' "${target}" "${package_name}" "${archive_kind}" >&2
    exit 1
  fi

  cp "${unpack_dir}/ikuai-bypass" "${context_dir}/docker/bin/${label}/ikuai-bypass"
  chmod +x "${context_dir}/docker/bin/${label}/ikuai-bypass"
done < <(printf '%s' "${matrix_json}" | jq -c '.[]')
